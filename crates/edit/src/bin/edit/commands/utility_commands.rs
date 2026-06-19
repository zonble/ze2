// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::tui::Context;

use super::arguments::command_bool_argument;
use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
use crate::localization::LocId;
use crate::settings::Settings;
use crate::state::*;

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::About,
        names: &["about"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::HelpAbout),
        default_focus_target: CommandFocusTarget::Default,
        handler: about,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::WordCount,
        names: &["word-count"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::UtilsWordCount),
        default_focus_target: CommandFocusTarget::Default,
        handler: word_count,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::EnableVimCommands,
        names: &["set-vim-commands-enabled"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: enable_vim_commands,
        argument_hint: Some("<bool>"),
    },
    CommandDefinition {
        command: Command::EnableEmacsCommands,
        names: &["set-emacs-commands-enabled"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: enable_emacs_commands,
        argument_hint: Some("<bool>"),
    },
];

fn about(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.wants_about = true;
}

fn word_count(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.wants_word_count = true;
}

fn enable_vim_commands(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let enabled = command_bool_argument(&args.argument).unwrap_or(true);
    state.command_bar_include_vim_commands = enabled;
    if let Err(err) = Settings::set_command_bar_include_vim_commands(enabled) {
        error_log_add(ctx, state, err);
    }
}

fn enable_emacs_commands(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let enabled = command_bool_argument(&args.argument).unwrap_or(true);
    state.command_bar_include_emacs_commands = enabled;
    if let Err(err) = Settings::set_command_bar_include_emacs_commands(enabled) {
        error_log_add(ctx, state, err);
    }
}
