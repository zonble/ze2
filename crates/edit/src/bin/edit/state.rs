// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::mem;
use std::path::{Path, PathBuf};

use edit::framebuffer::IndexedColor;
use edit::helpers::*;
use edit::oklab::StraightRgba;
use edit::tui::*;
use edit::{buffer, icu};

use crate::apperr;
use crate::documents::DocumentManager;
use crate::localization::*;
use crate::settings::{EditorColor, Settings};

#[repr(transparent)]
pub struct FormatApperr(apperr::Error);

impl From<apperr::Error> for FormatApperr {
    fn from(err: apperr::Error) -> Self {
        Self(err)
    }
}

impl std::fmt::Display for FormatApperr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            apperr::Error::SettingsInvalid(what) => {
                write!(f, "{}{}", loc(LocId::SettingsInvalid), what)
            }
            apperr::Error::Icu(icu::ICU_MISSING_ERROR) => f.write_str(loc(LocId::ErrorIcuMissing)),
            apperr::Error::Icu(ref err) => err.fmt(f),
            apperr::Error::Io(ref err) => err.fmt(f),
        }
    }
}

pub struct DisplayablePathBuf {
    value: PathBuf,
    str: Cow<'static, str>,
}

impl DisplayablePathBuf {
    #[allow(dead_code, reason = "only used on Windows")]
    pub fn from_string(string: String) -> Self {
        let str = Cow::Borrowed(string.as_str());
        let str = unsafe { mem::transmute::<Cow<'_, str>, Cow<'_, str>>(str) };
        let value = PathBuf::from(string);
        Self { value, str }
    }

    pub fn from_path(value: PathBuf) -> Self {
        let str = value.to_string_lossy();
        let str = unsafe { mem::transmute::<Cow<'_, str>, Cow<'_, str>>(str) };
        Self { value, str }
    }

    pub fn as_path(&self) -> &Path {
        &self.value
    }

    pub fn as_str(&self) -> &str {
        &self.str
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.value.as_os_str().as_encoded_bytes()
    }
}

impl Default for DisplayablePathBuf {
    fn default() -> Self {
        Self { value: Default::default(), str: Cow::Borrowed("") }
    }
}

impl Clone for DisplayablePathBuf {
    fn clone(&self) -> Self {
        Self::from_path(self.value.clone())
    }
}

impl From<OsString> for DisplayablePathBuf {
    fn from(s: OsString) -> Self {
        Self::from_path(PathBuf::from(s))
    }
}

impl<T: ?Sized + AsRef<OsStr>> From<&T> for DisplayablePathBuf {
    fn from(s: &T) -> Self {
        Self::from_path(PathBuf::from(s))
    }
}

pub struct StateSearch {
    pub kind: StateSearchKind,
    pub focus: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StateSearchKind {
    Hidden,
    Disabled,
    Search,
    Replace,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StateFilePicker {
    None,
    Open,
    SaveAs,

    SaveAsShown, // Transitioned from SaveAs
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StateEncodingChange {
    None,
    Convert,
    Reopen,
}

#[derive(Default)]
pub struct OscTitleFileStatus {
    pub filename: String,
    pub dirty: bool,
}

pub struct State {
    pub menubar_color_bg: StraightRgba,
    pub menubar_color_fg: StraightRgba,
    pub menubar_visible: bool,

    pub documents: DocumentManager,

    // A ring buffer of the last 10 errors.
    pub error_log: [String; 10],
    pub error_log_index: usize,
    pub error_log_count: usize,

    pub wants_file_picker: StateFilePicker,
    pub file_picker_pending_dir: DisplayablePathBuf,
    pub file_picker_pending_dir_revision: u64, // Bumped every time `file_picker_pending_dir` changes.
    pub file_picker_pending_name: PathBuf,
    pub file_picker_entries: Option<[Vec<DisplayablePathBuf>; 3]>, // ["..", directories, files]
    pub file_picker_overwrite_warning: Option<PathBuf>,            // The path the warning is about.
    pub file_picker_autocomplete: Vec<DisplayablePathBuf>,

    pub wants_search: StateSearch,
    pub search_needle: String,
    pub search_replacement: String,
    pub search_options: buffer::SearchOptions,
    pub search_success: bool,

    pub wants_ruler: bool,
    pub wants_center_text: bool,
    pub highlight_current_char: bool,
    pub editor_color: EditorColor,

    pub wants_language_picker: bool,

    pub wants_encoding_picker: bool,
    pub wants_encoding_change: StateEncodingChange,
    pub encoding_picker_needle: String,
    pub encoding_picker_results: Option<Vec<icu::Encoding>>,

    pub wants_save: bool,
    pub wants_statusbar_focus: bool,
    pub wants_menubar_focus: bool,
    pub wants_indentation_picker: bool,
    pub wants_go_to_file: bool,
    pub wants_about: bool,
    pub wants_word_count: bool,
    pub wants_close: bool,
    pub wants_close_after_save: bool,
    pub wants_exit_after_close: bool,
    pub wants_exit: bool,
    pub wants_exit_after_save: bool,
    pub wants_goto: bool,
    pub goto_target: String,
    pub goto_invalid: bool,
    pub wants_editor_focus: bool,

    pub command_bar_active: bool,
    pub command_bar_focus: bool,
    pub command_bar_input: String,
    pub command_bar_error: String,
    pub command_bar_autocomplete_index: Option<usize>,

    pub osc_title_file_status: OscTitleFileStatus,
    pub osc_clipboard_sync: bool,
    pub osc_clipboard_always_send: bool,
    pub exit: bool,
}

impl State {
    pub fn new() -> apperr::Result<Self> {
        let settings = Settings::borrow();
        let settings_ruler = settings.ruler;
        let settings_center_text = settings.center_text;
        let settings_highlight_current_char = settings.highlight_current_char;
        let settings_editor_color = settings.editor_color;
        drop(settings);

        Ok(Self {
            menubar_color_bg: StraightRgba::zero(),
            menubar_color_fg: StraightRgba::zero(),
            menubar_visible: false,

            documents: Default::default(),

            error_log: [const { String::new() }; 10],
            error_log_index: 0,
            error_log_count: 0,

            wants_file_picker: StateFilePicker::None,
            file_picker_pending_dir: Default::default(),
            file_picker_pending_dir_revision: 0,
            file_picker_pending_name: Default::default(),
            file_picker_entries: None,
            file_picker_overwrite_warning: None,
            file_picker_autocomplete: Vec::new(),

            wants_search: StateSearch { kind: StateSearchKind::Hidden, focus: false },
            search_needle: Default::default(),
            search_replacement: Default::default(),
            search_options: Default::default(),
            search_success: true,

            wants_ruler: settings_ruler,
            wants_center_text: settings_center_text,
            highlight_current_char: settings_highlight_current_char,
            editor_color: settings_editor_color,

            wants_language_picker: false,

            wants_encoding_picker: false,
            encoding_picker_needle: Default::default(),
            encoding_picker_results: Default::default(),

            wants_save: false,
            wants_statusbar_focus: false,
            wants_menubar_focus: false,
            wants_encoding_change: StateEncodingChange::None,
            wants_indentation_picker: false,
            wants_go_to_file: false,
            wants_about: false,
            wants_word_count: false,
            wants_close: false,
            wants_close_after_save: false,
            wants_exit_after_close: false,
            wants_exit: false,
            wants_exit_after_save: false,
            wants_goto: false,
            goto_target: Default::default(),
            goto_invalid: false,
            wants_editor_focus: false,

            command_bar_active: false,
            command_bar_focus: false,
            command_bar_input: Default::default(),
            command_bar_error: Default::default(),
            command_bar_autocomplete_index: None,

            osc_title_file_status: Default::default(),
            osc_clipboard_sync: false,
            osc_clipboard_always_send: false,
            exit: false,
        })
    }

    pub fn add_error(&mut self, err: apperr::Error) -> bool {
        let msg = format!("{}", FormatApperr::from(err));
        if msg.is_empty() {
            return false;
        }

        self.error_log[self.error_log_index] = msg;
        self.error_log_index = (self.error_log_index + 1) % self.error_log.len();
        self.error_log_count = self.error_log.len().min(self.error_log_count + 1);
        true
    }

    pub fn wants_dialog(&self) -> bool {
        self.wants_close
            || self.wants_exit
            || self.wants_goto
            || self.wants_file_picker != StateFilePicker::None
            || self.wants_language_picker
            || self.wants_encoding_change != StateEncodingChange::None
            || self.wants_go_to_file
            || self.wants_about
            || self.wants_word_count
            || self.error_log_count != 0
    }
}

pub fn draw_add_untitled_document(ctx: &mut Context, state: &mut State) {
    if let Err(err) = state.documents.add_untitled() {
        error_log_add(ctx, state, err);
    }
}

pub fn error_log_add(ctx: &mut Context, state: &mut State, err: apperr::Error) {
    if state.add_error(err) {
        ctx.needs_rerender();
    }
}

pub fn draw_error_log(ctx: &mut Context, state: &mut State) {
    ctx.modal_begin("error", loc(LocId::ErrorDialogTitle));
    ctx.attr_background_rgba(ctx.indexed(IndexedColor::Red));
    ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::BrightWhite));
    {
        ctx.block_begin("content");
        ctx.attr_padding(Rect::three(0, 2, 1));
        {
            let off = state.error_log_index + state.error_log.len() - state.error_log_count;

            for i in 0..state.error_log_count {
                let idx = (off + i) % state.error_log.len();
                let msg = &state.error_log[idx][..];

                if !msg.is_empty() {
                    ctx.next_block_id_mixin(i as u64);
                    ctx.label("error", msg);
                    ctx.attr_overflow(Overflow::TruncateTail);
                }
            }
        }
        ctx.block_end();

        if ctx.button("ok", loc(LocId::Ok), ButtonStyle::default()) {
            state.error_log_count = 0;
        }
        ctx.attr_position(Position::Center);
        ctx.inherit_focus();
    }
    if ctx.modal_end() {
        state.error_log_count = 0;
    }
}
