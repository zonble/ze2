// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::icu;
use edit::tui::Context;

use super::arguments::{command_replace_arguments, command_string_argument};
use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
use crate::draw_editor::{SearchAction, search_execute};
use crate::localization::LocId;
use crate::state::*;

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::Find,
        names: &["find", "search"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::EditFind),
        default_focus_target: CommandFocusTarget::Default,
        handler: find,
    },
    CommandDefinition {
        command: Command::Replace,
        names: &["replace"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::EditReplace),
        default_focus_target: CommandFocusTarget::Default,
        handler: replace,
    },
];

fn find(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if state.wants_search.kind != StateSearchKind::Disabled {
        state.wants_search.kind = StateSearchKind::Search;
        if let Some(argument) = command_string_argument(&args.argument) {
            state.search_needle = argument;
            state.wants_search.focus = args.focus_target == CommandFocusTarget::SearchPanel;
            if args.focus_target == CommandFocusTarget::SearchPanel {
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

fn replace(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if state.wants_search.kind != StateSearchKind::Disabled {
        state.wants_search.kind = StateSearchKind::Replace;
        if let Some((needle, replacement)) = command_replace_arguments(&args.argument) {
            state.search_needle = needle;
            state.search_replacement = replacement;
            state.wants_search.focus = args.focus_target == CommandFocusTarget::SearchPanel;
            if args.focus_target == CommandFocusTarget::SearchPanel {
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
