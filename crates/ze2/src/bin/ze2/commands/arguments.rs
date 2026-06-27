// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::env;
use std::path::{Path, PathBuf};
use ze2::icu;
use ze2::path;

use crate::settings::{EditorColor, EofStyle};

pub(crate) fn command_path_argument(argument: &Option<String>) -> Option<PathBuf> {
    let argument = argument.as_deref()?.trim();
    if argument.is_empty() {
        return None;
    }

    let path = Path::new(argument);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().unwrap_or_default().join(path)
    };
    Some(path::normalize(&path))
}

pub(crate) fn command_string_argument(argument: &Option<String>) -> Option<String> {
    let argument = argument.as_deref()?.trim();
    (!argument.is_empty()).then(|| argument.to_string())
}

pub(crate) fn command_replace_arguments(argument: &Option<String>) -> Option<(String, String)> {
    let argument = argument.as_deref()?.trim();
    let (needle, replacement) = argument.split_once(char::is_whitespace)?;
    let needle = needle.trim();
    if needle.is_empty() {
        return None;
    }
    Some((needle.to_string(), replacement.trim().to_string()))
}

pub(crate) fn command_bool_argument(argument: &Option<String>) -> Option<bool> {
    match argument.as_deref()?.trim().to_ascii_lowercase().as_str() {
        "true" | "on" | "yes" | "1" => Some(true),
        "false" | "off" | "no" | "0" => Some(false),
        _ => None,
    }
}

pub(crate) fn command_editor_color_argument(argument: &Option<String>) -> Option<EditorColor> {
    match argument.as_deref()?.trim().to_ascii_lowercase().as_str() {
        "original" => Some(EditorColor::Original),
        "white-on-blue" | "whiteonblue" => Some(EditorColor::WhiteOnBlue),
        _ => None,
    }
}

pub(crate) fn command_eof_style_argument(argument: &Option<String>) -> Option<EofStyle> {
    match argument.as_deref()?.trim().to_ascii_lowercase().as_str() {
        "original" => Some(EofStyle::Original),
        "classic" => Some(EofStyle::Classic),
        "ks3" => Some(EofStyle::Ks3),
        _ => None,
    }
}

pub(crate) fn command_encoding_argument(argument: &Option<String>) -> Option<&'static str> {
    let argument = argument.as_deref()?.trim();
    if argument.is_empty() {
        return None;
    }

    for enc in icu::get_available_encodings().all {
        if enc.canonical.eq_ignore_ascii_case(argument) || enc.label.eq_ignore_ascii_case(argument)
        {
            return Some(enc.canonical);
        }
    }

    None
}

pub(crate) fn command_line_break_argument(argument: &Option<String>) -> Option<bool> {
    match argument.as_deref()?.trim().to_ascii_lowercase().as_str() {
        "crlf" | "cr-lf" | "dos" | "windows" => Some(true),
        "lf" | "unix" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_arguments_split_once() {
        assert!(
            command_replace_arguments(&Some("old new value".to_string()))
                == Some(("old".to_string(), "new value".to_string()))
        );
        assert!(command_replace_arguments(&Some("old".to_string())).is_none());
    }

    #[test]
    fn bool_arguments_accept_common_values() {
        for value in ["true", "on", "yes", "1"] {
            assert!(command_bool_argument(&Some(value.to_string())) == Some(true));
        }
        for value in ["false", "off", "no", "0"] {
            assert!(command_bool_argument(&Some(value.to_string())) == Some(false));
        }
        assert!(command_bool_argument(&Some("toggle".to_string())).is_none());
    }

    #[test]
    fn editor_color_arguments_accept_supported_values() {
        assert!(
            command_editor_color_argument(&Some("original".to_string()))
                == Some(EditorColor::Original)
        );
        assert!(
            command_editor_color_argument(&Some("white-on-blue".to_string()))
                == Some(EditorColor::WhiteOnBlue)
        );
        assert!(
            command_editor_color_argument(&Some("whiteOnBlue".to_string()))
                == Some(EditorColor::WhiteOnBlue)
        );
        assert!(command_editor_color_argument(&Some("blue".to_string())).is_none());
    }

    #[test]
    fn eof_style_arguments_accept_supported_values() {
        assert!(
            command_eof_style_argument(&Some("original".to_string())) == Some(EofStyle::Original)
        );
        assert!(
            command_eof_style_argument(&Some("classic".to_string())) == Some(EofStyle::Classic)
        );
        assert!(command_eof_style_argument(&Some("ks3".to_string())) == Some(EofStyle::Ks3));
        assert!(command_eof_style_argument(&Some("modern".to_string())).is_none());
    }

    #[test]
    fn encoding_arguments_accept_canonical_and_label_names() {
        assert!(command_encoding_argument(&Some("utf-8".to_string())) == Some("UTF-8"));
        assert!(command_encoding_argument(&Some("UTF-8 BOM".to_string())) == Some("UTF-8 BOM"));
        assert!(command_encoding_argument(&Some("not-an-encoding".to_string())).is_none());
    }

    #[test]
    fn line_break_arguments_accept_common_values() {
        for value in ["crlf", "cr-lf", "dos", "windows"] {
            assert!(command_line_break_argument(&Some(value.to_string())) == Some(true));
        }
        for value in ["lf", "unix"] {
            assert!(command_line_break_argument(&Some(value.to_string())) == Some(false));
        }
        assert!(command_line_break_argument(&Some("toggle".to_string())).is_none());
    }
}
