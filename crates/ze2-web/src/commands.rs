// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::tui::Context;

use crate::state::State;

#[path = "../../ze2/src/bin/ze2/commands/arguments.rs"]
mod arguments;
#[path = "../../ze2/src/bin/ze2/commands/definition.rs"]
mod definition;
#[path = "../../ze2/src/bin/ze2/commands/editing_commands.rs"]
mod editing_commands;
#[path = "../../ze2/src/bin/ze2/commands/file_commands.rs"]
mod file_commands;
#[path = "../../ze2/src/bin/ze2/commands/file_format_commands.rs"]
mod file_format_commands;
#[path = "../../ze2/src/bin/ze2/commands/navigation_commands.rs"]
mod navigation_commands;
#[path = "../../ze2/src/bin/ze2/commands/parse.rs"]
mod parse;
#[path = "../../ze2/src/bin/ze2/commands/search_commands.rs"]
mod search_commands;
#[path = "../../ze2/src/bin/ze2/commands/shortcuts.rs"]
mod shortcuts;
#[path = "../../ze2/src/bin/ze2/commands/utility_commands.rs"]
mod utility_commands;
#[path = "../../ze2/src/bin/ze2/commands/view_commands.rs"]
mod view_commands;

pub use definition::{
    Command, CommandArgs, CommandBarShortcut, CommandFocusTarget, CommandInvocation,
};
pub use parse::{autocomplete_command_suggestions_with_modes, command_from_text_with_modes};
pub use shortcuts::{
    command_invocation_from_shortcut, commandbar_shortcut_from_key,
    should_handle_command_shortcut_before_editor,
};

use definition::CommandDefinition;

const COMMAND_GROUPS: &[&[CommandDefinition]] = &[
    file_commands::COMMANDS,
    file_format_commands::COMMANDS,
    editing_commands::COMMANDS,
    search_commands::COMMANDS,
    navigation_commands::COMMANDS,
    view_commands::COMMANDS,
    utility_commands::COMMANDS,
];

pub(crate) fn command_definitions() -> impl Iterator<Item = &'static CommandDefinition> {
    COMMAND_GROUPS.iter().flat_map(|group| group.iter())
}

fn command_definition(command: Command) -> Option<&'static CommandDefinition> {
    command_definitions().find(|definition| definition.command == command)
}

pub fn execute_command(ctx: &mut Context, state: &mut State, command: Command) {
    execute_command_invocation(
        ctx,
        state,
        CommandInvocation { command, args: CommandArgs::default() },
    );
}

pub fn execute_command_invocation(
    ctx: &mut Context,
    state: &mut State,
    invocation: CommandInvocation,
) {
    let Some(definition) = command_definition(invocation.command) else {
        return;
    };

    (definition.handler)(ctx, state, invocation.args);

    ctx.needs_rerender();
}
