//! The extension seam.
//!
//! Downloading a file is rarely the last step. A Garry's Mod addon arrives as a
//! `.gma` that a server may want unpacked; another format will want something
//! else. Rather than growing tapline a special case per game, an [`Extension`]
//! is handed each file as it lands and may act on it.
//!
//! # What an extension is not
//!
//! It is not something a depot can supply. Extensions are Rust code the
//! operator chose and compiled in, selected by name. Nothing in a manifest, a
//! Workshop item or a CDN response can introduce one, which is the same line
//! tapline draws by refusing to execute `installscript.vdf`: installing a game
//! server must not be a way to run code.
//!
//! # Where they run
//!
//! On a blocking task, after the file is synced to disk and never on the task
//! dispatching chunk fetches. That is not an implementation detail — an earlier
//! version of tapline awaited `fsync` on the dispatch loop and lost 13.5 seconds
//! of a 41-second install to it, because nothing new was queued while it ran.
//! Unpacking an archive is more expensive than an `fsync`.
//!
//! # The contract
//!
//! An extension gets a file that is complete and verified: its SHA-1 matched
//! the manifest before it was written, and it has been synced. It may read it,
//! write next to it, and say what it produced. It may not assume anything about
//! files it was not given, because those may still be downloading.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use tapline_ids::AppId;

/// A file that has finished downloading.
#[derive(Debug, Clone)]
pub struct Landed<'a> {
    /// The app it belongs to.
    pub app: AppId,
    /// The directory the install is rooted at.
    ///
    /// Everything an extension writes must stay inside this.
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
    /// Whether the original should be deleted now that it has been unpacked.
    ///
    /// Off by default. Deleting the download is a decision for whoever asked
    /// for the extension, not for the extension.
    pub remove_original: bool,
}

/// Something an extension can do to a file once it lands.
///
/// `Send + Sync` because extensions run on blocking tasks, concurrently with
/// each other and with the rest of the download.
pub trait Extension: Send + Sync {
    /// A stable, lowercase name. This is how the extension is selected from a
    /// command line or across the C ABI, so changing it breaks callers.
    fn name(&self) -> &'static str;

    /// Whether this extension wants this file.
    ///
    /// Called for every file in an install, so it should be cheap — an
    /// extension check is on the path of a 2,329-file install.
    fn claims(&self, file: &Landed<'_>) -> bool;

    /// Act on the file.
    ///
    /// Only called when [`Extension::claims`] returned true. Returning an error
    /// fails the install: an extension that was asked for and could not run has
    /// left the install in a state the caller did not ask for, and continuing
    /// quietly would hide that.
    fn run(&self, file: &Landed<'_>) -> Result<Produced, ExtensionError>;
}

/// Why an extension could not do its job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionError {
    /// The file was not what the extension expected.
    ///
    /// Carries what was found, not just "invalid": a Workshop item is
    /// attacker-authored, and the first question about a rejected one is always
    /// what it actually contained.
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

/// Where an extension should write, given the file it was handed.
///
/// Beside the archive, in a directory named after it without its extension —
/// which is what an addon manager expects to find and what unpacking
/// `addons/foo.gma` into `addons/foo/` means.
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
        // Including the original: deleting a download is the caller's decision.
        let produced = Produced::default();
        assert!(produced.files.is_empty());
        assert!(!produced.remove_original);
    }
}
