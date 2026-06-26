// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(dead_code)]

use std::alloc::{Layout, alloc, dealloc};
use std::ptr::{self, NonNull};
use std::slice;

use stdext::arena::{self, scratch_arena};
use ze2::framebuffer::IndexedColor;
use ze2::helpers::{CoordType, MEBI, Size};
use ze2::input::Input;
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
#[path = "../../ze2/src/bin/ze2/draw_filepicker.rs"]
mod draw_filepicker;
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
use draw_filepicker::*;
use draw_menubar::*;
use draw_statusbar::*;
use input_routing::*;
use localization::*;
use state::*;

struct Engine {
    tui: Tui,
    state: State,
    vt_parser: vt::Parser,
    input_parser: input::Parser,
    output: Vec<u8>,
    document: Vec<u8>,
}

static mut ENGINE: *mut Engine = ptr::null_mut();

impl Engine {
    fn new(width: i32, height: i32) -> Result<Self, &'static str> {
        localization::init();
        let _ = settings::Settings::reload();

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
        self.refresh_document_cache();
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.frame(Some(Input::Resize(Size {
            width: clamp_size(width),
            height: clamp_size(height),
        })));
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
            self.frame(Some(borrow_input(event)));
        }
    }

    fn set_document(&mut self, text: &str) {
        if self.state.documents.len() == 0 {
            let _ = self.state.documents.add_untitled();
        }

        if let Some(doc) = self.state.documents.active_mut() {
            let mut tb = doc.buffer.borrow_mut();
            let text = text.to_string();
            tb.copy_from_str(&text);
            tb.mark_as_clean();
        }

        self.state.wants_editor_focus = true;
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
    if state.wants_file_picker != StateFilePicker::None {
        draw_file_picker(ctx, state);
    }
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
pub unsafe extern "C" fn ze2_web_input(ptr: *const u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    let input = String::from_utf8_lossy(bytes);
    with_engine((), |engine| engine.input(&input));
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
pub extern "C" fn ze2_web_document_ptr() -> *const u8 {
    with_engine(ptr::null(), |engine| engine.document.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_document_len() -> usize {
    with_engine(0, |engine| engine.document.len())
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
