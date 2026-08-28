//! The extension seam.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use tapline_ids::AppId;

/// A file that has finished downloading.
#[derive(Debug, Clone)]
pub struct Landed<'a> {
    /// The app it belongs to.
    pub app: AppId,
    /// The directory the install is rooted at; nothing may escape it.
    pub root: &'a Path,
    /// The file's path relative to the root, with forward slashes.
    pub path: &'a str,
    /// The file on disk.
    pub full_path: &'a Path,
    /// Its size.
    pub bytes: u64,
}

/// What an extension did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Produced {
    /// Paths it created, relative to the root.
    pub files: Vec<String>,
    /// Whether the original should be deleted; off by default.
    pub remove_original: bool,
}

/// Something an extension can do to a file once it lands.
pub trait Extension: Send + Sync {
    /// A stable, lowercase name; renaming it breaks callers.
    fn name(&self) -> &'static str;

    /// Whether this extension wants this file; called for every file, keep it cheap.
    fn claims(&self, file: &Landed<'_>) -> bool;

    /// Act on the file; an error fails the install.
    fn run(&self, file: &Landed<'_>) -> Result<Produced, ExtensionError>;
}

/// Why an extension could not do its job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionError {
    /// The file was not what the extension expected.
    Malformed {
        /// The extension that refused it.
        extension: &'static str,
        /// What was wrong.
        reason: String,
    },
    /// A path inside the archive escaped the install root.
    UnsafePath {
        /// The path as the archive spelled it.
        path: String,
        /// Why it was refused.
        reason: String,
    },
    /// The filesystem refused.
    Io(String),
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed { extension, reason } => {
                write!(f, "{extension}: {reason}")
            }
            Self::UnsafePath { path, reason } => {
                write!(f, "refused the path {path:?} from an archive: {reason}")
            }
            Self::Io(message) => write!(f, "filesystem error: {message}"),
        }
    }
}

impl std::error::Error for ExtensionError {}

impl From<std::io::Error> for ExtensionError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// One file inside an archive, whatever the archive's format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    /// The path inside the archive; untrusted, chosen by the publisher.
    pub path: String,
    /// The entry's size once unpacked.
    pub size: u64,
    /// Where the entry's stored bytes start in the archive.
    pub offset: u64,
    /// How many bytes to read at that offset; smaller than `size` when compressed.
    pub stored_size: u64,
    /// What was done to those bytes.
    pub compression: Compression,
}

/// How an entry's bytes are stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Compression {
    /// Stored as-is.
    #[default]
    Stored,
    /// Deflated, as a ZIP's method 8.
    Deflate,
}

impl ArchiveEntry {
    /// An entry stored whole, which is what GMAD and a stored ZIP entry are.
    #[must_use]
    pub const fn stored(path: String, offset: u64, size: u64) -> Self {
        Self {
            path,
            size,
            offset,
            stored_size: size,
            compression: Compression::Stored,
        }
    }
}

/// How much of an archive must be read before its index is known, and where.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexLocation {
    /// The first `n` bytes are enough to find it.
    Head(u64),
    /// The last `n` bytes are.
    Tail(u64),
}

/// What a format learned from its index, and what it still needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexPlan {
    /// What is known so far.
    pub entries: Vec<ArchiveEntry>,
    /// Ranges needed before the entries are usable; empty when they already are.
    pub needs: Vec<(u64, u64)>,
}

impl IndexPlan {
    /// A plan that is already complete.
    #[must_use]
    pub const fn done(entries: Vec<ArchiveEntry>) -> Self {
        Self {
            entries,
            needs: Vec::new(),
        }
    }

    /// Whether anything more must be read.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.needs.is_empty()
    }
}

/// What a decoder reports to as it reads an archive.
pub trait EntrySink {
    /// Every entry's name and size is now known; the place to validate paths.
    fn index(&mut self, entries: &[ArchiveEntry]) -> Result<(), ExtensionError>;

    /// An entry's bytes are about to arrive.
    fn begin(&mut self, entry: &ArchiveEntry, index: usize) -> Result<(), ExtensionError>;

    /// Part of the current entry. May be called many times, or not at all.
    fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError>;

    /// The current entry is complete.
    fn end(&mut self) -> Result<(), ExtensionError>;

    /// The archive is complete; write anything held back, like a ZIP's central directory.
    fn finish(&mut self) -> Result<(), ExtensionError> {
        Ok(())
    }
}

impl<S: EntrySink + ?Sized> EntrySink for &mut S {
    fn index(&mut self, entries: &[ArchiveEntry]) -> Result<(), ExtensionError> {
        (**self).index(entries)
    }
    fn begin(&mut self, entry: &ArchiveEntry, index: usize) -> Result<(), ExtensionError> {
        (**self).begin(entry, index)
    }
    fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        (**self).data(bytes)
    }
    fn end(&mut self) -> Result<(), ExtensionError> {
        (**self).end()
    }
    fn finish(&mut self) -> Result<(), ExtensionError> {
        (**self).finish()
    }
}

impl<S: EntrySink + ?Sized> EntrySink for Box<S> {
    fn index(&mut self, entries: &[ArchiveEntry]) -> Result<(), ExtensionError> {
        (**self).index(entries)
    }
    fn begin(&mut self, entry: &ArchiveEntry, index: usize) -> Result<(), ExtensionError> {
        (**self).begin(entry, index)
    }
    fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        (**self).data(bytes)
    }
    fn end(&mut self) -> Result<(), ExtensionError> {
        (**self).end()
    }
    fn finish(&mut self) -> Result<(), ExtensionError> {
        (**self).finish()
    }
}

/// Reads an archive as its bytes arrive, in order, driving an [`EntrySink`].
pub trait Decoder {
    /// The format's name, as a pipeline names it.
    fn format(&self) -> &'static str;

    /// Feeds the next bytes of the archive.
    fn push(&mut self, bytes: &[u8]) -> Result<(), ExtensionError>;

    /// Ends the archive, closing the sink; stopping early is an error.
    fn finish(&mut self) -> Result<(), ExtensionError>;
}

/// Where an extension should write: a directory beside the archive, named after it.
#[must_use]
pub fn unpack_dir(file: &Landed<'_>) -> PathBuf {
    let stem = file
        .full_path
        .file_stem()
        .map_or_else(|| "unpacked".into(), std::ffi::OsStr::to_os_string);
    file.full_path.parent().unwrap_or(file.root).join(stem)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn landed<'a>(full: &'a Path, path: &'a str, root: &'a Path) -> Landed<'a> {
        Landed {
            app: AppId(4000),
            root,
            path,
            full_path: full,
            bytes: 1,
        }
    }

    #[test]
    fn an_archive_unpacks_beside_itself() {
        let root = Path::new("/srv/gmod/garrysmod/addons");
        let full = Path::new("/srv/gmod/garrysmod/addons/104691717.gma");
        assert_eq!(
            unpack_dir(&landed(full, "104691717.gma", root)),
            PathBuf::from("/srv/gmod/garrysmod/addons/104691717")
        );
    }

    #[test]
    fn a_file_without_an_extension_still_gets_a_directory() {
        let root = Path::new("/srv");
        let full = Path::new("/srv/addon");
        assert_eq!(
            unpack_dir(&landed(full, "addon", root)),
            PathBuf::from("/srv/addon")
        );
    }

    #[test]
    fn errors_say_what_was_wrong_rather_than_that_something_was() {
        let malformed = ExtensionError::Malformed {
            extension: "gmad",
            reason: "expected magic GMAD, found PK\\x03\\x04".to_owned(),
        };
        assert_eq!(
            malformed.to_string(),
            "gmad: expected magic GMAD, found PK\\x03\\x04"
        );

        let unsafe_path = ExtensionError::UnsafePath {
            path: "../../etc/passwd".to_owned(),
            reason: "leaves the install root".to_owned(),
        };
        assert!(unsafe_path.to_string().contains("../../etc/passwd"));
    }

    #[test]
    fn nothing_is_produced_by_default() {
        let produced = Produced::default();
        assert!(produced.files.is_empty());
        assert!(!produced.remove_original);
    }
}
