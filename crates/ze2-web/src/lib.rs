// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::alloc::{Layout, alloc, dealloc};
use std::ptr::{self, NonNull};
use std::slice;

use stdext::arena::{self, scratch_arena};
use ze2::helpers::{CoordType, MEBI, Rect, Size};
use ze2::input::{Input, vk};
use ze2::tui::{ButtonStyle, Context, Position, Tui};

struct Engine {
    tui: Tui,
    text: String,
    status: String,
    output: Vec<u8>,
}

static mut ENGINE: *mut Engine = ptr::null_mut();

impl Engine {
    fn new(width: i32, height: i32) -> Result<Self, &'static str> {
        let tui = Tui::new().map_err(|_| "failed to create TUI")?;
        let mut engine = Self {
            tui,
            text: "Edit this text from xterm.js".to_string(),
            status: "Browser WASM POC".to_string(),
            output: Vec::new(),
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
            draw(&mut ctx, &mut self.text, &self.status);
        }

        while self.tui.needs_settling() {
            let mut ctx = self.tui.create_context(None);
            draw(&mut ctx, &mut self.text, &self.status);
        }

        let scratch = scratch_arena(None);
        let output = self.tui.render(&scratch);
        self.output.clear();
        self.output.extend_from_slice(output.as_bytes());
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.frame(Some(Input::Resize(Size {
            width: clamp_size(width),
            height: clamp_size(height),
        })));
    }

    fn input(&mut self, input: &str) {
        let mut off = 0;
        while off < input.len() {
            let rest = &input[off..];

            if let Some((event, len)) = parse_key(rest) {
                self.frame(Some(event));
                off += len;
                continue;
            }

            let Some((text, len)) = next_text_run(rest) else {
                let ch = rest.chars().next().unwrap();
                off += ch.len_utf8();
                continue;
            };

            self.frame(Some(Input::Text(text)));
            off += len;
        }
    }

    fn set_document(&mut self, text: &str) {
        self.text.clear();
        self.text.push_str(text);
        self.status.clear();
        self.status.push_str("Loaded browser file");
        self.frame(None);
    }
}

fn draw(ctx: &mut Context<'_, '_>, text: &mut String, status: &str) {
    ctx.block_begin("root");
    ctx.attr_focus_well();
    ctx.attr_padding(Rect::three(1, 2, 1));

    ctx.label("title", "ze2 web POC");
    ctx.label("status", status);

    ctx.block_begin("editor-wrap");
    ctx.attr_border();
    ctx.attr_padding(Rect::one(1));
    ctx.editline("editor", text);
    ctx.focus_on_first_present();
    ctx.block_end();

    ctx.table_begin("toolbar");
    ctx.attr_position(Position::Left);
    ctx.table_set_cell_gap(Size { width: 2, height: 0 });
    ctx.table_next_row();
    if ctx.button("noop", "OK", ButtonStyle::default()) {
        ctx.needs_rerender();
    }
    ctx.label("hint", "Open/save is handled by the browser toolbar.");
    ctx.table_end();

    ctx.block_end();
}

fn clamp_size(value: i32) -> CoordType {
    value.clamp(1, 32767) as CoordType
}

fn parse_key(input: &str) -> Option<(Input<'_>, usize)> {
    let bytes = input.as_bytes();
    match bytes {
        [b'\r', ..] => Some((Input::Keyboard(vk::RETURN), 1)),
        [b'\t', ..] => Some((Input::Keyboard(vk::TAB), 1)),
        [0x7f, ..] | [0x08, ..] => Some((Input::Keyboard(vk::BACK), 1)),
        [0x1b, b'[', b'A', ..] => Some((Input::Keyboard(vk::UP), 3)),
        [0x1b, b'[', b'B', ..] => Some((Input::Keyboard(vk::DOWN), 3)),
        [0x1b, b'[', b'C', ..] => Some((Input::Keyboard(vk::RIGHT), 3)),
        [0x1b, b'[', b'D', ..] => Some((Input::Keyboard(vk::LEFT), 3)),
        [0x1b, b'[', b'H', ..] => Some((Input::Keyboard(vk::HOME), 3)),
        [0x1b, b'[', b'F', ..] => Some((Input::Keyboard(vk::END), 3)),
        [0x1b, b'[', b'3', b'~', ..] => Some((Input::Keyboard(vk::DELETE), 4)),
        _ => None,
    }
}

fn next_text_run(input: &str) -> Option<(&str, usize)> {
    let mut end = 0;
    for (idx, ch) in input.char_indices() {
        if ch.is_control() {
            break;
        }
        end = idx + ch.len_utf8();
    }

    if end == 0 { None } else { Some((&input[..end], end)) }
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
    with_engine(ptr::null(), |engine| engine.text.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn ze2_web_document_len() -> usize {
    with_engine(0, |engine| engine.text.len())
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
