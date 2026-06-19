// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::tui::Context;

use super::arguments::{command_bool_argument, command_editor_color_argument};
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
    },
    CommandDefinition {
        command: Command::SetWordWrapColumn,
        names: &["set-word-wrap-column", "set-wrap-column"],
        namesVim: &["set-textwidth"],
        namesEmacs: &["set-fill-column"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_word_wrap_column,
    },
    CommandDefinition {
        command: Command::CenterText,
        names: &["set-center-text", "toggle-center-text", "center-text"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::ViewCenterText),
        default_focus_target: CommandFocusTarget::Default,
        handler: center_text,
    },
    CommandDefinition {
        command: Command::SetHighlightCurrentChar,
        names: &["set-highlight-current-char"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_highlight_current_char,
    },
    CommandDefinition {
        command: Command::ToggleHighlightCurrentChar,
        names: &["toggle-highlight-current-char"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::ViewHighlightCurrentChar),
        default_focus_target: CommandFocusTarget::Default,
        handler: toggle_highlight_current_char,
    },
    CommandDefinition {
        command: Command::SetEditorColor,
        names: &["set-editor-color"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_editor_color,
    },
];

fn word_wrap(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let mut tb = doc.buffer.borrow_mut();
        let word_wrap =
            command_bool_argument(&args.argument).unwrap_or_else(|| !tb.is_word_wrap_enabled());
        tb.set_word_wrap(word_wrap);
        drop(tb);
        if let Err(err) = Settings::set_word_wrap(word_wrap) {
            error_log_add(ctx, state, err);
        }
    }
}

fn set_word_wrap_column(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let col =
        args.argument.as_deref().and_then(|s| s.trim().parse::<isize>().ok()).unwrap_or(0).max(0);
    // Enforce a minimum of 20 columns (0 means "no limit / full window width").
    let col = if col > 0 { col.max(20) } else { 0 };

    let mut err_to_log = None;
    if let Some(doc) = state.documents.active() {
        let mut tb = doc.buffer.borrow_mut();
        tb.set_word_wrap_max_column(col);
        if col > 0 && !tb.is_word_wrap_enabled() {
            tb.set_word_wrap(true);
            drop(tb);
            if let Err(err) = Settings::set_word_wrap(true) {
                err_to_log = Some(err);
            }
        }
    }
    if let Err(err) = Settings::set_word_wrap_column(col) {
        err_to_log = Some(err);
    }
    if let Some(err) = err_to_log {
        error_log_add(ctx, state, err);
    }
}

fn center_text(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let center_text = command_bool_argument(&args.argument).unwrap_or(!state.wants_center_text);
    state.wants_center_text = center_text;
    if let Err(err) = Settings::set_center_text(center_text) {
        error_log_add(ctx, state, err);
    }
}

fn set_highlight_current_char(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(highlight_current_char) = command_bool_argument(&args.argument) {
        state.highlight_current_char = highlight_current_char;
        if let Err(err) = Settings::set_highlight_current_char(state.highlight_current_char) {
            error_log_add(ctx, state, err);
        }
    }
}

fn toggle_highlight_current_char(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.highlight_current_char = !state.highlight_current_char;
    if let Err(err) = Settings::set_highlight_current_char(state.highlight_current_char) {
        error_log_add(ctx, state, err);
    }
}

fn set_editor_color(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(editor_color) = command_editor_color_argument(&args.argument) {
        state.editor_color = editor_color;
        if let Err(err) = Settings::set_editor_color(state.editor_color) {
            error_log_add(ctx, state, err);
        }
    }
}
