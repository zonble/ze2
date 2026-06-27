// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::framebuffer::IndexedColor;
use ze2::helpers::*;
use ze2::input::vk;
use ze2::tui::*;

use crate::commands::{
    autocomplete_command_suggestions_with_modes, command_from_text_with_modes,
    execute_command_invocation,
};
use crate::localization::{LocId, loc};
use crate::state::*;

pub fn draw_commandbar(ctx: &mut Context, state: &mut State) {
    let mut should_submit_input = false;
    let mut has_error = !state.command_bar_error.is_empty();

    ctx.table_begin("commandbar");
    ctx.attr_focus_well();
    ctx.attr_background_rgba(ctx.indexed(IndexedColor::Green));
    ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::BrightWhite));
    ctx.table_set_cell_gap(Size { width: 1, height: 0 });
    ctx.attr_intrinsic_size(Size {
        width: COORD_TYPE_SAFE_MAX,
        height: 1,
    });
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

        if ctx.editline("input", &mut state.command_bar_input) {
            state.command_bar_error.clear();
            has_error = false;
        }
        ctx.attr_background_rgba(ctx.indexed(IndexedColor::Green));
        ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::BrightWhite));
        ctx.attr_intrinsic_size(Size { width: COORD_TYPE_SAFE_MAX, height: 1 });
        ctx.inherit_focus();

        if ctx.contains_focus() {
            let suggestions = if !state.command_bar_input.is_empty()
                && !state.command_bar_input.contains(char::is_whitespace)
            {
                autocomplete_command_suggestions_with_modes(
                    &state.command_bar_input,
                    state.command_bar_include_vim_commands,
                    state.command_bar_include_emacs_commands,
                )
            } else {
                Vec::new()
            };

            if suggestions.is_empty() {
                state.command_bar_autocomplete_index = None;
                if ctx.is_focused() && ctx.consume_shortcut(vk::UP) {
                    state.command_bar_active = false;
                    state.command_bar_error.clear();
                    state.wants_editor_focus = true;
                    ctx.needs_rerender();
                }
            } else {
                let bg = ctx.indexed_alpha(IndexedColor::Background, 3, 4);
                let fg = ctx.contrasted(bg);

                let mut apply_autocomplete = false;
                // Handle keyboard navigation manually before the list takes it
                if ctx.is_focused() {
                    if ctx.consume_shortcut(vk::DOWN) {
                        let idx = state.command_bar_autocomplete_index.unwrap_or(0);
                        state.command_bar_autocomplete_index =
                            Some((idx + 1).min(suggestions.len() - 1));
                    } else if ctx.consume_shortcut(vk::UP) {
                        let idx = state.command_bar_autocomplete_index.unwrap_or(0);
                        state.command_bar_autocomplete_index = Some(idx.saturating_sub(1));
                    } else if ctx.consume_shortcut(vk::TAB) {
                        apply_autocomplete = true;
                    } else if ctx.consume_shortcut(vk::RETURN) {
                        should_submit_input = true;
                    }
                }

                if apply_autocomplete {
                    if let Some(idx) = state.command_bar_autocomplete_index {
                        if let Some(suggestion) = suggestions.get(idx) {
                            state.command_bar_input = suggestion.name.clone();
                            state.command_bar_autocomplete_index = None;
                        }
                    } else if let Some(suggestion) = suggestions.first() {
                        state.command_bar_input = suggestion.name.clone();
                        state.command_bar_autocomplete_index = None;
                    }
                }

                ctx.block_begin("suggestions");
                ctx.attr_float(FloatSpec {
                    anchor: Anchor::Last,
                    gravity_x: 0.0,
                    gravity_y: 1.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                });
                ctx.attr_border();
                ctx.attr_background_rgba(bg);
                ctx.attr_foreground_rgba(fg);
                {
                    for (idx, suggestion) in suggestions.iter().enumerate() {
                        let is_selected = state.command_bar_autocomplete_index == Some(idx);

                        ctx.next_block_id_mixin(idx as u64);
                        ctx.styled_label_begin("suggestion");
                        if is_selected {
                            ctx.attr_background_rgba(ctx.indexed(IndexedColor::White));
                            ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::Black));
                        }
                        ctx.styled_label_add_text("  ");
                        let text = suggestion.display_text();
                        ctx.styled_label_add_text(&text);
                        ctx.styled_label_end();
                    }
                }
                ctx.block_end();

                // If user typed anything else, reset the autocomplete selection
                if ctx.keyboard_input().is_some()
                    && ctx.keyboard_input() != Some(vk::RETURN)
                    && ctx.keyboard_input() != Some(vk::TAB)
                {
                    state.command_bar_autocomplete_index = None;
                }
            }
        } else {
            state.command_bar_autocomplete_index = None;
        }

        if state.command_bar_focus {
            state.command_bar_focus = false;
            state.command_bar_active = true;
            ctx.steal_focus();
        }

        if ctx.contains_focus() {
            state.command_bar_active = true;

            if ctx.consume_shortcut(vk::RETURN) {
                should_submit_input = true;
            }
        }

        if has_error {
            ctx.block_begin("error_box");
            ctx.attr_float(FloatSpec {
                anchor: Anchor::Parent,
                gravity_x: 0.0,
                gravity_y: 1.0,
                offset_x: 0.0,
                offset_y: 0.0,
            });
            ctx.attr_border();
            ctx.attr_background_rgba(ctx.indexed(IndexedColor::Red));
            ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::BrightWhite));
            ctx.attr_padding(Rect::two(0, 1));
            {
                ctx.label("error", &state.command_bar_error);
                ctx.attr_overflow(Overflow::TruncateTail);
            }
            ctx.block_end();
        }
    }
    ctx.table_end();

    if should_submit_input {
        submit_commandbar_input(ctx, state);
    }
}

fn submit_commandbar_input(ctx: &mut Context, state: &mut State) {
    let input = state.command_bar_input.trim();

    if let Some(invocations) = command_macro_from_text(
        input,
        state.command_bar_include_vim_commands,
        state.command_bar_include_emacs_commands,
    ) {
        state.command_bar_input.clear();
        state.command_bar_error.clear();
        state.command_bar_active = false;
        state.wants_editor_focus = true;
        for invocation in invocations {
            execute_command_invocation(ctx, state, invocation);
        }
    } else if let Some(invocation) = command_from_text_with_modes(
        input,
        state.command_bar_include_vim_commands,
        state.command_bar_include_emacs_commands,
    ) {
        state.command_bar_input.clear();
        state.command_bar_error.clear();
        state.command_bar_active = false;
        state.wants_editor_focus = true;
        execute_command_invocation(ctx, state, invocation);
    } else if !input.is_empty() {
        state.command_bar_input.clear();
        state.command_bar_error = loc(LocId::CommandBarUnknownCommand).to_string();
        ctx.needs_rerender();
    }
}

fn command_macro_from_text(
    input: &str,
    include_vim_commands: bool,
    include_emacs_commands: bool,
) -> Option<Vec<crate::commands::CommandInvocation>> {
    let mut rest = input.trim();
    if !rest.starts_with('[') {
        return None;
    }

    let mut invocations = Vec::new();
    while !rest.is_empty() {
        rest = rest.strip_prefix('[')?;
        let (command, tail) = rest.split_once(']')?;
        invocations.push(command_from_text_with_modes(
            command.trim(),
            include_vim_commands,
            include_emacs_commands,
        )?);
        rest = tail.trim_start();
    }

    Some(invocations)
}
