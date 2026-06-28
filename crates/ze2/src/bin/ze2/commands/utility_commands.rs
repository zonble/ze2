// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::time::{SystemTime, UNIX_EPOCH};

use ze2::tui::Context;

use super::arguments::command_bool_argument;
use super::parse::normalize_command_name;
use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget, command_definitions};
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
    CommandDefinition {
        command: Command::QuerySetting,
        names: &["query", "?"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: query_setting,
        argument_hint: Some("<setting>"),
    },
    CommandDefinition {
        command: Command::InsertDate,
        names: &["date"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: insert_date,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Uppercase,
        names: &["uppercase", "uc"],
        namesVim: &[],
        namesEmacs: &["upcase-region"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: uppercase,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Lowercase,
        names: &["lowercase", "lc"],
        namesVim: &[],
        namesEmacs: &["downcase-region"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: lowercase,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::CharCode,
        names: &["char-code", "char"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: char_code,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Help,
        names: &["help"],
        namesVim: &["help"],
        namesEmacs: &["describe-bindings"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: help,
        argument_hint: Some("<command>"),
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

fn query_setting(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let Some(name) = args.argument.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };

    state.command_bar_error = if let Some(doc) = state.documents.active() {
        let tb = doc.buffer.borrow();
        match name.to_ascii_lowercase().as_str() {
            "tabs" | "tab" => format!("tabs {}", tb.tab_size()),
            "tabexpand" => format!("tabexpand {}", !tb.indent_with_tabs()),
            "wrap" | "word-wrap" => format!("wrap {}", tb.is_word_wrap_enabled()),
            "margins" => format!(
                "margins {} {} {}",
                state.reflow_left_margin,
                tb.word_wrap_max_column(),
                state.reflow_paragraph_margin
            ),
            "char" => tb
                .current_char()
                .map_or_else(|| "char EOF".to_string(), |ch| format!("char {}", ch as u32)),
            "encoding" => format!("encoding {}", tb.encoding()),
            "newline" => format!("newline {}", if tb.is_crlf() { "CRLF" } else { "LF" }),
            _ => format!("unknown setting {name}"),
        }
    } else {
        "no active document".to_string()
    };
    state.command_bar_active = true;
}

fn insert_date(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let secs =
            SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_secs());
        doc.buffer.borrow_mut().write_canon(format!("unix:{secs}").as_bytes());
    }
}

fn uppercase(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().change_ascii_case(true);
    }
}

fn lowercase(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().change_ascii_case(false);
    }
}

fn char_code(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let ch =
        args.argument.as_deref().and_then(|text| text.chars().next()).or_else(|| {
            state.documents.active().and_then(|doc| doc.buffer.borrow().current_char())
        });

    state.command_bar_error = ch.map_or_else(
        || "char EOF".to_string(),
        |ch| format!("char {} 0x{:X}", ch as u32, ch as u32),
    );
    state.command_bar_active = true;
}

fn help(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    // Bare `help` opens the scrollable command list; `help <name>` is a quick
    // one-line lookup of a single command's aliases in the command bar.
    // Only the first argument token matters; extra words are ignored.
    match help_query_from_argument(args.argument.as_deref()) {
        Some(query) => {
            state.command_bar_error = help_text(
                query,
                state.command_bar_include_vim_commands,
                state.command_bar_include_emacs_commands,
            );
            state.command_bar_active = true;
        }
        None => state.wants_help = true,
    }
}

fn help_query_from_argument(argument: Option<&str>) -> Option<&str> {
    argument.map(str::trim).and_then(|query| query.split_whitespace().next())
}

fn help_text(query: &str, include_vim: bool, include_emacs: bool) -> String {
    let normalized = normalize_command_name(query);
    let Some(def) = command_definitions().find(|def| {
        def.all_names_with_modes(include_vim, include_emacs)
            .any(|name| normalize_command_name(name) == normalized)
    }) else {
        return format!("no command matching '{query}'");
    };

    let names: Vec<&str> = def.all_names_with_modes(include_vim, include_emacs).collect();
    match def.argument_hint {
        Some(hint) => format!("{} {hint}", names.join(", ")),
        None => names.join(", "),
    }
}

#[cfg(test)]
mod tests {
    use super::{help_query_from_argument, help_text};

    #[test]
    fn help_uses_only_the_second_token() {
        assert_eq!(help_query_from_argument(Some("about aa xacsadf")), Some("about"));
        assert_eq!(help_query_from_argument(Some("ab")), Some("ab"));
        assert_eq!(help_query_from_argument(Some("   ")), None);
    }

    #[test]
    fn help_describes_a_known_command() {
        let text = help_text("save as", true, true);
        assert!(text.contains("save-as") || text.contains("save"), "got: {text}");
    }

    #[test]
    fn help_reports_unknown_command() {
        assert!(help_text("definitely-not-a-command", true, true).starts_with("no command"));
    }
}
