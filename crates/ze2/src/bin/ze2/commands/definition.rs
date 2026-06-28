// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::tui::Context;

use crate::localization::LocId;
use crate::state::State;

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
    InsertLine,
    SplitLine,
    JoinLine,
    FirstNonblank,
    BeginWord,
    EndWord,
    TabWord,
    BacktabWord,
    MarkLine,
    MarkChar,
    MarkBlock,
    Unmark,
    CopyMark,
    MoveMark,
    DeleteMark,
    FillMark,
    OverlayBlock,
    ShiftLeft,
    ShiftRight,
    CopyToCmd,
    CopyFromCmd,
    FocusStatusbar,
    GoToFile,
    Goto,
    WordWrap,
    About,
    WordCount,
    SaveAndCloseFileAndExitIfLast,
    CloseFileAndExitIfLast,
    SetWordWrapColumn,
    SetMargins,
    SetTabs,
    Reflow,
    Menu,
    CenterText,
    ToggleRuler,
    SetHighlightCurrentChar,
    ToggleHighlightCurrentChar,
    SetEditorColor,
    SetEofStyle,
    SetEncoding,
    ReopenEncoding,
    SetLineBreak,
    EnableVimCommands,
    EnableEmacsCommands,
    QuerySetting,
    InsertDate,
    TransformUppercase,
    TransformLowercase,
    TransformHalfWidth,
    TransformFullWidth,
    TransformLatin,
    TransformKatakana,
    TransformHiragana,
    TransformSimplifiedChinese,
    TransformTraditionalChinese,
    CharCode,
    Help,
}

pub struct CommandInvocation {
    pub command: Command,
    pub args: CommandArgs,
}

pub struct CommandBarShortcut {
    pub text: &'static str,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum CommandFocusTarget {
    #[default]
    Default,
    SearchPanel,
    StatusBar,
}

#[derive(Default)]
pub struct CommandArgs {
    pub argument: Option<String>,
    pub focus_target: CommandFocusTarget,
}

pub type CommandHandler = fn(&mut Context, &mut State, CommandArgs);

#[allow(non_snake_case)]
pub(crate) struct CommandDefinition {
    pub command: Command,
    pub names: &'static [&'static str],
    pub namesVim: &'static [&'static str],
    pub namesEmacs: &'static [&'static str],
    pub loc_id: Option<LocId>,
    pub default_focus_target: CommandFocusTarget,
    pub handler: CommandHandler,
    pub argument_hint: Option<&'static str>,
}

impl CommandDefinition {
    #[allow(dead_code)]
    pub fn all_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.all_names_with_modes(true, true)
    }

    pub fn all_names_with_modes(
        &self,
        include_vim_commands: bool,
        include_emacs_commands: bool,
    ) -> impl Iterator<Item = &'static str> + '_ {
        self.names
            .iter()
            .chain(self.namesVim.iter().filter(move |_| include_vim_commands))
            .chain(self.namesEmacs.iter().filter(move |_| include_emacs_commands))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::commands::command_definitions;
    use crate::commands::parse::normalize_command_name;

    // The table is assembled from several per-category arrays and command_definition()
    // takes the FIRST match by id, so a duplicate Command registration silently makes
    // the later entry unreachable. Replaces the old hand-maintained Command::ALL list
    // (which only proved its own entries had definitions) with the real invariant.
    #[test]
    fn command_table_has_no_duplicate_ids() {
        let mut seen: Vec<Command> = Vec::new();
        for def in command_definitions() {
            assert!(
                !seen.contains(&def.command),
                "duplicate command registration near {:?}",
                def.names.first()
            );
            seen.push(def.command);
        }
    }

    #[test]
    fn command_table_has_no_duplicate_normalized_names() {
        type NameGetter = fn(&CommandDefinition) -> &'static [&'static str];
        for (family, names) in [
            ("standard", (|def: &CommandDefinition| def.names) as NameGetter),
            ("vim", (|def: &CommandDefinition| def.namesVim) as NameGetter),
            ("emacs", (|def: &CommandDefinition| def.namesEmacs) as NameGetter),
        ] {
            let mut seen: BTreeMap<String, Command> = BTreeMap::new();
            for def in command_definitions() {
                for raw in names(def) {
                    let name = normalize_command_name(raw);
                    if let Some(prev) = seen.insert(name.clone(), def.command) {
                        assert!(
                            prev == def.command,
                            "normalized {family} command name collision: {name}"
                        );
                    }
                }
            }
        }
    }
}
