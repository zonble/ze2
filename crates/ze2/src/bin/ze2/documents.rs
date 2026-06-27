// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

use ze2::buffer::{RcTextBuffer, TextBuffer};
use ze2::helpers::{CoordType, Point};
use ze2::lsh::{FILE_ASSOCIATIONS, Language, process_file_associations};
use ze2::{path, sys};

use crate::apperr;
use crate::settings::Settings;
use crate::state::DisplayablePathBuf;

const RECOVERY_COPY_MAX_BYTES: u64 = 100 * 1024 * 1024;

pub struct Document {
    pub buffer: RcTextBuffer,
    pub path: Option<PathBuf>,
    pub dir: Option<DisplayablePathBuf>,
    pub filename: String,
    pub file_id: Option<sys::FileId>,
    pub new_file_counter: usize,
    pub language_override: Option<Option<&'static Language>>,
}

impl Document {
    pub fn save(&mut self, new_path: Option<PathBuf>) -> apperr::Result<()> {
        let path = new_path.as_deref().unwrap_or_else(|| self.path.as_ref().unwrap().as_path());
        let mut file = DocumentManager::open_for_writing(path)?;

        {
            let mut tb = self.buffer.borrow_mut();
            tb.write_file(&mut file)?;
        }

        if let Ok(id) = sys::file_id(None, path) {
            self.file_id = Some(id);
        }

        if let Some(path) = new_path {
            self.set_path(path);
        }

        Ok(())
    }

    pub fn reread(&mut self, encoding: Option<&'static str>) -> apperr::Result<()> {
        let path = self.path.as_ref().unwrap().as_path();
        let mut file = DocumentManager::open_for_reading(path)?;

        {
            let mut tb = self.buffer.borrow_mut();
            tb.read_file(&mut file, encoding)?;
        }

        if let Ok(id) = sys::file_id(None, path) {
            self.file_id = Some(id);
        }

        Ok(())
    }

    fn set_path(&mut self, path: PathBuf) {
        let filename = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let dir = path.parent().map(ToOwned::to_owned).unwrap_or_default();

        self.filename = filename;
        self.dir = Some(DisplayablePathBuf::from_path(dir));
        self.path = Some(path);

        self.buffer.borrow_mut().set_ruler(if self.filename == "COMMIT_EDITMSG" { 72 } else { 0 });
        self.update_language();
    }

    pub fn auto_detect_language(&mut self) {
        self.language_override = None;
        self.update_language();
    }

    pub fn override_language(&mut self, lang: Option<&'static Language>) {
        self.language_override = Some(lang);
        self.update_language();
    }

    fn update_language(&mut self) {
        self.buffer.borrow_mut().set_language(self.get_language());
    }

    fn get_language(&self) -> Option<&'static Language> {
        if let Some(lang) = self.language_override {
            return lang;
        }

        if let Some(path) = &self.path {
            let settings = Settings::borrow();
            if let Some(lang) = process_file_associations(&settings.file_associations, path) {
                return Some(lang);
            }
            if let Some(lang) = process_file_associations(FILE_ASSOCIATIONS, path) {
                return Some(lang);
            }
        }

        None
    }
}

#[derive(Default)]
pub struct DocumentManager {
    list: Vec<Document>,
}

impl DocumentManager {
    #[inline]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    #[inline]
    pub fn active(&self) -> Option<&Document> {
        self.list.last()
    }

    #[inline]
    pub fn active_mut(&mut self) -> Option<&mut Document> {
        self.list.last_mut()
    }

    pub fn update_active<F: FnMut(&Document) -> bool>(&mut self, mut func: F) -> bool {
        let Some(idx) = self.list.iter().rposition(&mut func) else {
            return false;
        };

        // Already active (= last) document matched? Nothing to do.
        if idx == self.list.len() - 1 {
            return false;
        }

        // Otherwise, move the matched document to the end of the list so it becomes active.
        // Uses unsafe, because `rotate_left()` is horrendously bad with -Copt-level=s
        // (it's really almost comical) and I just don't tolerate that.
        // If I'm dead and you're looking to rewrite this use `list.push(list.remove(idx))`.
        unsafe {
            let beg = self.list.as_mut_ptr();
            let doc = beg.add(idx);
            let last = beg.add(self.list.len() - 1);
            let amount = self.list.len() - idx - 1;
            let mut temp = std::mem::MaybeUninit::<Document>::uninit();

            // Make a backup of the document
            std::ptr::copy_nonoverlapping(doc, temp.as_mut_ptr(), 1);
            // Shift the rest to the front
            std::ptr::copy(doc.add(1), doc, amount);
            // Move the backup to the end
            std::ptr::copy_nonoverlapping(temp.as_ptr(), last, 1);
        }

        true
    }

    pub fn remove_active(&mut self) {
        self.list.pop();
    }

    pub fn add_untitled(&mut self) -> apperr::Result<&mut Document> {
        let buffer = Self::create_buffer()?;
        let mut doc = Document {
            buffer,
            path: None,
            dir: Default::default(),
            filename: Default::default(),
            file_id: None,
            new_file_counter: 0,
            language_override: None,
        };
        self.gen_untitled_name(&mut doc);

        // In the future this could use push_mut, but it's unstable right now. As usual.
        self.list.push(doc);
        Ok(self.list.last_mut().unwrap())
    }

    pub fn gen_untitled_name(&self, doc: &mut Document) {
        let mut new_file_counter = 0;
        for doc in &self.list {
            new_file_counter = new_file_counter.max(doc.new_file_counter);
        }
        new_file_counter += 1;

        doc.filename = format!("Untitled-{new_file_counter}.txt");
        doc.new_file_counter = new_file_counter;
    }

    pub fn add_file_path(&mut self, path: &Path) -> apperr::Result<&mut Document> {
        let (path, goto) = Self::parse_filename_goto(path);
        let path = path::normalize(path);

        let mut file = match File::open(&path) {
            Ok(file) => Some(file),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => return Err(err.into()),
        };

        let file_id = if file.is_some() { Some(sys::file_id(file.as_ref(), &path)?) } else { None };

        // Check if the file is already open.
        if file_id.is_some() && self.update_active(|doc| doc.file_id == file_id) {
            let doc = self.active_mut().unwrap();
            if let Some(goto) = goto {
                doc.buffer.borrow_mut().cursor_move_to_logical(goto);
            }
            return Ok(doc);
        }

        let buffer = Self::create_buffer()?;
        {
            if let Some(file) = &mut file {
                let mut tb = buffer.borrow_mut();
                tb.read_file(file, None)?;
                Self::write_recovery_copy(&path);

                if let Some(goto) = goto
                    && goto != Default::default()
                {
                    tb.cursor_move_to_logical(goto);
                }
            }
        }

        let mut doc = Document {
            buffer,
            path: None,
            dir: None,
            filename: Default::default(),
            file_id,
            new_file_counter: 0,
            language_override: None,
        };
        doc.set_path(path);

        if let Some(active) = self.active()
            && active.path.is_none()
            && active.file_id.is_none()
            && !active.buffer.borrow().is_dirty()
        {
            // If the current document is a pristine Untitled document with no
            // name and no ID, replace it with the new document.
            self.remove_active();
        }

        self.list.push(doc);
        Ok(self.list.last_mut().unwrap())
    }

    pub fn reflow_all(&self) {
        for doc in &self.list {
            let mut tb = doc.buffer.borrow_mut();
            tb.reflow();
        }
    }

    pub fn open_for_reading(path: &Path) -> apperr::Result<File> {
        File::open(path).map_err(apperr::Error::from)
    }

    pub fn open_for_writing(path: &Path) -> apperr::Result<File> {
        // Error handling for directory creation and file writing

        // It is worth doing an existence check because it is significantly
        // faster than calling mkdir() and letting it fail (at least on Windows).
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        File::create(path).map_err(apperr::Error::from)
    }

    fn create_buffer() -> apperr::Result<RcTextBuffer> {
        let buffer = TextBuffer::new_rc(false)?;
        {
            let mut tb = buffer.borrow_mut();
            tb.set_insert_final_newline(!cfg!(windows)); // As mandated by POSIX.
            tb.set_margin_enabled(true);
            tb.set_line_highlight_enabled(true);
            Settings::borrow().apply_to_buffer(&mut tb);
        }
        Ok(buffer)
    }

    fn write_recovery_copy(path: &Path) {
        let Ok(mut src) = File::open(path) else {
            return;
        };
        let Ok(metadata) = src.metadata() else {
            return;
        };
        if metadata.len() > RECOVERY_COPY_MAX_BYTES {
            return;
        }
        let dst = Self::recovery_copy_path(path);
        let Some(dir) = dst.parent() else {
            return;
        };
        if Self::create_private_recovery_dir(dir).is_err() {
            return;
        }
        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
            options.mode(0o600);
        }
        let Ok(mut dst) = options.open(dst) else {
            return;
        };
        let _ = io::copy(&mut src, &mut dst);
        let _ = dst.flush();
    }

    fn create_private_recovery_dir(dir: &Path) -> io::Result<()> {
        #[cfg(unix)]
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        #[cfg(not(unix))]
        let builder = fs::DirBuilder::new();
        match builder.create(dir) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists && Self::is_private_dir(dir) => {
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn is_private_dir(dir: &Path) -> bool {
        let Ok(metadata) = fs::symlink_metadata(dir) else {
            return false;
        };
        if !metadata.is_dir() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode() & 0o077 == 0
                && metadata.uid() == unsafe { libc::geteuid() }
        }
        #[cfg(not(unix))]
        {
            // Privacy hardening here is Unix-only; other platforms keep this
            // best-effort recovery path available after the basic dir check.
            true
        }
    }

    fn recovery_copy_path(path: &Path) -> PathBuf {
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("file");
        let safe_name: String =
            name.chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                        ch
                    } else {
                        '_'
                    }
                })
                .collect();
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        // Segregate by user so the backup dir is private (created 0700 on write).
        let user = env::var("USER").or_else(|_| env::var("USERNAME")).unwrap_or_default();
        let user_safe: String = user
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
            .collect();
        let hash = hasher.finish();
        env::temp_dir()
            .join(format!("ze2-recovery-{user_safe}-{hash:016x}"))
            .join(format!("{safe_name}.bak"))
    }

    // Parse a filename in the form of "filename:line:char".
    // Returns the position of the first colon and the line/char coordinates.
    fn parse_filename_goto(path: &Path) -> (&Path, Option<Point>) {
        fn parse(s: &[u8]) -> Option<CoordType> {
            if s.is_empty() {
                return None;
            }

            let mut num: CoordType = 0;
            for &b in s {
                if !b.is_ascii_digit() {
                    return None;
                }
                let digit = (b - b'0') as CoordType;
                num = num.checked_mul(10)?.checked_add(digit)?;
            }
            Some(num)
        }

        fn find_colon_rev(bytes: &[u8], offset: usize) -> Option<usize> {
            (0..offset.min(bytes.len())).rev().find(|&i| bytes[i] == b':')
        }

        let bytes = path.as_os_str().as_encoded_bytes();
        let colend = match find_colon_rev(bytes, bytes.len()) {
            // Reject filenames that would result in an empty filename after stripping off the :line:char suffix.
            // For instance, a filename like ":123:456" will not be processed by this function.
            Some(colend) if colend > 0 => colend,
            _ => return (path, None),
        };

        let last = match parse(&bytes[colend + 1..]) {
            Some(last) => last,
            None => return (path, None),
        };
        let last = (last - 1).max(0);
        let mut len = colend;
        let mut goto = Point { x: 0, y: last };

        if let Some(colbeg) = find_colon_rev(bytes, colend) {
            // Same here: Don't allow empty filenames.
            if colbeg != 0
                && let Some(first) = parse(&bytes[colbeg + 1..colend])
            {
                let first = (first - 1).max(0);
                len = colbeg;
                goto = Point { x: last, y: first };
            }
        }

        // Strip off the :line:char suffix.
        let path = &bytes[..len];
        let path = unsafe { OsStr::from_encoded_bytes_unchecked(path) };
        let path = Path::new(path);
        (path, Some(goto))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_last_numbers() {
        fn parse(s: &str) -> (&str, Option<Point>) {
            let (p, g) = DocumentManager::parse_filename_goto(Path::new(s));
            (p.to_str().unwrap(), g)
        }

        assert_eq!(parse("123"), ("123", None));
        assert_eq!(parse("abc"), ("abc", None));
        assert_eq!(parse(":123"), (":123", None));
        assert_eq!(parse("abc:123"), ("abc", Some(Point { x: 0, y: 122 })));
        assert_eq!(parse("45:123"), ("45", Some(Point { x: 0, y: 122 })));
        assert_eq!(parse(":45:123"), (":45", Some(Point { x: 0, y: 122 })));
        assert_eq!(parse("abc:45:123"), ("abc", Some(Point { x: 122, y: 44 })));
        assert_eq!(parse("abc:def:123"), ("abc:def", Some(Point { x: 0, y: 122 })));
        assert_eq!(parse("1:2:3"), ("1", Some(Point { x: 2, y: 1 })));
        assert_eq!(parse("::3"), (":", Some(Point { x: 0, y: 2 })));
        assert_eq!(parse("1::3"), ("1:", Some(Point { x: 0, y: 2 })));
        assert_eq!(parse(""), ("", None));
        assert_eq!(parse(":"), (":", None));
        assert_eq!(parse("::"), ("::", None));
        assert_eq!(parse("a:1"), ("a", Some(Point { x: 0, y: 0 })));
        assert_eq!(parse("1:a"), ("1:a", None));
        assert_eq!(parse("file.txt:10"), ("file.txt", Some(Point { x: 0, y: 9 })));
        assert_eq!(parse("file.txt:10:5"), ("file.txt", Some(Point { x: 4, y: 9 })));
    }

    #[test]
    fn recovery_copy_path_includes_full_source_path() {
        let a = DocumentManager::recovery_copy_path(Path::new("/tmp/a/main.rs"));
        let b = DocumentManager::recovery_copy_path(Path::new("/tmp/b/main.rs"));

        assert_ne!(a, b);
        assert_ne!(a.parent(), b.parent());
        assert_eq!(a.file_name().unwrap().to_string_lossy(), "main.rs.bak");
        assert_eq!(b.file_name().unwrap().to_string_lossy(), "main.rs.bak");
    }

    #[test]
    fn recovery_dir_creation_reuses_private_dir() {
        let dir = std::env::temp_dir().join(format!("ze2-test-recovery-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        DocumentManager::create_private_recovery_dir(&dir).unwrap();
        DocumentManager::create_private_recovery_dir(&dir).unwrap();

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn recovery_copy_overwrites_existing_backup() {
        let src = std::env::temp_dir().join(format!("ze2-test-source-{}", std::process::id()));
        let dst = DocumentManager::recovery_copy_path(&src);
        let _ = fs::remove_file(&src);
        if let Some(dir) = dst.parent() {
            let _ = fs::remove_dir_all(dir);
        }

        fs::write(&src, "old").unwrap();
        DocumentManager::write_recovery_copy(&src);
        fs::write(&src, "new").unwrap();
        DocumentManager::write_recovery_copy(&src);

        assert_eq!(fs::read_to_string(&dst).unwrap(), "new");

        let _ = fs::remove_file(&src);
        if let Some(dir) = dst.parent() {
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn recovery_copy_skips_large_files() {
        let src =
            std::env::temp_dir().join(format!("ze2-test-large-source-{}", std::process::id()));
        let dst = DocumentManager::recovery_copy_path(&src);
        let _ = fs::remove_file(&src);
        if let Some(dir) = dst.parent() {
            let _ = fs::remove_dir_all(dir);
        }

        let file = File::create(&src).unwrap();
        file.set_len(RECOVERY_COPY_MAX_BYTES + 1).unwrap();
        DocumentManager::write_recovery_copy(&src);

        assert!(!dst.exists());

        let _ = fs::remove_file(&src);
        if let Some(dir) = dst.parent() {
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn is_private_dir_checks_permissions_and_symlinks() {
        let dir = std::env::temp_dir().join(format!("ze2-test-private-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        // Test normal private dir creation
        DocumentManager::create_private_recovery_dir(&dir).unwrap();
        assert!(DocumentManager::is_private_dir(&dir));

        // Test with non-private permissions (unix-only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o777)).unwrap();
            assert!(!DocumentManager::is_private_dir(&dir));
        }

        // Test with symlink
        let symlink_path =
            std::env::temp_dir().join(format!("ze2-test-private-symlink-{}", std::process::id()));
        let _ = fs::remove_file(&symlink_path);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&dir, &symlink_path).unwrap();
            assert!(!DocumentManager::is_private_dir(&symlink_path));
            fs::remove_file(&symlink_path).unwrap();
        }

        fs::remove_dir_all(&dir).unwrap();
    }
}
