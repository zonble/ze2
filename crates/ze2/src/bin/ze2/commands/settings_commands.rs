// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::tui::Context;

use super::arguments::{
    command_bool_argument, command_editor_color_argument, command_encoding_argument,
    command_eof_style_argument, command_line_break_argument,
};
use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
use crate::settings::{BindingMode, Settings};
use crate::state::*;

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::SetEncoding,
        names: &["set-encoding", "encoding"],
        namesVim: &["set-fileencoding", "set-fenc"],
        namesEmacs: &["set-buffer-file-coding-system"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_encoding,
        argument_hint: Some("<encoding>"),
    },
    CommandDefinition {
        command: Command::SetLineBreak,
        names: &["set-line-break", "set-line-break-char", "set-newline", "newline"],
        namesVim: &["set-fileformat", "set-ff"],
        namesEmacs: &["set-buffer-file-eol-type"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_line_break,
        argument_hint: Some("LF|CRLF"),
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
        argument_hint: Some("original|classic|ks3|hidden"),
    },
    CommandDefinition {
        command: Command::EnableVimCommands,
        names: &["set-vim-commands-enabled"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: enable_vim_commands,
        argument_hint: Some("<bool>"),
    },
    CommandDefinition {
        command: Command::EnableEmacsCommands,
        names: &["set-emacs-commands-enabled"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: enable_emacs_commands,
        argument_hint: Some("<bool>"),
    },
    CommandDefinition {
        command: Command::SetBinding,
        names: &["set-binding"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: set_binding,
        argument_hint: Some("original|ghostty"),
    },
];

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

fn set_highlight_current_char(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let enabled = command_bool_argument(&args.argument).unwrap_or(true);
    state.highlight_current_char = enabled;
    if let Err(err) = Settings::set_highlight_current_char(enabled) {
        error_log_add(ctx, state, err);
    }
}

fn set_editor_color(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(color) = command_editor_color_argument(&args.argument) {
        state.editor_color = color;
        if let Err(err) = Settings::set_editor_color(color) {
            error_log_add(ctx, state, err);
        }
    }
}

fn set_eof_style(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(style) = command_eof_style_argument(&args.argument) {
        state.eof_style = style;
        if let Err(err) = Settings::set_eof_style(style) {
            error_log_add(ctx, state, err);
        }
    }
}

fn enable_vim_commands(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let enabled = command_bool_argument(&args.argument).unwrap_or(true);
    state.command_bar_include_vim_commands = enabled;
    if let Err(err) = Settings::set_command_bar_include_vim_commands(enabled) {
        error_log_add(ctx, state, err);
    }
}

fn enable_emacs_commands(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let enabled = command_bool_argument(&args.argument).unwrap_or(true);
    state.command_bar_include_emacs_commands = enabled;
    if let Err(err) = Settings::set_command_bar_include_emacs_commands(enabled) {
        error_log_add(ctx, state, err);
    }
}

fn set_binding(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(mode) = args.argument.as_deref() {
        let binding_mode = match mode.trim() {
            "ghostty" => BindingMode::Ghostty,
            "original" => BindingMode::Original,
            _ => return,
        };
        state.binding = binding_mode;
        if let Err(err) = Settings::set_binding(binding_mode) {
            error_log_add(ctx, state, err);
        }
    }
}

fn set_encoding(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(encoding) = command_encoding_argument(&args.argument) {
        if let Some(doc) = state.documents.active_mut() {
            doc.buffer.borrow_mut().set_encoding(encoding);
        }
    } else if state.documents.active().is_some() {
        state.wants_encoding_change = StateEncodingChange::Convert;
    }
}

fn set_line_break(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(crlf) = command_line_break_argument(&args.argument)
        && let Some(doc) = state.documents.active()
    {
        doc.buffer.borrow_mut().normalize_newlines(crlf);
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
