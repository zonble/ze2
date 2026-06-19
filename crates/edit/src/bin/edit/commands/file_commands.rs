// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::tui::Context;

use super::arguments::command_path_argument;
use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
use crate::localization::LocId;
use crate::settings::Settings;
use crate::state::*;

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::NewFile,
        names: &["new", "file-new"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::FileNew),
        default_focus_target: CommandFocusTarget::Default,
        handler: new_file,
    },
    CommandDefinition {
        command: Command::OpenFile,
        names: &["open", "file-open", "e", "edit"],
        namesVim: &["o"],
        namesEmacs: &["find-file"],
        loc_id: Some(LocId::FileOpen),
        default_focus_target: CommandFocusTarget::Default,
        handler: open_file,
    },
    CommandDefinition {
        command: Command::SaveAndCloseFileAndExitIfLast,
        names: &["file"],
        namesVim: &["wq"],
        namesEmacs: &["save-buffers-kill-emacs"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: save_and_close_file_and_exit_if_last,
    },
    CommandDefinition {
        command: Command::Save,
        names: &["save", "file-save"],
        namesVim: &["w"],
        namesEmacs: &["save-buffer"],
        loc_id: Some(LocId::FileSave),
        default_focus_target: CommandFocusTarget::Default,
        handler: save,
    },
    CommandDefinition {
        command: Command::SaveAs,
        names: &["save-as", "file-save-as"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::FileSaveAs),
        default_focus_target: CommandFocusTarget::Default,
        handler: save_as,
    },
    CommandDefinition {
        command: Command::Preferences,
        names: &["preferences", "settings"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::FilePreferences),
        default_focus_target: CommandFocusTarget::Default,
        handler: preferences,
    },
    CommandDefinition {
        command: Command::CloseFile,
        names: &["close", "file-close"],
        namesVim: &["q", "bd", "bdelete"],
        namesEmacs: &["kill-buffer"],
        loc_id: Some(LocId::FileClose),
        default_focus_target: CommandFocusTarget::Default,
        handler: close_file,
    },
    CommandDefinition {
        command: Command::Exit,
        names: &["exit"],
        namesVim: &["q!", "qa!"],
        namesEmacs: &["kill-emacs"],
        loc_id: Some(LocId::FileExit),
        default_focus_target: CommandFocusTarget::Default,
        handler: exit,
    },
    CommandDefinition {
        command: Command::CloseFileAndExitIfLast,
        names: &["quit"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: close_file_and_exit_if_last,
    },
];

fn new_file(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    draw_add_untitled_document(ctx, state);
}

fn open_file(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(path) = command_path_argument(&args.argument) {
        match state.documents.add_file_path(&path) {
            Ok(_) => {}
            Err(err) => error_log_add(ctx, state, err),
        }
    } else {
        state.wants_file_picker = StateFilePicker::Open;
    }
}

fn save(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(path) = command_path_argument(&args.argument) {
        if let Some(doc) = state.documents.active_mut()
            && let Err(err) = doc.save(Some(path))
        {
            error_log_add(ctx, state, err);
        }
    } else {
        state.wants_save = true;
    }
}

fn save_as(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.wants_file_picker = StateFilePicker::SaveAs;
}

fn save_and_close_file_and_exit_if_last(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let mut save_succeeded = false;
    if let Some(doc) = state.documents.active()
        && !doc.buffer.borrow().is_dirty()
    {
        state.wants_close = true;
        state.wants_exit_after_close = true;
    } else if let Some(path) = command_path_argument(&args.argument) {
        if let Some(doc) = state.documents.active_mut() {
            match doc.save(Some(path)) {
                Ok(()) => save_succeeded = true,
                Err(err) => error_log_add(ctx, state, err),
            }
        }
    } else if let Some(doc) = state.documents.active_mut() {
        if doc.path.is_some() {
            match doc.save(None) {
                Ok(()) => save_succeeded = true,
                Err(err) => error_log_add(ctx, state, err),
            }
        } else {
            state.wants_file_picker = StateFilePicker::SaveAs;
            state.wants_close_after_save = true;
            state.wants_exit_after_close = true;
        }
    } else {
        state.wants_exit = true;
    }

    if save_succeeded {
        state.wants_close = true;
        state.wants_exit_after_close = true;
    }
}

fn preferences(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    let settings = Settings::borrow();
    let path = settings.path.as_path();
    if !path.as_os_str().is_empty() {
        match state.documents.add_file_path(path) {
            Ok(doc) => {
                if let mut tb = doc.buffer.borrow_mut()
                    && tb.text_length() == 0
                {
                    Settings::bootstrap(&mut tb);
                }
            }
            Err(err) => error_log_add(ctx, state, err),
        }
    }
}

fn close_file(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.wants_close = true;
}

fn close_file_and_exit_if_last(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if state.documents.active().is_some() {
        state.wants_close = true;
        state.wants_exit_after_close = true;
    } else {
        state.wants_exit = true;
    }
}

fn exit(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.wants_exit = true;
}
