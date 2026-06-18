// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::tui::Context;

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
    SaveAndCloseFile,
    CloseFileAndExitIfLast,
    SetWordWrapColumn,
    Menu,
    CenterText,
    SetHighlightCurrentChar,
    ToggleHighlightCurrentChar,
    SetEditorColor,
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
        Command::SaveAndCloseFile,
        Command::CloseFileAndExitIfLast,
        Command::SetWordWrapColumn,
        Command::Menu,
        Command::CenterText,
        Command::SetHighlightCurrentChar,
        Command::ToggleHighlightCurrentChar,
        Command::SetEditorColor,
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

pub(crate) struct CommandDefinition {
    pub command: Command,
    pub names: &'static [&'static str],
    pub loc_id: Option<LocId>,
    pub default_focus_target: CommandFocusTarget,
    pub handler: CommandHandler,
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
