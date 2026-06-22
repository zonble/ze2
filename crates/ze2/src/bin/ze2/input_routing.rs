// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::input::vk;
use ze2::tui::Context;

use crate::commands::{
    CommandInvocation, command_invocation_from_shortcut, commandbar_shortcut_from_key,
    should_handle_command_shortcut_before_editor,
};
use crate::state::{State, StateSearchKind};

pub fn handle_input_before_editor(
    ctx: &mut Context,
    state: &mut State,
) -> Option<CommandInvocation> {
    if handle_commandbar_shortcut(ctx, state) {
        ctx.set_input_consumed();
        return None;
    }

    insert_text_invocation_before_editor(ctx, state)
}

fn handle_commandbar_shortcut(ctx: &Context, state: &mut State) -> bool {
    if close_commandbar_before_editor(ctx, state) || focus_commandbar_before_editor(ctx, state) {
        return true;
    }

    false
}

fn close_commandbar_before_editor(ctx: &Context, state: &mut State) -> bool {
    if !state.command_bar_active
        || ctx.keyboard_input() != Some(vk::ESCAPE)
        || state.wants_dialog()
        || ctx.clipboard_ref().wants_host_sync()
    {
        return false;
    }

    state.command_bar_active = false;
    state.command_bar_focus = false;
    state.command_bar_error.clear();
    state.wants_editor_focus = true;
    true
}

fn focus_commandbar_before_editor(ctx: &Context, state: &mut State) -> bool {
    if !matches!(state.wants_search.kind, StateSearchKind::Hidden | StateSearchKind::Disabled)
        || state.wants_dialog()
        || ctx.clipboard_ref().wants_host_sync()
    {
        return false;
    }

    if ctx.keyboard_input() == Some(vk::ESCAPE) {
        state.command_bar_focus = true;
        return true;
    }

    let Some(shortcut) = ctx.keyboard_input().and_then(commandbar_shortcut_from_key) else {
        return false;
    };

    state.command_bar_input.clear();
    state.command_bar_input.push_str(shortcut.text);
    state.command_bar_error.clear();
    state.command_bar_active = true;
    state.command_bar_focus = true;
    true
}

fn insert_text_invocation_before_editor(ctx: &Context, state: &State) -> Option<CommandInvocation> {
    if state.command_bar_active
        || !matches!(state.wants_search.kind, StateSearchKind::Hidden | StateSearchKind::Disabled)
        || state.wants_dialog()
        || ctx.clipboard_ref().wants_host_sync()
    {
        return None;
    }

    let invocation = command_invocation_from_shortcut(ctx.keyboard_input()?)?;
    should_handle_command_shortcut_before_editor(invocation.command).then_some(invocation)
}
