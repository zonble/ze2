// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Command, CommandArgs, CommandFocusTarget, CommandInvocation, command_definitions};
use crate::localization::loc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandAliasSource {
    Standard,
    Vim,
    Emacs,
}

impl CommandAliasSource {
    fn priority(self) -> usize {
        match self {
            CommandAliasSource::Standard => 0,
            CommandAliasSource::Vim => 1,
            CommandAliasSource::Emacs => 2,
        }
    }

    pub fn label(self) -> Option<&'static str> {
        match self {
            CommandAliasSource::Standard => None,
            CommandAliasSource::Vim => Some("vim"),
            CommandAliasSource::Emacs => Some("emacs"),
        }
    }
}

#[derive(Clone)]
pub struct CommandAutocompleteSuggestion {
    pub name: String,
    pub source: CommandAliasSource,
    pub argument_hint: Option<&'static str>,
}

impl CommandAutocompleteSuggestion {
    pub fn display_text(&self) -> String {
        let mut text = self.name.clone();
        if let Some(hint) = self.argument_hint {
            text.push(' ');
            text.push_str(hint);
        }
        if let Some(label) = self.source.label() {
            text.push_str(&format!(" [{}]", label));
        }
        text
    }
}

#[allow(dead_code)]
pub fn autocomplete_commands(prefix: &str) -> Vec<String> {
    autocomplete_commands_with_modes(prefix, true, true)
}

pub fn autocomplete_commands_with_modes(
    prefix: &str,
    include_vim_commands: bool,
    include_emacs_commands: bool,
) -> Vec<String> {
    autocomplete_command_suggestions_with_modes(
        prefix,
        include_vim_commands,
        include_emacs_commands,
    )
    .into_iter()
    .map(|suggestion| suggestion.name)
    .collect()
}

pub fn autocomplete_command_suggestions_with_modes(
    prefix: &str,
    include_vim_commands: bool,
    include_emacs_commands: bool,
) -> Vec<CommandAutocompleteSuggestion> {
    let prefix = normalize_command_name(prefix);
    if prefix.is_empty() {
        return Vec::new();
    }

    let mut suggestions: std::collections::BTreeMap<
        String,
        (CommandAliasSource, Option<&'static str>),
    > = std::collections::BTreeMap::new();

    for definition in command_definitions() {
        for &name in definition.names {
            let norm_name = normalize_command_name(name);
            if norm_name.starts_with(&prefix) {
                insert_alias_suggestion(
                    &mut suggestions,
                    name,
                    CommandAliasSource::Standard,
                    definition.argument_hint,
                );
            }
        }
        if include_vim_commands {
            for &name in definition.namesVim {
                let norm_name = normalize_command_name(name);
                if norm_name.starts_with(&prefix) {
                    insert_alias_suggestion(
                        &mut suggestions,
                        name,
                        CommandAliasSource::Vim,
                        definition.argument_hint,
                    );
                }
            }
        }
        if include_emacs_commands {
            for &name in definition.namesEmacs {
                let norm_name = normalize_command_name(name);
                if norm_name.starts_with(&prefix) {
                    insert_alias_suggestion(
                        &mut suggestions,
                        name,
                        CommandAliasSource::Emacs,
                        definition.argument_hint,
                    );
                }
            }
        }
    }

    let mut suggestions: Vec<CommandAutocompleteSuggestion> = suggestions
        .into_iter()
        .map(|(name, (source, argument_hint))| CommandAutocompleteSuggestion {
            name,
            source,
            argument_hint,
        })
        .collect();
    suggestions.sort_by(|a, b| {
        a.name.cmp(&b.name).then_with(|| a.source.priority().cmp(&b.source.priority()))
    });
    suggestions.truncate(10);
    suggestions
}

fn insert_alias_suggestion(
    suggestions: &mut std::collections::BTreeMap<
        String,
        (CommandAliasSource, Option<&'static str>),
    >,
    name: &str,
    source: CommandAliasSource,
    argument_hint: Option<&'static str>,
) {
    match suggestions.get(name) {
        Some((existing, _)) if existing.priority() <= source.priority() => {}
        _ => {
            suggestions.insert(name.to_string(), (source, argument_hint));
        }
    }
}

#[allow(dead_code)]
pub fn command_from_text(text: &str) -> Option<CommandInvocation> {
    command_from_text_with_modes(text, true, true)
}

pub fn command_from_text_with_modes(
    text: &str,
    include_vim_commands: bool,
    include_emacs_commands: bool,
) -> Option<CommandInvocation> {
    if let Some(invocation) = command_from_shorthand(text) {
        return Some(invocation);
    }

    let normalized = normalize_command_name(text);
    if normalized.is_empty() {
        return None;
    }

    for definition in command_definitions() {
        if definition
            .all_names_with_modes(include_vim_commands, include_emacs_commands)
            .any(|name| normalized == normalize_command_name(name))
        {
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

    command_from_text_with_argument(text, include_vim_commands, include_emacs_commands)
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

fn command_from_text_with_argument(
    text: &str,
    include_vim_commands: bool,
    include_emacs_commands: bool,
) -> Option<CommandInvocation> {
    let text = text.trim();
    let (name, argument) = text.split_once(char::is_whitespace)?;
    let normalized = normalize_command_name(name);
    if normalized.is_empty() {
        return None;
    }

    for definition in command_definitions() {
        if definition
            .all_names_with_modes(include_vim_commands, include_emacs_commands)
            .any(|name| normalized == normalize_command_name(name))
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
                command: Command::SaveAndCloseFileAndExitIfLast,
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
        assert!(matches!(
            command_from_text("u"),
            Some(CommandInvocation {
                command: Command::Undo,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("undo-redo"),
            Some(CommandInvocation {
                command: Command::Redo,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("delete"),
            Some(CommandInvocation {
                command: Command::Cut,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("kill-region"),
            Some(CommandInvocation {
                command: Command::Cut,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("yank"),
            Some(CommandInvocation {
                command: Command::Copy,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("put"),
            Some(CommandInvocation {
                command: Command::Paste,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("mark-whole-buffer"),
            Some(CommandInvocation {
                command: Command::SelectAll,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("set-wrap"),
            Some(CommandInvocation {
                command: Command::WordWrap,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("toggle-truncate-lines"),
            Some(CommandInvocation {
                command: Command::WordWrap,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("set-vim-commands-enabled"),
            Some(CommandInvocation {
                command: Command::EnableVimCommands,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(matches!(
            command_from_text("set-emacs-commands-enabled"),
            Some(CommandInvocation {
                command: Command::EnableEmacsCommands,
                args: CommandArgs { argument: None, .. },
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
            ("set-wrap true", Command::WordWrap, "true"),
            ("visual-line-mode false", Command::WordWrap, "false"),
            ("set-textwidth 80", Command::SetWordWrapColumn, "80"),
            ("set-fill-column 80", Command::SetWordWrapColumn, "80"),
            ("set-vim-commands-enabled false", Command::EnableVimCommands, "false"),
            ("set-emacs-commands-enabled true", Command::EnableEmacsCommands, "true"),
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

    #[test]
    fn command_text_respects_vim_emacs_mode_toggles() {
        assert!(matches!(
            command_from_text_with_modes("undo", false, false),
            Some(CommandInvocation {
                command: Command::Undo,
                args: CommandArgs { argument: None, .. },
            })
        ));
        assert!(command_from_text_with_modes("u", false, false).is_none());
        assert!(command_from_text_with_modes("undo-redo", false, false).is_none());
        assert!(command_from_text_with_modes("u", true, false).is_some());
        assert!(command_from_text_with_modes("undo-redo", false, true).is_some());

        let no_modes = autocomplete_commands_with_modes("set", false, false);
        assert!(!no_modes.iter().any(|name| name == "set-wrap"));
        assert!(!no_modes.iter().any(|name| name == "set-fill-column"));

        let vim_only = autocomplete_commands_with_modes("set", true, false);
        assert!(vim_only.iter().any(|name| name == "set-wrap"));

        let emacs_only = autocomplete_commands_with_modes("set", false, true);
        assert!(emacs_only.iter().any(|name| name == "set-fill-column"));
    }

    #[test]
    fn autocomplete_suggestions_annotate_non_standard_aliases() {
        let suggestions_e = autocomplete_command_suggestions_with_modes("e", true, true);
        assert!(
            suggestions_e.iter().any(|s| s.name == "edit" && s.display_text() == "edit <path>")
        );

        let suggestions_o = autocomplete_command_suggestions_with_modes("o", true, true);
        assert!(
            suggestions_o.iter().any(|s| s.name == "o" && s.display_text() == "o <path> [vim]")
        );
    }
}
