// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use stdext::arena_format;
use ze2::framebuffer::IndexedColor;
use ze2::helpers::*;
use ze2::input::{kbmod, vk};
use ze2::tui::*;

use crate::commands::{
    Command, CommandArgs, CommandFocusTarget, CommandInvocation, execute_command,
    execute_command_invocation,
};
use crate::localization::*;
use crate::settings::{EditorColor, Settings};
use crate::state::*;

pub fn draw_menubar(ctx: &mut Context, state: &mut State, steal_focus_now: bool) {
    let menubar_was_visible = state.menubar_visible;
    if menubar_was_visible && ctx.keyboard_input() == Some(vk::ESCAPE) {
        state.menubar_visible = false;
        state.wants_menubar_focus = false;
        if state.command_bar_active {
            state.command_bar_focus = true;
        } else {
            state.wants_editor_focus = true;
        }
        ctx.set_input_consumed();
        ctx.needs_rerender();
        return;
    }

    let focus_shortcut = ctx.keyboard_input().is_some_and(|key| key == vk::F10 || key == vk::F1)
        || menu_shortcut_selected(ctx, menubar_was_visible, vk::F)
        || menu_shortcut_selected(ctx, menubar_was_visible, vk::E)
        || menu_shortcut_selected(ctx, menubar_was_visible, vk::G)
        || menu_shortcut_selected(ctx, menubar_was_visible, vk::V)
        || menu_shortcut_selected(ctx, menubar_was_visible, vk::U)
        || menu_shortcut_selected(ctx, menubar_was_visible, vk::H);
    if !state.menubar_visible && !focus_shortcut {
        return;
    }

    ctx.menubar_begin();
    ctx.attr_background_rgba(ctx.indexed(IndexedColor::White));
    ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::Black));
    {
        let contains_focus = ctx.contains_focus();
        state.menubar_visible = contains_focus || focus_shortcut || steal_focus_now;

        if ctx.menubar_menu_begin_selected(
            loc(LocId::File),
            'F',
            menu_shortcut_selected(ctx, menubar_was_visible, vk::F),
        ) {
            draw_menu_file(ctx, state);
        }
        if !contains_focus
            && (ctx.consume_shortcut(vk::F10) || ctx.consume_shortcut(vk::F1) || steal_focus_now)
        {
            ctx.steal_focus();
        }
        if state.documents.active().is_some() {
            if ctx.menubar_menu_begin_selected(
                loc(LocId::Edit),
                'E',
                menu_shortcut_selected(ctx, menubar_was_visible, vk::E),
            ) {
                draw_menu_edit(ctx, state);
            }
            if ctx.menubar_menu_begin_selected(
                loc(LocId::Goto),
                'G',
                menu_shortcut_selected(ctx, menubar_was_visible, vk::G),
            ) {
                draw_menu_goto(ctx, state);
            }
            if ctx.menubar_menu_begin_selected(
                loc(LocId::View),
                'V',
                menu_shortcut_selected(ctx, menubar_was_visible, vk::V),
            ) {
                draw_menu_view(ctx, state);
            }
            if ctx.menubar_menu_begin_selected(
                loc(LocId::Utils),
                'U',
                menu_shortcut_selected(ctx, menubar_was_visible, vk::U),
            ) {
                draw_menu_utils(ctx, state);
            }
        }
        if ctx.menubar_menu_begin_selected(
            loc(LocId::Help),
            'H',
            menu_shortcut_selected(ctx, menubar_was_visible, vk::H),
        ) {
            draw_menu_help(ctx, state);
        }
    }
    ctx.menubar_end();
    state.menubar_visible = ctx.contains_focus();
}

fn menu_shortcut_selected(ctx: &Context, menubar_visible: bool, key: ze2::input::InputKey) -> bool {
    ctx.matches_shortcut(kbmod::ALT | key) || (menubar_visible && ctx.matches_shortcut(key))
}

fn draw_menu_file(ctx: &mut Context, state: &mut State) {
    if ctx.menubar_menu_button(loc(LocId::FileNew), 'N', kbmod::CTRL | vk::N) {
        execute_command(ctx, state, Command::NewFile);
    }
    if ctx.menubar_menu_button(loc(LocId::FileOpen), 'O', kbmod::CTRL | vk::O) {
        execute_command(ctx, state, Command::OpenFile);
    }
    if state.documents.active().is_some() {
        if ctx.menubar_menu_button(loc(LocId::FileSave), 'S', kbmod::CTRL | vk::S) {
            execute_command(ctx, state, Command::Save);
        }
        if ctx.menubar_menu_button(loc(LocId::FileSaveAs), 'A', vk::NULL) {
            execute_command(ctx, state, Command::SaveAs);
        }
    }
    let settings = Settings::borrow();
    if !settings.path.as_path().as_os_str().is_empty()
        && ctx.menubar_menu_button(loc(LocId::FilePreferences), 'P', vk::NULL)
    {
        execute_command(ctx, state, Command::Preferences);
    }
    if state.documents.active().is_some()
        && ctx.menubar_menu_button(loc(LocId::FileClose), 'C', kbmod::CTRL | vk::W)
    {
        execute_command(ctx, state, Command::CloseFile);
    }
    if ctx.menubar_menu_button(loc(LocId::FileExit), 'X', kbmod::CTRL | vk::Q) {
        execute_command(ctx, state, Command::Exit);
    }
    ctx.menubar_menu_end();
}

fn draw_menu_edit(ctx: &mut Context, state: &mut State) {
    if ctx.menubar_menu_button(loc(LocId::EditUndo), 'U', kbmod::CTRL | vk::Z) {
        execute_command(ctx, state, Command::Undo);
    }
    if ctx.menubar_menu_button(loc(LocId::EditRedo), 'R', kbmod::CTRL | vk::Y) {
        execute_command(ctx, state, Command::Redo);
    }
    if ctx.menubar_menu_button(loc(LocId::EditCut), 'T', kbmod::CTRL | vk::X) {
        execute_command(ctx, state, Command::Cut);
    }
    if ctx.menubar_menu_button(loc(LocId::EditCopy), 'C', kbmod::CTRL | vk::C) {
        execute_command(ctx, state, Command::Copy);
    }
    if ctx.menubar_menu_button(loc(LocId::EditPaste), 'P', kbmod::CTRL | vk::V) {
        execute_command(ctx, state, Command::Paste);
    }
    if state.wants_search.kind != StateSearchKind::Disabled {
        if ctx.menubar_menu_button(loc(LocId::EditFind), 'F', kbmod::CTRL | vk::F) {
            execute_command(ctx, state, Command::Find);
        }
        if ctx.menubar_menu_button(loc(LocId::EditReplace), 'L', kbmod::CTRL | vk::R) {
            execute_command(ctx, state, Command::Replace);
        }
    }
    if ctx.menubar_menu_button(loc(LocId::EditSelectAll), 'A', kbmod::CTRL | vk::A) {
        execute_command(ctx, state, Command::SelectAll);
    }
    ctx.menubar_menu_end();
}

fn draw_menu_goto(ctx: &mut Context, state: &mut State) {
    // All values on the statusbar are currently document specific.
    if ctx.menubar_menu_button(loc(LocId::ViewFocusStatusbar), 'S', vk::NULL) {
        execute_command(ctx, state, Command::FocusStatusbar);
    }
    if ctx.menubar_menu_button(loc(LocId::ViewGoToFile), 'F', kbmod::CTRL | vk::P) {
        execute_command(ctx, state, Command::GoToFile);
    }
    if ctx.menubar_menu_button(loc(LocId::FileGoto), 'G', kbmod::CTRL | vk::G) {
        execute_command(ctx, state, Command::Goto);
    }
    ctx.menubar_menu_end();
}

fn draw_menu_view(ctx: &mut Context, state: &mut State) {
    if let Some(doc) = state.documents.active() {
        let tb = doc.buffer.borrow();
        let word_wrap = tb.is_word_wrap_enabled();
        let word_wrap_max = tb.word_wrap_max_column();
        drop(tb);

        if ctx.menubar_menu_checkbox(loc(LocId::ViewRuler), 'R', vk::NULL, state.wants_ruler) {
            state.wants_ruler = !state.wants_ruler;
            if let Err(err) = Settings::set_ruler(state.wants_ruler) {
                error_log_add(ctx, state, err);
            }
        }

        if ctx.menubar_menu_checkbox(loc(LocId::ViewWordWrap), 'W', kbmod::ALT | vk::Z, word_wrap) {
            execute_command(ctx, state, Command::WordWrap);
        }
        if ctx.menubar_menu_checkbox(loc(LocId::ViewWordWrap60), '6', vk::NULL, word_wrap_max == 60)
        {
            execute_command_invocation(
                ctx,
                state,
                CommandInvocation {
                    command: Command::SetWordWrapColumn,
                    args: CommandArgs {
                        argument: Some("60".into()),
                        focus_target: CommandFocusTarget::Default,
                    },
                },
            );
        }
        if ctx.menubar_menu_checkbox(loc(LocId::ViewWordWrap80), '8', vk::NULL, word_wrap_max == 80)
        {
            execute_command_invocation(
                ctx,
                state,
                CommandInvocation {
                    command: Command::SetWordWrapColumn,
                    args: CommandArgs {
                        argument: Some("80".into()),
                        focus_target: CommandFocusTarget::Default,
                    },
                },
            );
        }
        if ctx.menubar_menu_checkbox(
            loc(LocId::ViewResetWordWrapColumn),
            '0',
            vk::NULL,
            word_wrap_max == 0,
        ) {
            execute_command_invocation(
                ctx,
                state,
                CommandInvocation {
                    command: Command::SetWordWrapColumn,
                    args: CommandArgs {
                        argument: Some("0".into()),
                        focus_target: CommandFocusTarget::Default,
                    },
                },
            );
        }
        if ctx.menubar_menu_checkbox(
            loc(LocId::ViewCenterText),
            'C',
            vk::NULL,
            state.wants_center_text,
        ) {
            execute_command(ctx, state, Command::CenterText);
        }
        if ctx.menubar_menu_checkbox(
            loc(LocId::ViewHighlightCurrentChar),
            'H',
            vk::NULL,
            state.highlight_current_char,
        ) {
            state.highlight_current_char = !state.highlight_current_char;
            if let Err(err) = Settings::set_highlight_current_char(state.highlight_current_char) {
                error_log_add(ctx, state, err);
            }
        }
        if ctx.menubar_menu_checkbox(
            loc(LocId::ViewEditorColorOriginal),
            'O',
            vk::NULL,
            state.editor_color == EditorColor::Original,
        ) {
            state.editor_color = EditorColor::Original;
            if let Err(err) = Settings::set_editor_color(state.editor_color) {
                error_log_add(ctx, state, err);
            }
        }
        if ctx.menubar_menu_checkbox(
            loc(LocId::ViewEditorColorWhiteOnBlue),
            'L',
            vk::NULL,
            state.editor_color == EditorColor::WhiteOnBlue,
        ) {
            state.editor_color = EditorColor::WhiteOnBlue;
            if let Err(err) = Settings::set_editor_color(state.editor_color) {
                error_log_add(ctx, state, err);
            }
        }
    }

    ctx.menubar_menu_end();
}

fn draw_menu_utils(ctx: &mut Context, state: &mut State) {
    if ctx.menubar_menu_button(loc(LocId::UtilsWordCount), 'W', vk::NULL) {
        execute_command(ctx, state, Command::WordCount);
    }
    ctx.menubar_menu_end();
}

fn draw_menu_help(ctx: &mut Context, state: &mut State) {
    if ctx.menubar_menu_button(loc(LocId::HelpAbout), 'A', vk::NULL) {
        execute_command(ctx, state, Command::About);
    }
    ctx.menubar_menu_end();
}

pub fn draw_dialog_about(ctx: &mut Context, state: &mut State) {
    ctx.modal_begin("about", loc(LocId::AboutDialogTitle));
    {
        ctx.block_begin("content");
        ctx.inherit_focus();
        ctx.attr_padding(Rect::three(1, 2, 1));
        {
            ctx.label("description", "Microsoft Edit");
            ctx.attr_overflow(Overflow::TruncateTail);
            ctx.attr_position(Position::Center);

            ctx.label("description", "(zonble's fork)");
            ctx.attr_overflow(Overflow::TruncateTail);
            ctx.attr_position(Position::Center);

            ctx.label(
                "version",
                &arena_format!(
                    ctx.arena(),
                    "{}{}",
                    loc(LocId::AboutDialogVersion),
                    env!("CARGO_PKG_VERSION")
                ),
            );
            ctx.attr_overflow(Overflow::TruncateHead);
            ctx.attr_position(Position::Center);

            ctx.label("copyright", "Copyright (c) Microsoft Corporation");
            ctx.attr_overflow(Overflow::TruncateTail);
            ctx.attr_position(Position::Center);

            ctx.block_begin("choices");
            ctx.inherit_focus();
            ctx.attr_padding(Rect::three(1, 2, 0));
            ctx.attr_position(Position::Center);
            {
                if ctx.button("ok", loc(LocId::Ok), ButtonStyle::default()) {
                    state.wants_about = false;
                }
                ctx.inherit_focus();
            }
            ctx.block_end();
        }
        ctx.block_end();
    }
    if ctx.modal_end() {
        state.wants_about = false;
    }
}

pub fn draw_dialog_word_count(ctx: &mut Context, state: &mut State) {
    let Some(doc) = state.documents.active() else {
        state.wants_word_count = false;
        return;
    };

    let stats = doc.buffer.borrow().word_count_statistics();

    ctx.modal_begin("word-count", loc(LocId::WordCountDialogTitle));
    {
        ctx.block_begin("content");
        ctx.inherit_focus();
        ctx.attr_padding(Rect::three(1, 2, 1));
        {
            ctx.label(
                "all-characters",
                &arena_format!(
                    ctx.arena(),
                    "{}: {}",
                    loc(LocId::WordCountAllCharacters),
                    stats.all_characters
                ),
            );
            ctx.label(
                "characters-without-linebreaks-and-spaces",
                &arena_format!(
                    ctx.arena(),
                    "{}: {}",
                    loc(LocId::WordCountCharactersWithoutLinebreaksAndSpaces),
                    stats.characters_without_linebreaks_and_spaces
                ),
            );
            ctx.label(
                "all-lines",
                &arena_format!(
                    ctx.arena(),
                    "{}: {}",
                    loc(LocId::WordCountAllLines),
                    stats.all_lines
                ),
            );
            ctx.label(
                "empty-lines",
                &arena_format!(
                    ctx.arena(),
                    "{}: {}",
                    loc(LocId::WordCountEmptyLines),
                    stats.empty_lines
                ),
            );
            ctx.label(
                "lines-with-text",
                &arena_format!(
                    ctx.arena(),
                    "{}: {}",
                    loc(LocId::WordCountLinesWithText),
                    stats.lines_with_text
                ),
            );
            ctx.label(
                "latin-words",
                &arena_format!(
                    ctx.arena(),
                    "{}: {}",
                    loc(LocId::WordCountLatinWords),
                    stats.latin_words
                ),
            );
            ctx.label(
                "asian-characters",
                &arena_format!(
                    ctx.arena(),
                    "{}: {}",
                    loc(LocId::WordCountAsianCharacters),
                    stats.asian_characters
                ),
            );

            ctx.block_begin("choices");
            ctx.inherit_focus();
            ctx.attr_padding(Rect::three(1, 2, 0));
            ctx.attr_position(Position::Center);
            {
                if ctx.button("ok", loc(LocId::Ok), ButtonStyle::default()) {
                    state.wants_word_count = false;
                }
                ctx.inherit_focus();
            }
            ctx.block_end();
        }
        ctx.block_end();
    }
    if ctx.modal_end() {
        state.wants_word_count = false;
    }
}

// ponytail: title is a literal; give it a LocId when the rest of the UI gets one.
pub fn draw_dialog_help(ctx: &mut Context, state: &mut State) {
    let include_vim = state.command_bar_include_vim_commands;
    let include_emacs = state.command_bar_include_emacs_commands;

    let width = (ctx.size().width - 20).max(20);
    let height = (ctx.size().height - 8).max(10);

    ctx.modal_begin("help", "Commands");
    ctx.attr_intrinsic_size(Size { width, height });
    {
        // The list must live in a bounded scrollarea, otherwise it grows past the
        // screen and rows beyond the first viewport can't be reached.
        // -2 for the modal's top and bottom border rows.
        ctx.scrollarea_begin("commands-scroll", Size { width: 0, height: height - 2 });
        {
            ctx.list_begin("commands");
            ctx.inherit_focus();
            // The modal focuses itself on open, but the scrollarea between it and the
            // list breaks the incremental inherit_focus chain, so no item ever gains
            // focus and arrow keys do nothing. Seed focus onto the first item while the
            // list isn't focused yet; steal_focus builds the full path in one shot, and
            // later frames keep focus so navigation works.
            let list_focused = ctx.contains_focus();
            let mut first = true;
            for def in crate::commands::command_definitions() {
                let names: Vec<&str> =
                    def.all_names_with_modes(include_vim, include_emacs).collect();
                if names.is_empty() {
                    continue;
                }
                let mut line = names.join(", ");
                if let Some(hint) = def.argument_hint {
                    line.push(' ');
                    line.push_str(hint);
                }
                if ctx.list_item(false, &line) == ListSelection::Activated {
                    state.wants_help = false;
                }
                ctx.attr_overflow(Overflow::TruncateTail);
                if first && !list_focused {
                    ctx.list_item_steal_focus();
                }
                first = false;
            }
            ctx.list_end();
        }
        ctx.scrollarea_end();
    }
    if ctx.modal_end() {
        state.wants_help = false;
    }
}
