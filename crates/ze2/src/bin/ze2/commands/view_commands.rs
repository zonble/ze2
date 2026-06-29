// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::tui::Context;

use super::arguments::command_bool_argument;
use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
use crate::localization::LocId;
use crate::settings::Settings;
use crate::state::*;

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::WordWrap,
        names: &["word-wrap", "wrap"],
        namesVim: &["set-wrap"],
        namesEmacs: &["toggle-truncate-lines", "visual-line-mode"],
        loc_id: Some(LocId::ViewWordWrap),
        default_focus_target: CommandFocusTarget::Default,
        handler: word_wrap,
        argument_hint: Some("<bool>"),
    },
    CommandDefinition {
        command: Command::Reflow,
        names: &["reflow", "rf"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: reflow,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::CenterText,
        names: &["set-center-text", "toggle-center-text", "center-text"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::ViewCenterText),
        default_focus_target: CommandFocusTarget::Default,
        handler: center_text,
        argument_hint: Some("<bool>"),
    },
    CommandDefinition {
        command: Command::ToggleRuler,
        names: &["toggle-ruler", "set-ruler"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::ViewRuler),
        default_focus_target: CommandFocusTarget::Default,
        handler: toggle_ruler,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::ToggleHighlightCurrentChar,
        names: &["toggle-highlight-current-char"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::ViewHighlightCurrentChar),
        default_focus_target: CommandFocusTarget::Default,
        handler: toggle_highlight_current_char,
        argument_hint: None,
    },
];

fn word_wrap(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let mut tb = doc.buffer.borrow_mut();
        let was_enabled = tb.is_word_wrap_enabled();
        let word_wrap =
            command_bool_argument(&args.argument).unwrap_or_else(|| !tb.is_word_wrap_enabled());
        tb.set_word_wrap(word_wrap);
        if !was_enabled && word_wrap {
            tb.clear_mark();
        }
        drop(tb);
        if let Err(err) = Settings::set_word_wrap(word_wrap) {
            error_log_add(ctx, state, err);
        }
    }
}

fn reflow(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let mut tb = doc.buffer.borrow_mut();
        let right = if state.reflow_right_margin > 0 {
            state.reflow_right_margin
        } else {
            tb.word_wrap_max_column()
        };
        if right > 0 {
            tb.reflow_document(state.reflow_left_margin, right, state.reflow_paragraph_margin);
        } else {
            tb.reflow();
        }
    }
}

fn center_text(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let center_text = command_bool_argument(&args.argument).unwrap_or(!state.wants_center_text);
    state.wants_center_text = center_text;
    if let Err(err) = Settings::set_center_text(center_text) {
        error_log_add(ctx, state, err);
    }
}

fn toggle_ruler(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.wants_ruler = !state.wants_ruler;
    if let Err(err) = Settings::set_ruler(state.wants_ruler) {
        error_log_add(ctx, state, err);
    }
}

fn toggle_highlight_current_char(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.highlight_current_char = !state.highlight_current_char;
    if let Err(err) = Settings::set_highlight_current_char(state.highlight_current_char) {
        error_log_add(ctx, state, err);
    }
}
