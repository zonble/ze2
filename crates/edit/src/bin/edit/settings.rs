use std::path::PathBuf;
use std::{fs, io};

use edit::buffer::TextBuffer;
use edit::cell::{Ref, SemiRefCell};
use edit::helpers::CoordType;
use edit::json;
use edit::lsh::{LANGUAGES, Language};
use stdext::arena::{read_to_string, scratch_arena};
use stdext::arena_format;

use crate::apperr;

pub struct Settings {
    pub path: PathBuf,
    pub file_associations: Vec<(String, &'static Language)>,
    pub word_wrap: bool,
    pub word_wrap_column: CoordType,
    pub ruler: bool,
    pub center_text: bool,
    pub highlight_current_char: bool,
    pub editor_color: EditorColor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EditorColor {
    Original,
    WhiteOnBlue,
}

struct SettingsCell(SemiRefCell<Settings>);
unsafe impl Sync for SettingsCell {}
static SETTINGS: SettingsCell = SettingsCell(SemiRefCell::new(Settings::new()));

impl Settings {
    /// Fills the given settings.json text buffer with some initial contents for convenience.
    pub fn bootstrap(tb: &mut TextBuffer) {
        tb.set_crlf(false);
        tb.write_raw(b"{\n}\n");
        tb.cursor_move_to_logical(Default::default());
        tb.mark_as_clean();
    }

    const fn new() -> Self {
        Settings {
            path: PathBuf::new(),
            file_associations: Vec::new(),
            word_wrap: false,
            word_wrap_column: 0,
            ruler: false,
            center_text: false,
            highlight_current_char: false,
            editor_color: EditorColor::Original,
        }
    }

    pub fn borrow() -> Ref<'static, Settings> {
        SETTINGS.0.borrow()
    }

    pub fn reload() -> apperr::Result<()> {
        let s = &mut *SETTINGS.0.borrow_mut();

        // Reset all members if we had been loaded previously.
        if !s.path.as_os_str().is_empty() {
            *s = Settings::new();
        }

        s.load()
    }

    fn load(&mut self) -> apperr::Result<()> {
        self.path = match settings_json_path() {
            Some(p) => p,
            None => return Ok(()),
        };

        let scratch = scratch_arena(None);
        let str = match read_to_string(&scratch, &self.path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(err.into()),
            Ok(str) => str,
        };
        let Ok(json) = json::parse(&scratch, &str) else {
            return Err(apperr::Error::SettingsInvalid("Invalid JSON"));
        };
        let Some(root) = json.as_object() else {
            return Err(apperr::Error::SettingsInvalid("Non-object root"));
        };

        if let Some(f) = root.get_object("files.associations") {
            for &(mut key, ref value) in f.iter() {
                if !key.contains('/') {
                    key = arena_format!(&*scratch, "**/{key}").leak();
                }

                let Some(id) = value.as_str() else {
                    return Err(apperr::Error::SettingsInvalid("files.associations"));
                };
                let Some(language) = LANGUAGES.iter().find(|lang| lang.id == id) else {
                    return Err(apperr::Error::SettingsInvalid("language ID"));
                };

                self.file_associations.push((key.to_string(), language));
            }
        }

        if let Some(word_wrap) = root.get_bool("editor.wordWrap") {
            self.word_wrap = word_wrap;
        } else if let Some(word_wrap) = root.get_str("editor.wordWrap") {
            self.word_wrap = matches!(word_wrap, "on" | "true");
        }

        if let Some(ruler) = root.get_bool("editor.ruler") {
            self.ruler = ruler;
        } else if let Some(ruler) = root.get_str("editor.ruler") {
            self.ruler = matches!(ruler, "on" | "true");
        }

        if let Some(column) = root.get_number("editor.wordWrapColumn") {
            self.word_wrap_column = normalize_word_wrap_column(column as CoordType);
        }

        if let Some(center_text) = root.get_bool("editor.centerText") {
            self.center_text = center_text;
        } else if let Some(center_text) = root.get_str("editor.centerText") {
            self.center_text = matches!(center_text, "on" | "true");
        }

        if let Some(highlight_current_char) = root.get_bool("editor.highlightCurrentChar") {
            self.highlight_current_char = highlight_current_char;
        } else if let Some(highlight_current_char) = root.get_str("editor.highlightCurrentChar") {
            self.highlight_current_char = matches!(highlight_current_char, "on" | "true");
        } else if let Some(cursor_style) = root.get_str("editor.cursorStyle") {
            self.highlight_current_char = cursor_style == "block";
        }

        if let Some(editor_color) = root.get_str("editor.color") {
            self.editor_color = match editor_color {
                "whiteOnBlue" => EditorColor::WhiteOnBlue,
                _ => EditorColor::Original,
            };
        }

        Ok(())
    }

    pub fn apply_to_buffer(&self, tb: &mut TextBuffer) {
        tb.set_word_wrap(self.word_wrap);
        tb.set_word_wrap_max_column(self.word_wrap_column);
    }

    pub fn set_word_wrap(enabled: bool) -> apperr::Result<()> {
        let path = Self::write_setting("editor.wordWrap", if enabled { "true" } else { "false" })?;
        let settings = &mut *SETTINGS.0.borrow_mut();
        settings.path = path;
        settings.word_wrap = enabled;
        Ok(())
    }

    pub fn set_ruler(enabled: bool) -> apperr::Result<()> {
        let path = Self::write_setting("editor.ruler", if enabled { "true" } else { "false" })?;
        let settings = &mut *SETTINGS.0.borrow_mut();
        settings.path = path;
        settings.ruler = enabled;
        Ok(())
    }

    pub fn set_center_text(enabled: bool) -> apperr::Result<()> {
        let path =
            Self::write_setting("editor.centerText", if enabled { "true" } else { "false" })?;
        let settings = &mut *SETTINGS.0.borrow_mut();
        settings.path = path;
        settings.center_text = enabled;
        Ok(())
    }

    pub fn set_highlight_current_char(enabled: bool) -> apperr::Result<()> {
        let path = Self::write_setting(
            "editor.highlightCurrentChar",
            if enabled { "true" } else { "false" },
        )?;
        let settings = &mut *SETTINGS.0.borrow_mut();
        settings.path = path;
        settings.highlight_current_char = enabled;
        Ok(())
    }

    pub fn set_editor_color(color: EditorColor) -> apperr::Result<()> {
        let value = match color {
            EditorColor::Original => "\"original\"",
            EditorColor::WhiteOnBlue => "\"whiteOnBlue\"",
        };
        let path = Self::write_setting("editor.color", value)?;
        let settings = &mut *SETTINGS.0.borrow_mut();
        settings.path = path;
        settings.editor_color = color;
        Ok(())
    }

    pub fn set_word_wrap_column(column: CoordType) -> apperr::Result<()> {
        let column = normalize_word_wrap_column(column);
        let path = Self::write_setting("editor.wordWrapColumn", &column.to_string())?;
        let settings = &mut *SETTINGS.0.borrow_mut();
        settings.path = path;
        settings.word_wrap_column = column;
        Ok(())
    }

    fn write_setting(key: &str, value: &str) -> apperr::Result<PathBuf> {
        let Some(path) = settings_json_path() else {
            return Ok(PathBuf::new());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let text = match fs::read_to_string(&path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => "{}\n".to_string(),
            Err(err) => return Err(err.into()),
            Ok(text) => text,
        };
        let text = if text.trim().is_empty() { "{}\n".to_string() } else { text };

        if !text.trim().is_empty() {
            let scratch = scratch_arena(None);
            let json = json::parse(&scratch, &text)
                .map_err(|_| apperr::Error::SettingsInvalid("Invalid JSON"))?;
            if json.as_object().is_none() {
                return Err(apperr::Error::SettingsInvalid("Non-object root"));
            }
        }

        let Some(updated) = update_json_setting(&text, key, value) else {
            return Err(apperr::Error::SettingsInvalid("Non-object root"));
        };
        fs::write(path, updated)?;
        Ok(settings_json_path().unwrap_or_default())
    }
}

fn normalize_word_wrap_column(column: CoordType) -> CoordType {
    if column > 0 { column.max(20) } else { 0 }
}

fn update_json_setting(text: &str, key: &str, value: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let root_start = find_root_object_start(bytes)?;
    let root_end = find_root_object_end(bytes, root_start)?;

    if let Some((value_start, value_end)) = find_top_level_value(bytes, root_start, root_end, key) {
        let mut out = String::with_capacity(text.len() + value.len());
        out.push_str(&text[..value_start]);
        out.push_str(value);
        out.push_str(&text[value_end..]);
        return Some(out);
    }

    let has_entries = object_has_entries(bytes, root_start, root_end);
    let insert_at = if has_entries { trim_end_ws(bytes, root_end) } else { root_start + 1 };
    let mut out = String::with_capacity(text.len() + key.len() + value.len() + 8);
    out.push_str(&text[..insert_at]);
    if has_entries {
        out.push_str(",\n  ");
    } else {
        out.push_str("\n  ");
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    out.push_str(value);
    out.push('\n');
    out.push_str(&text[root_end..]);
    Some(out)
}

fn find_root_object_start(bytes: &[u8]) -> Option<usize> {
    let i = skip_ws_comments(bytes, 0, bytes.len());
    (i < bytes.len() && bytes[i] == b'{').then_some(i)
}

fn find_root_object_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 1;
    let mut depth = 1;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => i = string_end(bytes, i)?,
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 1;
            }
            b'{' | b'[' => depth += 1,
            b'}' | b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn find_top_level_value(
    bytes: &[u8],
    root_start: usize,
    root_end: usize,
    key: &str,
) -> Option<(usize, usize)> {
    let mut i = root_start + 1;
    while i < root_end {
        i = skip_ws_comments(bytes, i, root_end);
        if i >= root_end {
            return None;
        }
        if bytes[i] != b'"' {
            i += 1;
            continue;
        }

        let key_start = i + 1;
        let key_end = string_end(bytes, i)?;
        let mut colon = skip_ws_comments(bytes, key_end + 1, root_end);
        if colon >= root_end || bytes[colon] != b':' {
            i = key_end + 1;
            continue;
        }

        if &bytes[key_start..key_end] == key.as_bytes() {
            colon += 1;
            let value_start = skip_ws_comments(bytes, colon, root_end);
            let value_end = trim_end_ws(bytes, find_value_end(bytes, value_start, root_end)?);
            return Some((value_start, value_end));
        }

        i = key_end + 1;
    }
    None
}

fn find_value_end(bytes: &[u8], start: usize, root_end: usize) -> Option<usize> {
    let mut i = start;
    let mut depth = 1;
    while i < root_end {
        match bytes[i] {
            b'"' => i = string_end(bytes, i)?,
            b'/' if i + 1 < root_end && bytes[i + 1] == b'/' => {
                i += 2;
                while i < root_end && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < root_end && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < root_end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 1;
            }
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth -= 1,
            b',' if depth == 1 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    Some(root_end)
}

fn object_has_entries(bytes: &[u8], root_start: usize, root_end: usize) -> bool {
    skip_ws_comments(bytes, root_start + 1, root_end) < root_end
}

fn skip_ws_comments(bytes: &[u8], mut i: usize, end: usize) -> usize {
    loop {
        while i < end && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i + 1 < end && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            i += 2;
            while i < end && bytes[i] != b'\n' {
                i += 1;
            }
        } else if i + 1 < end && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < end && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(end);
        } else {
            return i;
        }
    }
}

fn trim_end_ws(bytes: &[u8], mut end: usize) -> usize {
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    end
}

fn string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

fn settings_json_path() -> Option<PathBuf> {
    let mut config_dir = config_dir()?;
    config_dir.push("settings.json");
    Some(config_dir)
}

fn config_dir() -> Option<PathBuf> {
    fn var_path(key: &str) -> Option<PathBuf> {
        std::env::var_os(key).map(PathBuf::from)
    }

    fn push(mut path: PathBuf, suffix: &str) -> PathBuf {
        path.push(suffix);
        path
    }

    #[cfg(target_os = "windows")]
    {
        var_path("APPDATA").map(|p| push(p, "Microsoft\\Edit"))
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        var_path("HOME").map(|p| push(p, "Library/Application Support/com.microsoft.edit"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios")))]
    {
        var_path("XDG_CONFIG_HOME")
            .or_else(|| var_path("HOME").map(|p| push(p, ".config")))
            .map(|p| push(p, "msedit"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_json_setting_inserts_into_empty_object() {
        assert_eq!(
            update_json_setting("{}\n", "editor.wordWrap", "true").unwrap(),
            "{\n  \"editor.wordWrap\": true\n}\n"
        );
    }

    #[test]
    fn update_json_setting_replaces_existing_value() {
        assert_eq!(
            update_json_setting(
                "{\n  \"editor.wordWrap\": false,\n  \"editor.wordWrapColumn\": 80\n}\n",
                "editor.wordWrapColumn",
                "120",
            )
            .unwrap(),
            "{\n  \"editor.wordWrap\": false,\n  \"editor.wordWrapColumn\": 120\n}\n"
        );
    }

    #[test]
    fn update_json_setting_preserves_unknown_entries() {
        assert_eq!(
            update_json_setting(
                "{\n  // Keep this comment.\n  \"files.associations\": {\"*.foo\": \"text\"}\n}\n",
                "editor.wordWrap",
                "true",
            )
            .unwrap(),
            "{\n  // Keep this comment.\n  \"files.associations\": {\"*.foo\": \"text\"},\n  \"editor.wordWrap\": true\n}\n"
        );
    }
}
