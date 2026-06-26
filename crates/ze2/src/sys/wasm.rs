// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::ffi::{c_char, c_void};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::time;

use stdext::arena::Arena;
use stdext::collections::{BString, BVec};

pub fn init() -> Deinit {
    Deinit
}

pub fn reopen_stdin_if_redirected() -> io::Result<Option<File>> {
    Ok(None)
}

pub fn switch_modes() -> io::Result<()> {
    Ok(())
}

pub struct Deinit;

pub fn inject_window_size_into_stdin() {}

pub fn read_stdin(_arena: &Arena, _timeout: time::Duration) -> Option<BString<'_>> {
    Some(BString::empty())
}

pub fn write_stdout(_text: &str) {}

#[derive(Clone, PartialEq, Eq)]
pub struct FileId(PathBuf);

pub fn file_id(_file: Option<&File>, path: &Path) -> io::Result<FileId> {
    Ok(FileId(path.to_path_buf()))
}

pub struct LibIcu {
    pub libicuuc: NonNull<c_void>,
    pub libicui18n: NonNull<c_void>,
}

pub fn load_icu() -> io::Result<LibIcu> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "dynamic ICU loading is not available in wasm"))
}

pub unsafe fn get_proc_address<T>(_handle: NonNull<c_void>, _name: *const c_char) -> io::Result<T> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "dynamic symbol lookup is not available in wasm",
    ))
}

pub fn icu_detect_renaming_suffix(_arena: &Arena, _handle: NonNull<c_void>) -> BString<'_> {
    BString::empty()
}

pub fn icu_add_renaming_suffix<'a>(
    _arena: &'a Arena,
    name: *const c_char,
    _suffix: &str,
) -> *const c_char {
    name
}

pub fn preferred_languages<'a>(arena: &'a Arena) -> BVec<'a, &'a str> {
    let mut langs = BVec::empty();
    langs.push(arena, "zh-Hant");
    langs
}
