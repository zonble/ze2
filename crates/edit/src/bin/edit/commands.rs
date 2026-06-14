// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::icu;
use edit::input::{InputKey, InputKeyMod, kbmod, vk};
use edit::path;
use edit::tui::Context;
use std::env;
use std::path::{Path, PathBuf};

use crate::draw_editor::{SearchAction, search_execute, validate_goto_point};
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
    WordCount,
    SaveAndCloseFile,
    CloseFileAndExitIfLast,
    SetWordWrapColumn,
    Menu,
    CenterText,
}

pub struct CommandInvocation {
    pub command: Command,
    pub argument: Option<String>,
    pub focus_target: CommandFocusTarget,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandFocusTarget {
    Default,
    SearchPanel,
}

pub struct CommandBarShortcut {
    pub text: &'static str,
}

pub fn execute_command(ctx: &mut Context, state: &mut State, command: Command) {
    execute_command_invocation(
        ctx,
        state,
        CommandInvocation { command, argument: None, focus_target: CommandFocusTarget::Default },
    );
}

pub fn execute_command_invocation(
    ctx: &mut Context,
    state: &mut State,
    invocation: CommandInvocation,
) {
    let CommandInvocation { command, argument, focus_target } = invocation;
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
        Command::SaveAndCloseFile => {
            let mut save_succeeded = false;
            if let Some(doc) = state.documents.active()
                && !doc.buffer.borrow().is_dirty()
            {
                state.wants_close = true;
                state.wants_exit_after_close = true;
            } else if let Some(path) = command_path_argument(&argument) {
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
        Command::CloseFileAndExitIfLast => {
            if state.documents.active().is_some() {
                state.wants_close = true;
                state.wants_exit_after_close = true;
            } else {
                state.wants_exit = true;
            }
        }
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
                if let Some(argument) = command_string_argument(&argument) {
                    state.search_needle = argument;
                    state.wants_search.focus = focus_target == CommandFocusTarget::SearchPanel;
                    if focus_target == CommandFocusTarget::SearchPanel {
                        state.wants_editor_focus = false;
                    }
                    if let Err(err) = icu::init() {
                        error_log_add(ctx, state, err.into());
                        state.wants_search.kind = StateSearchKind::Disabled;
                    } else {
                        search_execute(ctx, state, SearchAction::Search);
                    }
                } else {
                    state.wants_search.focus = true;
                    state.wants_editor_focus = false;
                }
            }
        }
        Command::Replace => {
            if state.wants_search.kind != StateSearchKind::Disabled {
                state.wants_search.kind = StateSearchKind::Replace;
                if let Some((needle, replacement)) = command_replace_arguments(&argument) {
                    state.search_needle = needle;
                    state.search_replacement = replacement;
                    state.wants_search.focus = focus_target == CommandFocusTarget::SearchPanel;
                    if focus_target == CommandFocusTarget::SearchPanel {
                        state.wants_editor_focus = false;
                    }
                    if let Err(err) = icu::init() {
                        error_log_add(ctx, state, err.into());
                        state.wants_search.kind = StateSearchKind::Disabled;
                    } else {
                        search_execute(ctx, state, SearchAction::Replace);
                    }
                } else {
                    state.wants_search.focus = true;
                    state.wants_editor_focus = false;
                }
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
                doc.buffer.borrow_mut().write_canon_smart(text.as_bytes());
            }
        }
        Command::FocusStatusbar => state.wants_statusbar_focus = true,
        Command::GoToFile => {
            if let Some(file) = command_string_argument(&argument) {
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
        Command::Goto => {
            if let Some(line) = command_string_argument(&argument) {
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
        Command::WordWrap => {
            if let Some(doc) = state.documents.active() {
                let mut tb = doc.buffer.borrow_mut();
                let word_wrap =
                    command_bool_argument(&argument).unwrap_or_else(|| !tb.is_word_wrap_enabled());
                tb.set_word_wrap(word_wrap);
                drop(tb);
                if let Err(err) = Settings::set_word_wrap(word_wrap) {
                    error_log_add(ctx, state, err);
                }
            }
        }
        Command::About => state.wants_about = true,
        Command::WordCount => state.wants_word_count = true,
        Command::SetWordWrapColumn => {
            let col = argument
                .as_deref()
                .and_then(|s| s.trim().parse::<isize>().ok())
                .unwrap_or(0)
                .max(0);
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
        Command::Menu => {
            state.wants_menubar_focus = true;
            state.wants_editor_focus = false;
        }
        Command::CenterText => {
            let center_text =
                command_bool_argument(&argument).unwrap_or(!state.wants_center_text);
            state.wants_center_text = center_text;
            if let Err(err) = Settings::set_center_text(center_text) {
                error_log_add(ctx, state, err);
            }
        }
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

fn command_string_argument(argument: &Option<String>) -> Option<String> {
    let argument = argument.as_deref()?.trim();
    (!argument.is_empty()).then(|| argument.to_string())
}

fn command_replace_arguments(argument: &Option<String>) -> Option<(String, String)> {
    let argument = argument.as_deref()?.trim();
    let (needle, replacement) = argument.split_once(char::is_whitespace)?;
    let needle = needle.trim();
    if needle.is_empty() {
        return None;
    }
    Some((needle.to_string(), replacement.trim().to_string()))
}

fn command_bool_argument(argument: &Option<String>) -> Option<bool> {
    match argument.as_deref()?.trim().to_ascii_lowercase().as_str() {
        "true" | "on" | "yes" | "1" => Some(true),
        "false" | "off" | "no" | "0" => Some(false),
        _ => None,
    }
}

pub fn command_invocation_from_shortcut(key: InputKey) -> Option<CommandInvocation> {
    if let Some(text) = text_from_insert_shortcut(key) {
        return Some(CommandInvocation {
            command: Command::InsertText,
            argument: Some(text.to_string()),
            focus_target: CommandFocusTarget::Default,
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
    .map(|command| CommandInvocation {
        command,
        argument: None,
        focus_target: CommandFocusTarget::Default,
    })
}

pub fn commandbar_shortcut_from_key(key: InputKey) -> Option<CommandBarShortcut> {
    Some(CommandBarShortcut {
        text: match key {
            k if k == vk::F2 => "save ",
            k if k == vk::F3 => "file ",
            k if k == vk::F4 => "quit ",
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

pub fn autocomplete_commands(prefix: &str) -> Vec<String> {
    let prefix = normalize_command_name(prefix);
    if prefix.is_empty() {
        return Vec::new();
    }

    let mut suggestions = Vec::new();

    for definition in COMMANDS {
        for name in definition.names {
            let norm_name = normalize_command_name(name);
            if norm_name.starts_with(&prefix) {
                suggestions.push(name.to_string());
            }
        }
    }

    suggestions.sort();
    suggestions.dedup();
    suggestions.truncate(10);
    suggestions
}

pub fn command_from_text(text: &str) -> Option<CommandInvocation> {
    if let Some(invocation) = command_from_shorthand(text) {
        return Some(invocation);
    }

    let normalized = normalize_command_name(text);
    if normalized.is_empty() {
        return None;
    }

    for definition in COMMANDS {
        if definition.names.iter().any(|name| normalized == normalize_command_name(name)) {
            return Some(CommandInvocation {
                command: definition.command,
                argument: None,
                focus_target: CommandFocusTarget::Default,
            });
        }

        if definition.loc_id.is_some_and(|loc_id| normalized == normalize_command_name(loc(loc_id)))
        {
            return Some(CommandInvocation {
                command: definition.command,
                argument: None,
                focus_target: CommandFocusTarget::Default,
            });
        }
    }

    command_from_text_with_argument(text)
}

fn command_from_shorthand(text: &str) -> Option<CommandInvocation> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    if text.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(CommandInvocation {
            command: Command::Goto,
            argument: Some(text.to_string()),
            focus_target: CommandFocusTarget::Default,
        });
    }

    if let Some((needle, replacement)) = text.strip_prefix("s/").and_then(|text| text.split_once('/'))
        && !needle.is_empty()
    {
        return Some(CommandInvocation {
            command: Command::Replace,
            argument: Some(format!("{needle} {replacement}")),
            focus_target: CommandFocusTarget::SearchPanel,
        });
    }

    let needle = text.strip_prefix('/')?.trim();
    (!needle.is_empty()).then(|| CommandInvocation {
        command: Command::Find,
        argument: Some(needle.to_string()),
        focus_target: CommandFocusTarget::SearchPanel,
    })
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
                focus_target: CommandFocusTarget::Default,
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
            Some(CommandInvocation { command: Command::SaveAs, argument: None, .. })
        ));
        assert!(matches!(
            command_from_text("select-all"),
            Some(CommandInvocation { command: Command::SelectAll, argument: None, .. })
        ));
        assert!(matches!(
            command_from_text("Go to Line:Column..."),
            Some(CommandInvocation { command: Command::Goto, argument: None, .. })
        ));
        assert!(matches!(
            command_from_text("e"),
            Some(CommandInvocation { command: Command::OpenFile, argument: None, .. })
        ));
        assert!(matches!(
            command_from_text("edit"),
            Some(CommandInvocation { command: Command::OpenFile, argument: None, .. })
        ));
        assert!(matches!(
            command_from_text("file"),
            Some(CommandInvocation { command: Command::SaveAndCloseFile, argument: None, .. })
        ));
        assert!(matches!(
            command_from_text("quit"),
            Some(CommandInvocation { command: Command::CloseFileAndExitIfLast, argument: None, .. })
        ));
    }

    #[test]
    fn command_text_preserves_argument_tail() {
        let Some(CommandInvocation { command: Command::OpenFile, argument: Some(argument), .. }) =
            command_from_text("open path with spaces.txt")
        else {
            panic!("open command did not parse");
        };

        assert!(argument == "path with spaces.txt");
    }

    #[test]
    fn command_text_accepts_parameterized_commands() {
        for (text, expected_command, expected_argument) in [
            ("find needle", Command::Find, "needle"),
            ("replace old new value", Command::Replace, "old new value"),
            ("go-to-file src/main.rs", Command::GoToFile, "src/main.rs"),
            ("go-to-line 42", Command::Goto, "42"),
            ("word-wrap false", Command::WordWrap, "false"),
        ] {
            let Some(CommandInvocation { command, argument: Some(argument), focus_target }) =
                command_from_text(text)
            else {
                panic!("command did not parse: {text}");
            };

            assert!(command == expected_command);
            assert!(argument == expected_argument);
            assert!(focus_target == CommandFocusTarget::Default);
        }
    }

    #[test]
    fn command_text_accepts_commandbar_shorthands() {
        for (text, expected_command, expected_argument, expected_focus_target) in [
            ("42", Command::Goto, "42", CommandFocusTarget::Default),
            (" 42 ", Command::Goto, "42", CommandFocusTarget::Default),
            ("/needle", Command::Find, "needle", CommandFocusTarget::SearchPanel),
            (
                "/needle with spaces",
                Command::Find,
                "needle with spaces",
                CommandFocusTarget::SearchPanel,
            ),
            (" / needle ", Command::Find, "needle", CommandFocusTarget::SearchPanel),
            ("s/old/new", Command::Replace, "old new", CommandFocusTarget::SearchPanel),
            (
                "s/old/new value",
                Command::Replace,
                "old new value",
                CommandFocusTarget::SearchPanel,
            ),
        ] {
            let Some(CommandInvocation { command, argument: Some(argument), focus_target }) =
                command_from_text(text)
            else {
                panic!("command shorthand did not parse: {text}");
            };

            assert!(command == expected_command);
            assert!(argument == expected_argument);
            assert!(focus_target == expected_focus_target);
        }

        assert!(command_from_text("/").is_none());
        assert!(command_from_text("s//new").is_none());
    }

    #[test]
    fn replace_arguments_split_once() {
        assert!(
            command_replace_arguments(&Some("old new value".to_string()))
                == Some(("old".to_string(), "new value".to_string()))
        );
        assert!(command_replace_arguments(&Some("old".to_string())).is_none());
    }

    #[test]
    fn bool_arguments_accept_common_values() {
        for value in ["true", "on", "yes", "1"] {
            assert!(command_bool_argument(&Some(value.to_string())) == Some(true));
        }
        for value in ["false", "off", "no", "0"] {
            assert!(command_bool_argument(&Some(value.to_string())) == Some(false));
        }
        assert!(command_bool_argument(&Some("toggle".to_string())).is_none());
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
            let Some(CommandInvocation { command: Command::InsertText, argument: Some(text), .. }) =
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
        names: &["new", "file-new"],
        loc_id: Some(LocId::FileNew),
    },
    CommandDefinition {
        command: Command::OpenFile,
        names: &["open", "file-open", "e", "edit"],
        loc_id: Some(LocId::FileOpen),
    },
    CommandDefinition { command: Command::SaveAndCloseFile, names: &["file"], loc_id: None },
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
        names: &["close", "file-close"],
        loc_id: Some(LocId::FileClose),
    },
    CommandDefinition { command: Command::Exit, names: &["exit"], loc_id: Some(LocId::FileExit) },
    CommandDefinition { command: Command::CloseFileAndExitIfLast, names: &["quit"], loc_id: None },
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
    CommandDefinition { command: Command::InsertText, names: &["insert"], loc_id: None },
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
        command: Command::SetWordWrapColumn,
        names: &["set-word-wrap-column", "set-wrap-column"],
        loc_id: None,
    },
    CommandDefinition { command: Command::Menu, names: &["menu"], loc_id: None },
    CommandDefinition {
        command: Command::CenterText,
        names: &["set-center-text", "toggle-center-text", "center-text"],
        loc_id: Some(LocId::ViewCenterText),
    },
    CommandDefinition {
        command: Command::About,
        names: &["about"],
        loc_id: Some(LocId::HelpAbout),
    },
    CommandDefinition {
        command: Command::WordCount,
        names: &["word-count"],
        loc_id: Some(LocId::UtilsWordCount),
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
