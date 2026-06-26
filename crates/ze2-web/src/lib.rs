// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(dead_code)]

use std::alloc::{Layout, alloc, dealloc};
use std::ptr::{self, NonNull};
use std::slice;
use std::sync::atomic::{AtomicI32, Ordering};

use stdext::arena::{self, scratch_arena};
use ze2::framebuffer::IndexedColor;
use ze2::helpers::{CoordType, MEBI, Point, Size};
use ze2::input::{Input, kbmod, vk};
use ze2::tui::{Context, ModifierTranslations, Tui};
use ze2::{input, vt};

#[path = "../../ze2/src/bin/ze2/apperr.rs"]
mod apperr;
mod commands;
#[path = "../../ze2/src/bin/ze2/documents.rs"]
mod documents;
#[path = "../../ze2/src/bin/ze2/draw_commandbar.rs"]
mod draw_commandbar;
#[path = "../../ze2/src/bin/ze2/draw_editor.rs"]
mod draw_editor;
#[path = "../../ze2/src/bin/ze2/draw_menubar.rs"]
mod draw_menubar;
#[path = "../../ze2/src/bin/ze2/draw_statusbar.rs"]
mod draw_statusbar;
#[path = "../../ze2/src/bin/ze2/input_routing.rs"]
mod input_routing;
#[path = "../../ze2/src/bin/ze2/localization.rs"]
mod localization;
#[path = "../../ze2/src/bin/ze2/settings.rs"]
mod settings;
#[path = "../../ze2/src/bin/ze2/state.rs"]
mod state;

use commands::*;
use draw_commandbar::*;
use draw_editor::*;
use draw_menubar::*;
use draw_statusbar::*;
use input_routing::*;
use localization::*;
use settings::{EditorColor, Settings};
use state::*;

struct Engine {
    tui: Tui,
    state: State,
    vt_parser: vt::Parser,
    input_parser: input::Parser,
    output: Vec<u8>,
    document: Vec<u8>,
    host_action: HostAction,
}

static mut ENGINE: *mut Engine = ptr::null_mut();
static PENDING_HOST_ACTION: AtomicI32 = AtomicI32::new(HostAction::None as i32);

#[repr(i32)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum HostAction {
    #[default]
    None = 0,
    Open = 1,
    Save = 2,
    ClipboardRead = 3,
    ClipboardWrite = 4,
}

pub fn request_host_clipboard_read() {
    PENDING_HOST_ACTION.store(HostAction::ClipboardRead as i32, Ordering::Relaxed);
}

impl Engine {
    fn new(width: i32, height: i32) -> Result<Self, &'static str> {
        localization::init();
        let _ = Settings::reload();

        let mut tui = Tui::new().map_err(|_| "failed to create TUI")?;
        tui.setup_modifier_translations(ModifierTranslations {
            ctrl: loc(LocId::Ctrl),
            alt: loc(LocId::Alt),
            shift: loc(LocId::Shift),
        });
        tui.set_eof_marker(loc(LocId::EndOfFileMarker));

        let floater_bg = tui
            .indexed_alpha(IndexedColor::Background, 2, 3)
            .oklab_blend(tui.indexed_alpha(IndexedColor::Foreground, 1, 3));
        let floater_fg = tui.contrasted(floater_bg);
        tui.set_floater_default_bg(floater_bg);
        tui.set_floater_default_fg(floater_fg);
        tui.set_modal_default_bg(floater_bg);
        tui.set_modal_default_fg(floater_fg);

        let mut state = State::new().map_err(|_| "failed to create editor state")?;
        state.documents.add_untitled().map_err(|_| "failed to create document")?;
        state.wants_editor_focus = true;

        let mut engine = Self {
            tui,
            state,
            vt_parser: vt::Parser::new(),
            input_parser: input::Parser::new(),
            output: Vec::new(),
            document: Vec::new(),
            host_action: HostAction::None,
        };
        engine.frame(Some(Input::Resize(Size {
            width: clamp_size(width),
            height: clamp_size(height),
        })));
        Ok(engine)
    }

    fn frame(&mut self, input: Option<Input<'_>>) {
        {
            let mut ctx = self.tui.create_context(input);
            draw(&mut ctx, &mut self.state);
        }

        while self.tui.needs_settling() {
            let mut ctx = self.tui.create_context(None);
            draw(&mut ctx, &mut self.state);
        }

        let scratch = scratch_arena(None);
        let output = self.tui.render(&scratch);
        self.output.clear();
        self.output.extend_from_slice(output.as_bytes());
        self.collect_host_action();
        self.refresh_document_cache();
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.frame(Some(Input::Resize(Size {
            width: clamp_size(width),
            height: clamp_size(height),
        })));
    }

    fn redraw_full(&mut self) {
        let size = self.tui.size();
        self.frame(Some(Input::Resize(Size { width: 1, height: 1 })));
        self.frame(Some(Input::Resize(size)));
    }

    fn input(&mut self, input: &str) {
        let vt_iter = self.vt_parser.parse(input);
        let mut events = Vec::new();
        {
            let mut input_iter = self.input_parser.parse(vt_iter);
            while let Some(event) = input_iter.next() {
                events.push(owned_input(event));
            }
        }

        for event in &events {
            if matches!(event, OwnedInput::Keyboard(key) if *key == (kbmod::CTRL | vk::V)) {
                self.host_action = HostAction::ClipboardRead;
                continue;
            }
            self.frame(Some(borrow_input(event)));
        }
    }

    fn paste(&mut self, text: &[u8]) {
        self.frame(Some(Input::Paste(text.to_vec())));
    }

    fn set_document(&mut self, text: &str) {
        while self.state.documents.len() != 0 {
            self.state.documents.remove_active();
        }

        if let Ok(doc) = self.state.documents.add_untitled() {
            let mut tb = doc.buffer.borrow_mut();
            tb.write_raw(text.as_bytes());
            tb.cursor_move_to_logical(Point::default());
            tb.mark_as_clean();
        }

        self.state.wants_editor_focus = true;
        self.frame(None);
    }

    fn apply_settings(
        &mut self,
        word_wrap: bool,
        word_wrap_column: CoordType,
        ruler: bool,
        center_text: bool,
        highlight_current_char: bool,
        editor_color: EditorColor,
    ) {
        let _ = Settings::set_word_wrap(word_wrap);
        let _ = Settings::set_word_wrap_column(word_wrap_column);
        let _ = Settings::set_ruler(ruler);
        let _ = Settings::set_center_text(center_text);
        let _ = Settings::set_highlight_current_char(highlight_current_char);
        let _ = Settings::set_editor_color(editor_color);

        self.state.wants_ruler = ruler;
        self.state.wants_center_text = center_text;
        self.state.highlight_current_char = highlight_current_char;
        self.state.editor_color = editor_color;

        if let Some(doc) = self.state.documents.active() {
            let mut tb = doc.buffer.borrow_mut();
            tb.set_word_wrap(word_wrap);
            tb.set_word_wrap_max_column(word_wrap_column);
        }

        self.frame(None);
    }

    fn refresh_document_cache(&mut self) {
        self.document.clear();
        let Some(doc) = self.state.documents.active() else {
            return;
        };

        let tb = doc.buffer.borrow();
        let mut off = 0;
        loop {
            let chunk = tb.read_forward(off);
            if chunk.is_empty() {
                break;
            }
            self.document.extend_from_slice(chunk);
            off += chunk.len();
        }
    }

    fn collect_host_action(&mut self) {
        let pending = PENDING_HOST_ACTION.swap(HostAction::None as i32, Ordering::Relaxed);
        if pending == HostAction::ClipboardRead as i32 {
            self.host_action = HostAction::ClipboardRead;
            return;
        }

        match self.state.wants_file_picker {
            StateFilePicker::Open => {
                self.state.wants_file_picker = StateFilePicker::None;
                self.host_action = HostAction::Open;
            }
            StateFilePicker::SaveAs | StateFilePicker::SaveAsShown => {
                self.state.wants_file_picker = StateFilePicker::None;
                self.state.wants_save = false;
                self.host_action = HostAction::Save;
            }
            StateFilePicker::None => {}
        }

        if self.tui.clipboard_ref().wants_host_sync() {
            self.host_action = HostAction::ClipboardWrite;
        }
    }

    fn take_host_action(&mut self) -> HostAction {
        let action = self.host_action;
        self.host_action = HostAction::None;
        action
    }
}

enum OwnedInput {
    Resize(Size),
    Text(String),
    Paste(Vec<u8>),
    Keyboard(ze2::input::InputKey),
    Mouse(ze2::input::InputMouse),
}

fn owned_input(input: Input<'_>) -> OwnedInput {
    match input {
        Input::Resize(size) => OwnedInput::Resize(size),
        Input::Text(text) => OwnedInput::Text(text.to_string()),
        Input::Paste(paste) => OwnedInput::Paste(paste),
        Input::Keyboard(key) => OwnedInput::Keyboard(key),
        Input::Mouse(mouse) => OwnedInput::Mouse(mouse),
    }
}

fn borrow_input(input: &OwnedInput) -> Input<'_> {
    match input {
        OwnedInput::Resize(size) => Input::Resize(*size),
        OwnedInput::Text(text) => Input::Text(text),
        OwnedInput::Paste(paste) => Input::Paste(paste.clone()),
        OwnedInput::Keyboard(key) => Input::Keyboard(*key),
        OwnedInput::Mouse(mouse) => Input::Mouse(*mouse),
    }
}

fn draw(ctx: &mut Context, state: &mut State) {
    draw_menubar(ctx, state, false);

    if let Some(invocation) = handle_input_before_editor(ctx, state) {
        execute_command_invocation(ctx, state, invocation);
        ctx.set_input_consumed();
    }

    draw_editor(ctx, state);
    draw_commandbar(ctx, state);
    draw_statusbar(ctx, state);

    if state.wants_menubar_focus {
        state.wants_menubar_focus = false;
        state.menubar_visible = true;
        draw_menubar(ctx, state, true);
    }

    if state.wants_close {
        draw_handle_wants_close(ctx, state);
    }
    if state.wants_exit {
        draw_handle_wants_exit(ctx, state);
    }
    if state.wants_goto {
        draw_goto_menu(ctx, state);
    }
    // The browser host owns file picking. Keep the state flag for Engine to
    // convert into a JS host action instead of rendering the native file picker.
    if state.wants_save {
        draw_handle_save(ctx, state);
    }
    if state.wants_language_picker {
        draw_dialog_language_change(ctx, state);
    }
    if state.wants_encoding_change != StateEncodingChange::None {
        draw_dialog_encoding_change(ctx, state);
    }
    if state.wants_go_to_file {
        draw_go_to_file(ctx, state);
    }
    if state.wants_about {
        draw_dialog_about(ctx, state);
    }
    if state.wants_word_count {
        draw_dialog_word_count(ctx, state);
    }
    if state.error_log_count != 0 {
        draw_error_log(ctx, state);
    }

    if let Some(key) = ctx.keyboard_input() {
        if let Some(invocation) = command_invocation_from_shortcut(key) {
            execute_command_invocation(ctx, state, invocation);
        } else if key == ze2::input::vk::F3 {
            search_execute(ctx, state, SearchAction::Search);
        } else {
            return;
        }

        ctx.needs_rerender();
        ctx.set_input_consumed();
    }
}

fn draw_handle_wants_exit(_ctx: &mut Context, state: &mut State) {
    while let Some(doc) = state.documents.active() {
        if doc.buffer.borrow().is_dirty() {
            state.wants_close = true;
            return;
        }
        state.documents.remove_active();
    }

    if state.documents.len() == 0 {
        state.exit = true;
    }
}

fn clamp_size(value: i32) -> CoordType {
    value.clamp(1, 32767) as CoordType
}

fn with_engine<T>(default: T, f: impl FnOnce(&mut Engine) -> T) -> T {
    unsafe {
        let Some(engine) = ENGINE.as_mut() else {
            return default;
        };
        f(engine)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_init(width: i32, height: i32) -> i32 {
    if arena::init(8 * MEBI).is_err() {
        return 0;
    }

    match Engine::new(width, height) {
        Ok(engine) => unsafe {
            if !ENGINE.is_null() {
                drop(Box::from_raw(ENGINE));
            }
            ENGINE = Box::into_raw(Box::new(engine));
            1
        },
        Err(_) => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_resize(width: i32, height: i32) {
    with_engine((), |engine| engine.resize(width, height));
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_redraw() {
    with_engine((), |engine| engine.redraw_full());
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ze2_web_input(ptr: *const u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    let input = String::from_utf8_lossy(bytes);
    with_engine((), |engine| engine.input(&input));
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_flush_input() {
    with_engine((), |engine| engine.input(""));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ze2_web_paste(ptr: *const u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    with_engine((), |engine| engine.paste(bytes));
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_take_host_action() -> i32 {
    with_engine(HostAction::None as i32, |engine| engine.take_host_action() as i32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ze2_web_set_document(ptr: *const u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    let text = String::from_utf8_lossy(bytes);
    with_engine((), |engine| engine.set_document(&text));
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_apply_settings(
    word_wrap: i32,
    word_wrap_column: i32,
    ruler: i32,
    center_text: i32,
    highlight_current_char: i32,
    editor_color: i32,
) {
    with_engine((), |engine| {
        engine.apply_settings(
            word_wrap != 0,
            word_wrap_column.max(0) as CoordType,
            ruler != 0,
            center_text != 0,
            highlight_current_char != 0,
            if editor_color == 1 { EditorColor::WhiteOnBlue } else { EditorColor::Original },
        )
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_setting_word_wrap() -> i32 {
    with_engine(Settings::borrow().word_wrap as i32, |engine| {
        engine
            .state
            .documents
            .active()
            .map_or(Settings::borrow().word_wrap, |doc| doc.buffer.borrow().is_word_wrap_enabled())
            as i32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_setting_word_wrap_column() -> i32 {
    with_engine(Settings::borrow().word_wrap_column as i32, |engine| {
        engine.state.documents.active().map_or(Settings::borrow().word_wrap_column, |doc| {
            doc.buffer.borrow().word_wrap_max_column()
        }) as i32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_setting_ruler() -> i32 {
    with_engine(Settings::borrow().ruler as i32, |engine| engine.state.wants_ruler as i32)
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_setting_center_text() -> i32 {
    with_engine(Settings::borrow().center_text as i32, |engine| {
        engine.state.wants_center_text as i32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_setting_highlight_current_char() -> i32 {
    with_engine(Settings::borrow().highlight_current_char as i32, |engine| {
        engine.state.highlight_current_char as i32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_setting_editor_color() -> i32 {
    with_engine(0, |engine| (engine.state.editor_color == EditorColor::WhiteOnBlue) as i32)
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_document_ptr() -> *const u8 {
    with_engine(ptr::null(), |engine| engine.document.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_document_len() -> usize {
    with_engine(0, |engine| engine.document.len())
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_clipboard_ptr() -> *const u8 {
    with_engine(ptr::null(), |engine| engine.tui.clipboard_ref().read().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_clipboard_len() -> usize {
    with_engine(0, |engine| engine.tui.clipboard_ref().read().len())
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_mark_clipboard_synced() {
    with_engine((), |engine| engine.tui.clipboard_mut().mark_as_synchronized());
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_output_ptr() -> *const u8 {
    with_engine(ptr::null(), |engine| engine.output.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_output_len() -> usize {
    with_engine(0, |engine| engine.output.len())
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_alloc(size: usize) -> *mut u8 {
    let Ok(layout) = Layout::from_size_align(size.max(1), 8) else {
        return ptr::null_mut();
    };
    unsafe { alloc(layout) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ze2_web_dealloc(ptr: *mut u8, size: usize) {
    if let Some(ptr) = NonNull::new(ptr)
        && let Ok(layout) = Layout::from_size_align(size.max(1), 8)
    {
        unsafe { dealloc(ptr.as_ptr(), layout) };
    }
}
