// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::tui::Context;

use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
use crate::localization::LocId;
use crate::state::State;

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::Undo,
        names: &["undo"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::EditUndo),
        default_focus_target: CommandFocusTarget::Default,
        handler: undo,
    },
    CommandDefinition {
        command: Command::Redo,
        names: &["redo"],
        namesVim: &[],
        namesEmacs: &[""],
        loc_id: Some(LocId::EditRedo),
        default_focus_target: CommandFocusTarget::Default,
        handler: redo,
    },
    CommandDefinition {
        command: Command::Cut,
        names: &["cut"],
        namesVim: &["delete"],
        namesEmacs: &["kill-region"],
        loc_id: Some(LocId::EditCut),
        default_focus_target: CommandFocusTarget::Default,
        handler: cut,
    },
    CommandDefinition {
        command: Command::Copy,
        names: &["copy"],
        namesVim: &["yank"],
        namesEmacs: &["kill-ring-save"],
        loc_id: Some(LocId::EditCopy),
        default_focus_target: CommandFocusTarget::Default,
        handler: copy,
    },
    CommandDefinition {
        command: Command::Paste,
        names: &["paste"],
        namesVim: &["put"],
        namesEmacs: &["clipboard-yank"],
        loc_id: Some(LocId::EditPaste),
        default_focus_target: CommandFocusTarget::Default,
        handler: paste,
    },
    CommandDefinition {
        command: Command::SelectAll,
        names: &["select-all"],
        namesVim: &[],
        namesEmacs: &["mark-whole-buffer"],
        loc_id: Some(LocId::EditSelectAll),
        default_focus_target: CommandFocusTarget::Default,
        handler: select_all,
    },
    CommandDefinition {
        command: Command::SelectLine,
        names: &["select-line", "line"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_line,
    },
    CommandDefinition {
        command: Command::InsertText,
        names: &["insert"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: insert_text,
    },
];

fn undo(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().undo();
    }
}

fn redo(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().redo();
    }
}

fn cut(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cut(ctx.clipboard_mut());
    }
}

fn copy(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().copy(ctx.clipboard_mut());
    }
}

fn paste(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().paste(ctx.clipboard_ref(), false);
    }
}

fn select_all(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().select_all();
    }
}

fn select_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().select_line();
    }
}

fn insert_text(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(text) = args.argument
        && let Some(doc) = state.documents.active()
    {
        doc.buffer.borrow_mut().write_canon_smart(text.as_bytes());
    }
}
