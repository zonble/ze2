// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::framebuffer::IndexedColor;
use edit::helpers::*;
use edit::input::{kbmod, vk};
use edit::tui::*;
use stdext::arena_format;

use crate::commands::{
    Command, CommandFocusTarget, CommandInvocation, execute_command, execute_command_invocation,
};
use crate::localization::*;
use crate::settings::Settings;
use crate::state::*;

pub fn draw_menubar(ctx: &mut Context, state: &mut State, steal_focus_now: bool) {
    let menubar_was_visible = state.menubar_visible;
    let focus_shortcut = ctx.keyboard_input().is_some_and(|key| key == vk::F10 || key == vk::F1)
        || menu_shortcut_selected(ctx, menubar_was_visible, vk::F)
        || menu_shortcut_selected(ctx, menubar_was_visible, vk::E)
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

fn menu_shortcut_selected(
    ctx: &Context,
    menubar_visible: bool,
    key: edit::input::InputKey,
) -> bool {
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

fn draw_menu_view(ctx: &mut Context, state: &mut State) {
    if let Some(doc) = state.documents.active() {
        let tb = doc.buffer.borrow();
        let word_wrap = tb.is_word_wrap_enabled();
        let word_wrap_max = tb.word_wrap_max_column();
        drop(tb);

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
                    argument: Some("60".into()),
                    focus_target: CommandFocusTarget::Default,
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
                    argument: Some("80".into()),
                    focus_target: CommandFocusTarget::Default,
                },
            );
        }
        if ctx.menubar_menu_checkbox(loc(LocId::ViewResetWordWrapColumn), '0', vk::NULL, word_wrap_max == 0)
        {
            execute_command_invocation(
                ctx,
                state,
                CommandInvocation {
                    command: Command::SetWordWrapColumn,
                    argument: Some("0".into()),
                    focus_target: CommandFocusTarget::Default,
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
