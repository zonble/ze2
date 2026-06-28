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
        command: Command::TransformUppercase,
        names: &["transform-uppercase", "uppercase", "uc"],
        namesVim: &[],
        namesEmacs: &["upcase-region"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: transform_uppercase,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::TransformLowercase,
        names: &["transform-lowercase", "lowercase", "lc"],
        namesVim: &[],
        namesEmacs: &["downcase-region"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: transform_lowercase,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::TransformHalfWidth,
        names: &["transform-half-width", "halfwidth"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: transform_half_width,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::TransformFullWidth,
        names: &["transform-full-width", "fullwidth"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: transform_full_width,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::TransformLatin,
        names: &["transform-latin", "latin"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: transform_latin,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::TransformKatakana,
        names: &["transform-katakana", "katakana"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: transform_katakana,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::TransformHiragana,
        names: &["transform-hiragana", "hiragana"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: transform_hiragana,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::TransformSimplifiedChinese,
        names: &["transform-simplified-chinese", "simplified-chinese"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: transform_simplified_chinese,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::TransformTraditionalChinese,
        names: &["transform-traditional-chinese", "traditional-chinese"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: transform_traditional_chinese,
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

fn transform_uppercase(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().change_ascii_case(true);
    }
}

fn transform_lowercase(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().change_ascii_case(false);
    }
}

fn apply_icu_transform(state: &mut State, transform_id: &str) {
    if let Some(doc) = state.documents.active() {
        let mut buffer = doc.buffer.borrow_mut();
        buffer.change_with_icu_transform(transform_id);
    }
}

fn transform_half_width(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    apply_icu_transform(state, transform_id_for_half_width());
}

fn transform_full_width(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    apply_icu_transform(state, transform_id_for_full_width());
}

fn transform_latin(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    apply_icu_transform(state, "Any-Latin");
}

fn transform_katakana(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    apply_icu_transform(state, "Any-Katakana");
}

fn transform_hiragana(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    apply_icu_transform(state, "Any-Hiragana");
}

fn transform_simplified_chinese(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    apply_icu_transform(state, "Hant-Hans");
}

fn transform_traditional_chinese(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    apply_icu_transform(state, "Hans-Hant");
}

fn transform_id_for_half_width() -> &'static str {
    "Fullwidth-Halfwidth"
}

fn transform_id_for_full_width() -> &'static str {
    "Halfwidth-Fullwidth"
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
    use super::{
        help_query_from_argument, help_text, transform_id_for_full_width,
        transform_id_for_half_width,
    };
    use ze2::icu;

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

    #[test]
    fn transform_ids_match_command_names() {
        assert_eq!(transform_id_for_half_width(), "Fullwidth-Halfwidth");
        assert_eq!(transform_id_for_full_width(), "Halfwidth-Fullwidth");
    }

    #[test]
    fn transform_text_ids_cover_any_input() {
        assert_eq!("Any-Latin", "Any-Latin");
        assert_eq!("Any-Katakana", "Any-Katakana");
        assert_eq!("Any-Hiragana", "Any-Hiragana");
        assert_eq!("Hans-Hans", "Hans-Hans");
        assert_eq!("Hans-Hant", "Hans-Hant");
    }

    #[test]
    fn latin_transform_can_expand_output_length() {
        let output = icu::transform_text("Any-Latin", "我是楊維中".as_bytes()).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.len() > "我是楊維中".len(), "got: {output}");
        assert!(output.contains('Y') || output.contains('y'), "got: {output}");
    }

    #[test]
    fn chinese_transform_ids_work_both_directions() {
        let simplified =
            String::from_utf8(icu::transform_text("Hant-Hans", "繁體中文".as_bytes()).unwrap())
                .unwrap();
        let traditional =
            String::from_utf8(icu::transform_text("Hans-Hant", "简体中文".as_bytes()).unwrap())
                .unwrap();
        assert!(simplified.contains("体") || simplified.contains("体"), "got: {simplified}");
        assert!(traditional.contains("體") || traditional.contains("體"), "got: {traditional}");
    }
}
