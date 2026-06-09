// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::input::{InputKey, InputKeyMod, kbmod, vk};
use edit::path;
use edit::tui::Context;
use std::env;
use std::path::{Path, PathBuf};

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
    SelectLine,
    InsertText,
    FocusStatusbar,
    GoToFile,
    Goto,
    WordWrap,
    About,
}

pub struct CommandInvocation {
    pub command: Command,
    pub argument: Option<String>,
}

pub struct CommandBarShortcut {
    pub text: &'static str,
}

pub fn execute_command(ctx: &mut Context, state: &mut State, command: Command) {
    execute_command_invocation(ctx, state, CommandInvocation { command, argument: None });
}

pub fn execute_command_invocation(
    ctx: &mut Context,
    state: &mut State,
    invocation: CommandInvocation,
) {
    let CommandInvocation { command, argument } = invocation;
    match command {
        Command::NewFile => draw_add_untitled_document(ctx, state),
        Command::OpenFile => {
            if let Some(path) = command_path_argument(&argument) {
                match state.documents.add_file_path(&path) {
                    Ok(_) => {}
                    Err(err) => error_log_add(ctx, state, err),
                }
            } else {
                state.wants_file_picker = StateFilePicker::Open;
            }
        }
        Command::Save => {
            if let Some(path) = command_path_argument(&argument) {
                if let Some(doc) = state.documents.active_mut()
                    && let Err(err) = doc.save(Some(path))
                {
                    error_log_add(ctx, state, err);
                }
            } else {
                state.wants_save = true;
            }
        }
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
        Command::SelectLine => {
            if let Some(doc) = state.documents.active() {
                doc.buffer.borrow_mut().select_line();
            }
        }
        Command::InsertText => {
            if let Some(text) = argument
                && let Some(doc) = state.documents.active()
            {
                doc.buffer.borrow_mut().write_canon(text.as_bytes());
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

fn command_path_argument(argument: &Option<String>) -> Option<PathBuf> {
    let argument = argument.as_deref()?.trim();
    if argument.is_empty() {
        return None;
    }

    let path = Path::new(argument);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().unwrap_or_default().join(path)
    };
    Some(path::normalize(&path))
}

pub fn command_invocation_from_shortcut(key: InputKey) -> Option<CommandInvocation> {
    if let Some(text) = text_from_insert_shortcut(key) {
        return Some(CommandInvocation {
            command: Command::InsertText,
            argument: Some(text.to_string()),
        });
    }

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
        k if k == kbmod::CTRL | vk::L => Command::SelectLine,
        _ => return None,
    })
    .map(|command| CommandInvocation { command, argument: None })
}

pub fn commandbar_shortcut_from_key(key: InputKey) -> Option<CommandBarShortcut> {
    Some(CommandBarShortcut {
        text: match key {
            k if k == vk::F2 => "save ",
            k if k == vk::F3 => "file ",
            k if k == vk::F4 => "quit",
            _ => return None,
        },
    })
}

pub fn should_handle_command_shortcut_before_editor(command: Command) -> bool {
    matches!(command, Command::InsertText)
}

fn text_from_insert_shortcut(key: InputKey) -> Option<&'static str> {
    INSERT_SHORTCUTS
        .iter()
        .find(|shortcut| shortcut.modifiers | shortcut.key == key)
        .map(|shortcut| shortcut.text)
}

pub fn command_from_text(text: &str) -> Option<CommandInvocation> {
    let normalized = normalize_command_name(text);
    if normalized.is_empty() {
        return None;
    }

    for definition in COMMANDS {
        if definition.names.iter().any(|name| normalized == normalize_command_name(name)) {
            return Some(CommandInvocation { command: definition.command, argument: None });
        }

        if definition.loc_id.is_some_and(|loc_id| normalized == normalize_command_name(loc(loc_id)))
        {
            return Some(CommandInvocation { command: definition.command, argument: None });
        }
    }

    command_from_text_with_argument(text)
}

fn command_from_text_with_argument(text: &str) -> Option<CommandInvocation> {
    let text = text.trim();
    let (name, argument) = text.split_once(char::is_whitespace)?;
    let normalized = normalize_command_name(name);
    if normalized.is_empty() {
        return None;
    }

    for definition in COMMANDS {
        if definition.names.iter().any(|name| normalized == normalize_command_name(name))
            || definition
                .loc_id
                .is_some_and(|loc_id| normalized == normalize_command_name(loc(loc_id)))
        {
            return Some(CommandInvocation {
                command: definition.command,
                argument: Some(argument.trim().to_string()),
            });
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
        assert!(matches!(
            command_from_text("save as"),
            Some(CommandInvocation { command: Command::SaveAs, argument: None })
        ));
        assert!(matches!(
            command_from_text("select-all"),
            Some(CommandInvocation { command: Command::SelectAll, argument: None })
        ));
        assert!(matches!(
            command_from_text("Go to Line:Column..."),
            Some(CommandInvocation { command: Command::Goto, argument: None })
        ));
    }

    #[test]
    fn command_text_preserves_argument_tail() {
        let Some(CommandInvocation { command: Command::OpenFile, argument: Some(argument) }) =
            command_from_text("open path with spaces.txt")
        else {
            panic!("open command did not parse");
        };

        assert!(argument == "path with spaces.txt");
    }

    #[test]
    fn insert_shortcuts_map_to_text_invocations() {
        for (key, expected) in [
            (kbmod::ALT | vk::COMMA, "，"),
            (kbmod::ALT | vk::PERIOD, "。"),
            (kbmod::ALT | vk::DELETE, "。"),
            (kbmod::ALT | vk::SEMICOLON, "；"),
            (kbmod::ALT | vk::COLON, "："),
            (kbmod::ALT | vk::APOSTROPHE, "、"),
            (kbmod::ALT | vk::LBRACKET, "「"),
            (kbmod::ALT | vk::RBRACKET, "」"),
            (kbmod::ALT | vk::LBRACE, "『"),
            (kbmod::ALT | vk::RBRACE, "』"),
            (kbmod::ALT | vk::N1, "！"),
            (kbmod::ALT | vk::EXCLAMATION, "！"),
        ] {
            let Some(CommandInvocation { command: Command::InsertText, argument: Some(text) }) =
                command_invocation_from_shortcut(key)
            else {
                panic!("insert shortcut did not parse");
            };

            assert!(text == expected);
        }
    }
}

struct CommandDefinition {
    command: Command,
    names: &'static [&'static str],
    loc_id: Option<LocId>,
}

struct InsertShortcut {
    modifiers: InputKeyMod,
    key: InputKey,
    text: &'static str,
}

const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::NewFile,
        names: &["new", "new-file", "file-new"],
        loc_id: Some(LocId::FileNew),
    },
    CommandDefinition {
        command: Command::OpenFile,
        names: &["open", "file", "open-file", "file-open"],
        loc_id: Some(LocId::FileOpen),
    },
    CommandDefinition {
        command: Command::Save,
        names: &["save", "file-save"],
        loc_id: Some(LocId::FileSave),
    },
    CommandDefinition {
        command: Command::SaveAs,
        names: &["save-as", "file-save-as"],
        loc_id: Some(LocId::FileSaveAs),
    },
    CommandDefinition {
        command: Command::Preferences,
        names: &["preferences", "settings"],
        loc_id: Some(LocId::FilePreferences),
    },
    CommandDefinition {
        command: Command::CloseFile,
        names: &["close", "close-file", "file-close"],
        loc_id: Some(LocId::FileClose),
    },
    CommandDefinition {
        command: Command::Exit,
        names: &["exit", "quit"],
        loc_id: Some(LocId::FileExit),
    },
    CommandDefinition { command: Command::Undo, names: &["undo"], loc_id: Some(LocId::EditUndo) },
    CommandDefinition { command: Command::Redo, names: &["redo"], loc_id: Some(LocId::EditRedo) },
    CommandDefinition { command: Command::Cut, names: &["cut"], loc_id: Some(LocId::EditCut) },
    CommandDefinition { command: Command::Copy, names: &["copy"], loc_id: Some(LocId::EditCopy) },
    CommandDefinition {
        command: Command::Paste,
        names: &["paste"],
        loc_id: Some(LocId::EditPaste),
    },
    CommandDefinition {
        command: Command::Find,
        names: &["find", "search"],
        loc_id: Some(LocId::EditFind),
    },
    CommandDefinition {
        command: Command::Replace,
        names: &["replace"],
        loc_id: Some(LocId::EditReplace),
    },
    CommandDefinition {
        command: Command::SelectAll,
        names: &["select-all"],
        loc_id: Some(LocId::EditSelectAll),
    },
    CommandDefinition {
        command: Command::SelectLine,
        names: &["select-line", "line"],
        loc_id: None,
    },
    CommandDefinition {
        command: Command::InsertText,
        names: &["insert", "type", "text"],
        loc_id: None,
    },
    CommandDefinition {
        command: Command::FocusStatusbar,
        names: &["statusbar", "focus-statusbar"],
        loc_id: Some(LocId::ViewFocusStatusbar),
    },
    CommandDefinition {
        command: Command::GoToFile,
        names: &["go-to-file", "file-list"],
        loc_id: Some(LocId::ViewGoToFile),
    },
    CommandDefinition {
        command: Command::Goto,
        names: &["goto", "go-to-line", "go-to-line-column"],
        loc_id: Some(LocId::FileGoto),
    },
    CommandDefinition {
        command: Command::WordWrap,
        names: &["word-wrap", "wrap"],
        loc_id: Some(LocId::ViewWordWrap),
    },
    CommandDefinition {
        command: Command::About,
        names: &["about"],
        loc_id: Some(LocId::HelpAbout),
    },
];

const INSERT_SHORTCUTS: &[InsertShortcut] = &[
    InsertShortcut { modifiers: kbmod::ALT, key: vk::COMMA, text: "，" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::PERIOD, text: "。" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::DELETE, text: "。" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::SEMICOLON, text: "；" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::COLON, text: "：" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::SEMICOLON, text: "：" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::APOSTROPHE, text: "、" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::LBRACKET, text: "「" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::RBRACKET, text: "」" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::LBRACE, text: "『" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::LBRACKET, text: "『" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::RBRACE, text: "』" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::RBRACKET, text: "』" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N1, text: "！" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::EXCLAMATION, text: "！" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::N1, text: "！" },
];
