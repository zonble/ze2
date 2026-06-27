// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::tui::Context;

use super::arguments::{
    command_bool_argument, command_editor_color_argument, command_eof_style_argument,
};
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
        command: Command::SetWordWrapColumn,
        names: &["set-word-wrap-column", "set-wrap-column", "right-margin", "rg"],
        namesVim: &["set-textwidth"],
        namesEmacs: &["set-fill-column"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_word_wrap_column,
        argument_hint: Some("<column>"),
    },
    CommandDefinition {
        command: Command::SetMargins,
        names: &["set-margins", "margins"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_margins,
        argument_hint: Some("<left> <right> <paragraph>"),
    },
    CommandDefinition {
        command: Command::SetTabs,
        names: &["set-tabs", "tabs"],
        namesVim: &["set-tabstop"],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_tabs,
        argument_hint: Some("<width>"),
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
        command: Command::SetHighlightCurrentChar,
        names: &["set-highlight-current-char"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_highlight_current_char,
        argument_hint: Some("<bool>"),
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
    CommandDefinition {
        command: Command::SetEditorColor,
        names: &["set-editor-color"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_editor_color,
        argument_hint: Some("original|white-on-blue"),
    },
    CommandDefinition {
        command: Command::SetEofStyle,
        names: &["set-eof-style"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_eof_style,
        argument_hint: Some("original|classic|ks3"),
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
    state.reflow_right_margin = col;

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

fn set_margins(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let Some((left, right, paragraph)) = parse_margins(args.argument.as_deref()) else {
        return;
    };
    set_word_wrap_column(
        ctx,
        state,
        CommandArgs {
            argument: Some(right.to_string()),
            focus_target: CommandFocusTarget::Default,
        },
    );
    state.reflow_left_margin = left.max(0);
    // set_word_wrap_column already stored the right margin, clamped to the same value
    // as the live word-wrap column. Overwriting it with the raw `right` here made
    // `reflow` use a different width than the display wrapped at.
    state.reflow_paragraph_margin = paragraph.max(0);
}

fn parse_margins(argument: Option<&str>) -> Option<(isize, isize, isize)> {
    let nums =
        argument?.split_whitespace().map(str::parse).collect::<Result<Vec<isize>, _>>().ok()?;
    Some(match nums.as_slice() {
        [right] => (0, *right, 0),
        [left, right] => (*left, *right, *left),
        [left, right, paragraph, ..] => (*left, *right, *paragraph),
        [] => return None,
    })
}

fn set_tabs(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let Some(width) = args
        .argument
        .as_deref()
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<isize>().ok())
    else {
        return;
    };
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().set_tab_size(width);
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

fn set_eof_style(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(eof_style) = command_eof_style_argument(&args.argument) {
        state.eof_style = eof_style;
        if let Err(err) = Settings::set_eof_style(state.eof_style) {
            error_log_add(ctx, state, err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_margins;

    #[test]
    fn set_margins_parses_advertised_arguments() {
        assert_eq!(parse_margins(Some("72")), Some((0, 72, 0)));
        assert_eq!(parse_margins(Some("5 72")), Some((5, 72, 5)));
        assert_eq!(parse_margins(Some("5 72 3")), Some((5, 72, 3)));
    }

    #[test]
    fn set_margins_truncates_extra_and_rejects_junk() {
        // 4th+ values are dropped, not treated as an error.
        assert_eq!(parse_margins(Some("1 2 3 4")), Some((1, 2, 3)));
        // Missing/empty/non-numeric input is rejected so the command no-ops.
        assert_eq!(parse_margins(None), None);
        assert_eq!(parse_margins(Some("")), None);
        assert_eq!(parse_margins(Some("a b")), None);
    }
}
