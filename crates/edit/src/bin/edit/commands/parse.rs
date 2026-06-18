// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Command, CommandArgs, CommandFocusTarget, CommandInvocation, command_definitions};
use crate::localization::loc;

pub fn autocomplete_commands(prefix: &str) -> Vec<String> {
    let prefix = normalize_command_name(prefix);
    if prefix.is_empty() {
        return Vec::new();
    }

    let mut suggestions = Vec::new();

    for definition in command_definitions() {
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

    for definition in command_definitions() {
        if definition.names.iter().any(|name| normalized == normalize_command_name(name)) {
            return Some(CommandInvocation {
                command: definition.command,
                args: CommandArgs { argument: None, focus_target: definition.default_focus_target },
            });
        }

        if definition.loc_id.is_some_and(|loc_id| normalized == normalize_command_name(loc(loc_id)))
        {
            return Some(CommandInvocation {
                command: definition.command,
                args: CommandArgs { argument: None, focus_target: definition.default_focus_target },
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
            args: CommandArgs {
                argument: Some(text.to_string()),
                focus_target: CommandFocusTarget::Default,
            },
        });
    }

    if let Some((needle, replacement)) =
        text.strip_prefix("s/").and_then(|text| text.split_once('/'))
        && !needle.is_empty()
    {
        return Some(CommandInvocation {
            command: Command::Replace,
            args: CommandArgs {
                argument: Some(format!("{needle} {replacement}")),
                focus_target: CommandFocusTarget::SearchPanel,
            },
        });
    }

    let needle = text.strip_prefix('/')?.trim();
    (!needle.is_empty()).then(|| CommandInvocation {
        command: Command::Find,
        args: CommandArgs {
            argument: Some(needle.to_string()),
            focus_target: CommandFocusTarget::SearchPanel,
        },
    })
}

fn command_from_text_with_argument(text: &str) -> Option<CommandInvocation> {
    let text = text.trim();
    let (name, argument) = text.split_once(char::is_whitespace)?;
    let normalized = normalize_command_name(name);
    if normalized.is_empty() {
        return None;
    }

    for definition in command_definitions() {
        if definition.names.iter().any(|name| normalized == normalize_command_name(name))
            || definition
                .loc_id
                .is_some_and(|loc_id| normalized == normalize_command_name(loc(loc_id)))
        {
            return Some(CommandInvocation {
                command: definition.command,
                args: CommandArgs {
                    argument: Some(argument.trim().to_string()),
                    focus_target: definition.default_focus_target,
                },
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
            Some(CommandInvocation {
                command: Command::SaveAs,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("select-all"),
            Some(CommandInvocation {
                command: Command::SelectAll,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("Go to Line:Column..."),
            Some(CommandInvocation {
                command: Command::Goto,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("e"),
            Some(CommandInvocation {
                command: Command::OpenFile,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("edit"),
            Some(CommandInvocation {
                command: Command::OpenFile,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("file"),
            Some(CommandInvocation {
                command: Command::SaveAndCloseFile,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("quit"),
            Some(CommandInvocation {
                command: Command::CloseFileAndExitIfLast,
                args: CommandArgs { argument: None, .. },
                ..
            })
        ));
        assert!(matches!(
            command_from_text("focus-statusbar"),
            Some(CommandInvocation {
                command: Command::FocusStatusbar,
                args: CommandArgs { argument: None, focus_target: CommandFocusTarget::StatusBar },
            })
        ));
        assert!(matches!(
            command_from_text("statusbar"),
            Some(CommandInvocation {
                command: Command::FocusStatusbar,
                args: CommandArgs { argument: None, focus_target: CommandFocusTarget::StatusBar },
            })
        ));
    }

    #[test]
    fn command_text_preserves_argument_tail() {
        let Some(CommandInvocation {
            command: Command::OpenFile,
            args: CommandArgs { argument: Some(argument), .. },
        }) = command_from_text("open path with spaces.txt")
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
            ("set-highlight-current-char true", Command::SetHighlightCurrentChar, "true"),
            ("set-editor-color white-on-blue", Command::SetEditorColor, "white-on-blue"),
        ] {
            let Some(CommandInvocation {
                command,
                args: CommandArgs { argument: Some(argument), focus_target },
            }) = command_from_text(text)
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
            ("s/old/new value", Command::Replace, "old new value", CommandFocusTarget::SearchPanel),
        ] {
            let Some(CommandInvocation {
                command,
                args: CommandArgs { argument: Some(argument), focus_target },
            }) = command_from_text(text)
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
}
