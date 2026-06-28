// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::buffer::{CursorMovement, TextMarkKind};
use ze2::tui::Context;

use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
use crate::localization::{LocId, loc};
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
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Redo,
        names: &["redo"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::EditRedo),
        default_focus_target: CommandFocusTarget::Default,
        handler: redo,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Cut,
        names: &["cut"],
        namesVim: &["delete"],
        namesEmacs: &["kill-region"],
        loc_id: Some(LocId::EditCut),
        default_focus_target: CommandFocusTarget::Default,
        handler: cut,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Copy,
        names: &["copy"],
        namesVim: &["yank"],
        namesEmacs: &["kill-ring-save"],
        loc_id: Some(LocId::EditCopy),
        default_focus_target: CommandFocusTarget::Default,
        handler: copy,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Paste,
        names: &["paste"],
        namesVim: &["put"],
        namesEmacs: &["clipboard-yank"],
        loc_id: Some(LocId::EditPaste),
        default_focus_target: CommandFocusTarget::Default,
        handler: paste,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectAll,
        names: &["select-all"],
        namesVim: &[],
        namesEmacs: &["mark-whole-buffer"],
        loc_id: Some(LocId::EditSelectAll),
        default_focus_target: CommandFocusTarget::Default,
        handler: select_all,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectLine,
        names: &["select-line", "line"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::InsertText,
        names: &["insert"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: insert_text,
        argument_hint: Some("<text>"),
    },
    CommandDefinition {
        command: Command::InsertLine,
        names: &["insert-line", "il"],
        namesVim: &[],
        namesEmacs: &["open-line"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: insert_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SplitLine,
        names: &["split-line", "split", "sp"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: split_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::JoinLine,
        names: &["join-line", "join", "jo"],
        namesVim: &["join"],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: join_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::FirstNonblank,
        names: &["first-nonblank", "fn"],
        namesVim: &[],
        namesEmacs: &["back-to-indentation"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: first_nonblank,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::BeginWord,
        names: &["begin-word", "wb"],
        namesVim: &[],
        namesEmacs: &["backward-word"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: begin_word,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::EndWord,
        names: &["end-word", "we"],
        namesVim: &[],
        namesEmacs: &["forward-word"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: end_word,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::TabWord,
        names: &["tab-word", "tw"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: tab_word,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::BacktabWord,
        names: &["backtab-word", "bw"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: backtab_word,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MarkLine,
        names: &["mark-line", "ml"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: mark_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MarkChar,
        names: &["mark-char", "mc"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: mark_char,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MarkBlock,
        names: &["mark-block", "mb"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: mark_block,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Unmark,
        names: &["unmark", "um"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: unmark,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::CopyMark,
        names: &["copy-mark", "cm"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: copy_mark,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveMark,
        names: &["move-mark", "mm"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: move_mark,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::DeleteMark,
        names: &["delete-mark", "dm"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: delete_mark,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::FillMark,
        names: &["fill-mark", "fm"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: fill_mark,
        argument_hint: Some("<char>"),
    },
    CommandDefinition {
        command: Command::OverlayBlock,
        names: &["overlay-block", "ob"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: overlay_block,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::ShiftLeft,
        names: &["shift-left", "sl"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: shift_left,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::ShiftRight,
        names: &["shift-right", "sr"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: shift_right,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::CopyToCmd,
        names: &["copy-to-command", "ct"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: copy_to_command,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::CopyFromCmd,
        names: &["copy-from-command", "cf"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: copy_from_command,
        argument_hint: None,
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

fn insert_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().insert_line_below();
    }
}

fn split_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().split_line();
    }
}

fn join_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().join_next_line();
    }
}

fn first_nonblank(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_to_first_nonblank();
    }
}

fn begin_word(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_to_begin_word();
    }
}

fn end_word(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_to_end_word();
    }
}

fn tab_word(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_delta(CursorMovement::Word, 1);
    }
}

fn backtab_word(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_delta(CursorMovement::Word, -1);
    }
}

fn mark_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().mark(TextMarkKind::Line);
    }
}

fn mark_char(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().mark(TextMarkKind::Char);
    }
}

fn mark_block(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let mut tb = doc.buffer.borrow_mut();
        if tb.is_word_wrap_enabled() {
            state.command_bar_error = loc(LocId::CommandMarkBlockWordWrapEnabled).to_string();
            return;
        }
        tb.mark(TextMarkKind::Block);
    }
}

fn unmark(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().clear_mark();
    }
}

fn copy_mark(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().copy_mark_to_cursor();
    }
}

fn move_mark(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().move_mark_to_cursor();
    }
}

fn delete_mark(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().delete_mark(ctx.clipboard_mut());
    }
}

fn fill_mark(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(doc) = state.documents.active()
        && let Some(text) = args.argument
    {
        doc.buffer.borrow_mut().fill_mark(text.as_bytes());
    }
}

fn overlay_block(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().overlay_block_from_clipboard(ctx.clipboard_ref());
    }
}

fn shift_left(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().shift_block_mark(false);
    }
}

fn shift_right(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().shift_block_mark(true);
    }
}

fn copy_to_command(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let line = doc.buffer.borrow().current_line_text();
        state.command_bar_input = String::from_utf8_lossy(&line).into_owned();
        state.command_bar_active = true;
        state.command_bar_focus = true;
        state.wants_editor_focus = false;
    }
}

fn copy_from_command(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().write_canon(state.command_bar_input.as_bytes());
    }
}
