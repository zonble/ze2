// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::framebuffer::IndexedColor;
use edit::helpers::*;
use edit::input::vk;
use edit::tui::*;

use crate::commands::{command_from_text, execute_command};
use crate::state::*;

pub fn draw_commandbar(ctx: &mut Context, state: &mut State) {
    let mut execute = false;

    ctx.table_begin("commandbar");
    ctx.attr_focus_well();
    ctx.attr_background_rgba(ctx.indexed(IndexedColor::Green));
    ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::BrightWhite));
    ctx.table_set_cell_gap(Size { width: 1, height: 0 });
    ctx.attr_intrinsic_size(Size { width: COORD_TYPE_SAFE_MAX, height: 1 });
    ctx.attr_padding(Rect::two(0, 1));
    {
        if ctx.contains_focus()
            && ctx.keyboard_input() == Some(vk::ESCAPE)
            && !state.wants_dialog()
            && !ctx.clipboard_ref().wants_host_sync()
        {
            state.command_bar_active = false;
            state.command_bar_error.clear();
            state.wants_editor_focus = true;
            ctx.needs_rerender();
            ctx.set_input_consumed();
        }

        ctx.table_next_row();
        ctx.label("prompt", ">");

        ctx.editline("input", &mut state.command_bar_input);
        ctx.attr_background_rgba(ctx.indexed(IndexedColor::Green));
        ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::BrightWhite));
        ctx.attr_intrinsic_size(Size { width: COORD_TYPE_SAFE_MAX, height: 1 });

        if state.command_bar_focus {
            state.command_bar_focus = false;
            state.command_bar_active = true;
            ctx.steal_focus();
        }

        if ctx.is_focused() {
            state.command_bar_active = true;

            if ctx.consume_shortcut(vk::RETURN) {
                execute = true;
            }
        }

        if !state.command_bar_error.is_empty() {
            ctx.label("error", &state.command_bar_error);
            ctx.attr_overflow(Overflow::TruncateTail);
        }
    }
    ctx.table_end();

    if execute {
        let input = state.command_bar_input.trim();
        if let Some(command) = command_from_text(input) {
            state.command_bar_input.clear();
            state.command_bar_error.clear();
            state.command_bar_active = false;
            state.wants_editor_focus = true;
            execute_command(ctx, state, command);
        } else if !input.is_empty() {
            state.command_bar_error = "Unknown command".to_string();
            ctx.needs_rerender();
        }
    }
}
