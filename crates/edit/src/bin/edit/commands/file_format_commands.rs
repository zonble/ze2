// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::tui::Context;

use super::arguments::{command_encoding_argument, command_line_break_argument};
use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
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
        command: Command::ReopenEncoding,
        names: &["reopen-encoding"],
        namesVim: &["edit-encoding"],
        namesEmacs: &["revert-buffer-with-coding-system"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: reopen_encoding,
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
];

fn set_encoding(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(encoding) = command_encoding_argument(&args.argument) {
        if let Some(doc) = state.documents.active_mut() {
            doc.buffer.borrow_mut().set_encoding(encoding);
        }
    } else if state.documents.active().is_some() {
        state.wants_encoding_change = StateEncodingChange::Convert;
    }
}

fn reopen_encoding(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(encoding) = command_encoding_argument(&args.argument) {
        if let Some(doc) = state.documents.active_mut()
            && doc.path.is_some()
        {
            let mut res = Ok(());
            if doc.buffer.borrow().is_dirty() {
                res = doc.save(None);
            }
            if res.is_ok() {
                res = doc.reread(Some(encoding));
            }
            if let Err(err) = res {
                error_log_add(ctx, state, err);
            }
        }
    } else if state.documents.active().is_some() {
        state.wants_encoding_change = StateEncodingChange::Reopen;
    }
}

fn set_line_break(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(crlf) = command_line_break_argument(&args.argument)
        && let Some(doc) = state.documents.active()
    {
        doc.buffer.borrow_mut().normalize_newlines(crlf);
    }
}
