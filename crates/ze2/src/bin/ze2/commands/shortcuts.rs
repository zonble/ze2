// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::input::{InputKey, InputKeyMod, kbmod, vk};

use super::{Command, CommandArgs, CommandBarShortcut, CommandFocusTarget, CommandInvocation};

struct InsertShortcut {
    modifiers: InputKeyMod,
    key: InputKey,
    text: &'static str,
}

pub fn command_invocation_from_shortcut(key: InputKey) -> Option<CommandInvocation> {
    if let Some(text) = text_from_insert_shortcut(key) {
        return Some(CommandInvocation {
            command: Command::InsertText,
            args: CommandArgs {
                argument: Some(text.to_string()),
                focus_target: CommandFocusTarget::Default,
            },
        });
    }

    Some(match key {
        k if k == kbmod::CTRL | vk::N => Command::NewFile,
        k if k == kbmod::CTRL | vk::O => Command::OpenFile,
        k if k == kbmod::CTRL | vk::S => Command::Save,
        k if k == kbmod::CTRL_SHIFT | vk::S => Command::SaveAs,
        k if k == kbmod::CTRL | vk::W => Command::CloseFile,
        k if k == kbmod::CTRL | vk::P => Command::GoToFile,
        k if k == kbmod::CTRL | vk::Q => Command::Exit,
        k if k == kbmod::CTRL | vk::G => Command::Goto,
        k if k == kbmod::CTRL | vk::F => Command::Find,
        k if k == kbmod::CTRL | vk::R => Command::Replace,
        k if k == kbmod::CTRL | vk::L => Command::SelectLine,
        _ => return None,
    })
    .map(|command| CommandInvocation { command, args: CommandArgs::default() })
}

pub fn commandbar_shortcut_from_key(key: InputKey) -> Option<CommandBarShortcut> {
    Some(CommandBarShortcut {
        text: match key {
            k if k == vk::F2 => "save ",
            k if k == vk::F3 => "file ",
            k if k == vk::F4 => "quit ",
            _ => return None,
        },
    })
}

pub fn should_handle_command_shortcut_before_editor(command: Command) -> bool {
    matches!(command, Command::InsertText)
}

fn text_from_insert_shortcut(key: InputKey) -> Option<&'static str> {
    INSERT_SHORTCUTS
        .iter()
        .find(|shortcut| shortcut.modifiers | shortcut.key == key)
        .map(|shortcut| shortcut.text)
}

const INSERT_SHORTCUTS: &[InsertShortcut] = &[
    InsertShortcut { modifiers: kbmod::ALT, key: vk::COMMA, text: "，" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::PERIOD, text: "。" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::DELETE, text: "。" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::SEMICOLON, text: "；" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::COLON, text: "：" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::SEMICOLON, text: "：" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::APOSTROPHE, text: "、" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::LBRACKET, text: "「" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::RBRACKET, text: "」" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::LBRACE, text: "『" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::LBRACKET, text: "『" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::RBRACE, text: "』" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::RBRACKET, text: "』" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N1, text: "！" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::EXCLAMATION, text: "！" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::N1, text: "！" },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_shortcuts_map_to_text_invocations() {
        for (key, expected) in [
            (kbmod::ALT | vk::COMMA, "，"),
            (kbmod::ALT | vk::PERIOD, "。"),
            (kbmod::ALT | vk::DELETE, "。"),
            (kbmod::ALT | vk::SEMICOLON, "；"),
            (kbmod::ALT | vk::COLON, "："),
            (kbmod::ALT | vk::APOSTROPHE, "、"),
            (kbmod::ALT | vk::LBRACKET, "「"),
            (kbmod::ALT | vk::RBRACKET, "」"),
            (kbmod::ALT | vk::LBRACE, "『"),
            (kbmod::ALT | vk::RBRACE, "』"),
            (kbmod::ALT | vk::N1, "！"),
            (kbmod::ALT | vk::EXCLAMATION, "！"),
        ] {
            let Some(CommandInvocation {
                command: Command::InsertText,
                args: CommandArgs { argument: Some(text), .. },
            }) = command_invocation_from_shortcut(key)
            else {
                panic!("insert shortcut did not parse");
            };

            assert!(text == expected);
        }
    }
}
