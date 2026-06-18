// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::tui::Context;

use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
use crate::localization::LocId;
use crate::state::State;

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::About,
        names: &["about"],
        loc_id: Some(LocId::HelpAbout),
        default_focus_target: CommandFocusTarget::Default,
        handler: about,
    },
    CommandDefinition {
        command: Command::WordCount,
        names: &["word-count"],
        loc_id: Some(LocId::UtilsWordCount),
        default_focus_target: CommandFocusTarget::Default,
        handler: word_count,
    },
];

fn about(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.wants_about = true;
}

fn word_count(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    state.wants_word_count = true;
}
