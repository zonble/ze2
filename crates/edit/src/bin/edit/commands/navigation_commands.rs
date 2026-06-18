// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::tui::Context;

use super::arguments::{command_path_argument, command_string_argument};
use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
use crate::draw_editor::validate_goto_point;
use crate::localization::LocId;
use crate::state::*;

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::FocusStatusbar,
        names: &["statusbar", "focus-statusbar"],
        loc_id: Some(LocId::ViewFocusStatusbar),
        default_focus_target: CommandFocusTarget::StatusBar,
        handler: focus_statusbar,
    },
    CommandDefinition {
        command: Command::GoToFile,
        names: &["go-to-file", "file-list"],
        loc_id: Some(LocId::ViewGoToFile),
        default_focus_target: CommandFocusTarget::Default,
        handler: go_to_file,
    },
    CommandDefinition {
        command: Command::Goto,
        names: &["goto", "go-to-line", "go-to-line-column"],
        loc_id: Some(LocId::FileGoto),
        default_focus_target: CommandFocusTarget::Default,
        handler: goto,
    },
    CommandDefinition {
        command: Command::Menu,
        names: &["menu"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: menu,
    },
];

fn focus_statusbar(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    state.wants_statusbar_focus = args.focus_target == CommandFocusTarget::StatusBar
        || args.focus_target == CommandFocusTarget::Default;
    if state.wants_statusbar_focus {
        state.wants_editor_focus = false;
    }
}

fn go_to_file(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(file) = command_string_argument(&args.argument) {
        let path = command_path_argument(&Some(file.clone())).unwrap();

        if !state.documents.update_active(|doc| {
            doc.filename == file
                || doc.path.as_ref().is_some_and(|doc_path| {
                    doc_path == &path
                        || doc_path.to_string_lossy() == file
                        || doc_path.to_string_lossy().ends_with(&file)
                })
        }) {
            match state.documents.add_file_path(&path) {
                Ok(_) => {}
                Err(err) => error_log_add(ctx, state, err),
            }
        }
    } else {
        state.wants_go_to_file = true;
    }
}

fn goto(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(line) = command_string_argument(&args.argument) {
        match validate_goto_point(&line) {
            Ok(point) => {
                if let Some(doc) = state.documents.active() {
                    let mut buf = doc.buffer.borrow_mut();
                    buf.cursor_move_to_logical(point);
                    buf.make_cursor_visible();
                }
            }
            Err(_) => {
                state.goto_target = line;
                state.goto_invalid = true;
                state.wants_goto = true;
            }
        }
    } else {
        state.wants_goto = true;
    }
}

fn menu(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.wants_menubar_focus = true;
    state.wants_editor_focus = false;
}
