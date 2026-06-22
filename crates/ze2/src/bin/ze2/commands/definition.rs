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
    FocusStatusbar,
    GoToFile,
    Goto,
    WordWrap,
    About,
    WordCount,
    SaveAndCloseFileAndExitIfLast,
    CloseFileAndExitIfLast,
    SetWordWrapColumn,
    Menu,
    CenterText,
    SetHighlightCurrentChar,
    ToggleHighlightCurrentChar,
    SetEditorColor,
    SetEncoding,
    ReopenEncoding,
    SetLineBreak,
    EnableVimCommands,
    EnableEmacsCommands,
}

#[cfg(test)]
impl Command {
    const ALL: &[Command] = &[
        Command::NewFile,
        Command::OpenFile,
        Command::Save,
        Command::SaveAs,
        Command::Preferences,
        Command::CloseFile,
        Command::Exit,
        Command::Undo,
        Command::Redo,
        Command::Cut,
        Command::Copy,
        Command::Paste,
        Command::Find,
        Command::Replace,
        Command::SelectAll,
        Command::SelectLine,
        Command::InsertText,
        Command::FocusStatusbar,
        Command::GoToFile,
        Command::Goto,
        Command::WordWrap,
        Command::About,
        Command::WordCount,
        Command::SaveAndCloseFileAndExitIfLast,
        Command::CloseFileAndExitIfLast,
        Command::SetWordWrapColumn,
        Command::Menu,
        Command::CenterText,
        Command::SetHighlightCurrentChar,
        Command::ToggleHighlightCurrentChar,
        Command::SetEditorColor,
        Command::SetEncoding,
        Command::ReopenEncoding,
        Command::SetLineBreak,
        Command::EnableVimCommands,
        Command::EnableEmacsCommands,
    ];
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
    use super::*;
    use crate::commands::command_definition;

    #[test]
    fn every_command_has_a_definition() {
        for &command in Command::ALL {
            assert!(command_definition(command).is_some());
        }
    }
}
