#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use tapline_ids::AppId;

#[derive(Debug, Clone)]
pub struct Landed<'a> {
    pub app: AppId,
    pub root: &'a Path,
    pub path: &'a str,
    pub full_path: &'a Path,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Produced {
    pub files: Vec<String>,
    pub remove_original: bool,
}

pub trait Extension: Send + Sync {
    fn name(&self) -> &'static str;

    fn claims(&self, file: &Landed<'_>) -> bool;

    fn run(&self, file: &Landed<'_>) -> Result<Produced, ExtensionError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionError {
    Malformed {
        extension: &'static str,
        reason: String,
    },
    UnsafePath {
        path: String,
        reason: String,
    },
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub path: String,
    pub size: u64,
    pub offset: u64,
    pub stored_size: u64,
    pub compression: Compression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Compression {
    #[default]
    Stored,
    Deflate,
}

impl ArchiveEntry {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexLocation {
    Head(u64),
    Tail(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexPlan {
    pub entries: Vec<ArchiveEntry>,
    pub needs: Vec<(u64, u64)>,
}

impl IndexPlan {
    #[must_use]
    pub const fn done(entries: Vec<ArchiveEntry>) -> Self {
        Self {
            entries,
            needs: Vec::new(),
        }
    }

    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.needs.is_empty()
    }
}

pub trait EntrySink {
    fn index(&mut self, entries: &[ArchiveEntry]) -> Result<(), ExtensionError>;

    fn begin(&mut self, entry: &ArchiveEntry, index: usize) -> Result<(), ExtensionError>;

    fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError>;

    fn end(&mut self) -> Result<(), ExtensionError>;

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

pub trait Decoder {
    fn format(&self) -> &'static str;

    fn push(&mut self, bytes: &[u8]) -> Result<(), ExtensionError>;

    fn finish(&mut self) -> Result<(), ExtensionError>;
}

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
