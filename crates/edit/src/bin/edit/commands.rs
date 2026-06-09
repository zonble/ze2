// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::input::{InputKey, kbmod, vk};
use edit::tui::Context;

use crate::localization::*;
use crate::settings::Settings;
use crate::state::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Command {
    NewFile,
    OpenFile,
    Save,
    SaveAs,
    Preferences,
    CloseFile,
    Exit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Find,
    Replace,
    SelectAll,
    FocusStatusbar,
    GoToFile,
    Goto,
    WordWrap,
    About,
}

pub fn execute_command(ctx: &mut Context, state: &mut State, command: Command) {
    match command {
        Command::NewFile => draw_add_untitled_document(ctx, state),
        Command::OpenFile => state.wants_file_picker = StateFilePicker::Open,
        Command::Save => state.wants_save = true,
        Command::SaveAs => state.wants_file_picker = StateFilePicker::SaveAs,
        Command::Preferences => {
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
        Command::CloseFile => state.wants_close = true,
        Command::Exit => state.wants_exit = true,
        Command::Undo => {
            if let Some(doc) = state.documents.active() {
                doc.buffer.borrow_mut().undo();
            }
        }
        Command::Redo => {
            if let Some(doc) = state.documents.active() {
                doc.buffer.borrow_mut().redo();
            }
        }
        Command::Cut => {
            if let Some(doc) = state.documents.active() {
                doc.buffer.borrow_mut().cut(ctx.clipboard_mut());
            }
        }
        Command::Copy => {
            if let Some(doc) = state.documents.active() {
                doc.buffer.borrow_mut().copy(ctx.clipboard_mut());
            }
        }
        Command::Paste => {
            if let Some(doc) = state.documents.active() {
                doc.buffer.borrow_mut().paste(ctx.clipboard_ref(), false);
            }
        }
        Command::Find => {
            if state.wants_search.kind != StateSearchKind::Disabled {
                state.wants_search.kind = StateSearchKind::Search;
                state.wants_search.focus = true;
            }
        }
        Command::Replace => {
            if state.wants_search.kind != StateSearchKind::Disabled {
                state.wants_search.kind = StateSearchKind::Replace;
                state.wants_search.focus = true;
            }
        }
        Command::SelectAll => {
            if let Some(doc) = state.documents.active() {
                doc.buffer.borrow_mut().select_all();
            }
        }
        Command::FocusStatusbar => state.wants_statusbar_focus = true,
        Command::GoToFile => state.wants_go_to_file = true,
        Command::Goto => state.wants_goto = true,
        Command::WordWrap => {
            if let Some(doc) = state.documents.active() {
                let mut tb = doc.buffer.borrow_mut();
                let word_wrap = tb.is_word_wrap_enabled();
                tb.set_word_wrap(!word_wrap);
            }
        }
        Command::About => state.wants_about = true,
    }

    ctx.needs_rerender();
}

pub fn command_from_shortcut(key: InputKey) -> Option<Command> {
    Some(match key {
        k if k == kbmod::CTRL | vk::N => Command::NewFile,
        k if k == kbmod::CTRL | vk::O => Command::OpenFile,
        k if k == kbmod::CTRL | vk::S => Command::Save,
        k if k == kbmod::CTRL_SHIFT | vk::S => Command::SaveAs,
        k if k == kbmod::CTRL | vk::W => Command::CloseFile,
        k if k == kbmod::CTRL | vk::P => Command::GoToFile,
        k if k == kbmod::CTRL | vk::Q => Command::Exit,
        k if k == kbmod::CTRL | vk::G => Command::Goto,
        k if k == kbmod::CTRL | vk::F => Command::Find,
        k if k == kbmod::CTRL | vk::R => Command::Replace,
        _ => return None,
    })
}

pub fn command_from_text(text: &str) -> Option<Command> {
    let normalized = normalize_command_name(text);
    if normalized.is_empty() {
        return None;
    }

    for (command, names, loc_id) in COMMANDS {
        if names.iter().any(|name| normalized == normalize_command_name(name)) {
            return Some(*command);
        }

        if normalized == normalize_command_name(loc(*loc_id)) {
            return Some(*command);
        }
    }

    None
}

fn normalize_command_name(text: &str) -> String {
    let mut out = String::new();
    let mut last_was_separator = true;

    for ch in text.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            out.push('-');
            last_was_separator = true;
        }
    }

    if out.ends_with('-') {
        out.pop();
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_text_accepts_common_aliases() {
        assert!(command_from_text("save as") == Some(Command::SaveAs));
        assert!(command_from_text("select-all") == Some(Command::SelectAll));
        assert!(command_from_text("Go to Line:Column...") == Some(Command::Goto));
    }
}

const COMMANDS: &[(Command, &[&str], LocId)] = &[
    (Command::NewFile, &["new", "new-file", "file-new"], LocId::FileNew),
    (Command::OpenFile, &["open", "open-file", "file-open"], LocId::FileOpen),
    (Command::Save, &["save", "file-save"], LocId::FileSave),
    (Command::SaveAs, &["save-as", "file-save-as"], LocId::FileSaveAs),
    (Command::Preferences, &["preferences", "settings"], LocId::FilePreferences),
    (Command::CloseFile, &["close", "close-file", "file-close"], LocId::FileClose),
    (Command::Exit, &["exit", "quit"], LocId::FileExit),
    (Command::Undo, &["undo"], LocId::EditUndo),
    (Command::Redo, &["redo"], LocId::EditRedo),
    (Command::Cut, &["cut"], LocId::EditCut),
    (Command::Copy, &["copy"], LocId::EditCopy),
    (Command::Paste, &["paste"], LocId::EditPaste),
    (Command::Find, &["find", "search"], LocId::EditFind),
    (Command::Replace, &["replace"], LocId::EditReplace),
    (Command::SelectAll, &["select-all"], LocId::EditSelectAll),
    (Command::FocusStatusbar, &["statusbar", "focus-statusbar"], LocId::ViewFocusStatusbar),
    (Command::GoToFile, &["go-to-file", "file-list"], LocId::ViewGoToFile),
    (Command::Goto, &["goto", "go-to-line", "go-to-line-column"], LocId::FileGoto),
    (Command::WordWrap, &["word-wrap", "wrap"], LocId::ViewWordWrap),
    (Command::About, &["about"], LocId::HelpAbout),
];
