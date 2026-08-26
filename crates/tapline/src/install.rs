//! Turning a manifest into files on disk.

use std::fmt;
use std::path::PathBuf;
use tapline_cdn::{CdnError, PoolError};
use tapline_fs::PathError;
use tapline_ids::{AppId, DepotId};
use tapline_manifest::ManifestError;
use tapline_net::NetError;
use tapline_pics::{DepotFilter, Os, PicsError};

/// What to install, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOptions {
    /// The directory to install into. Created if absent.
    pub install_dir: PathBuf,
    /// The target platform.
    pub os: Os,
    /// The branch, `public` unless a beta was asked for.
    pub branch: String,
    /// Whether to include DLC depots.
    pub include_dlc: bool,
    /// Check what is already on disk before fetching each chunk.
    ///
    /// Turns a killed download into a cheap resume and a corrupted file into a
    /// surgical repair: a chunk that already hashes correctly costs a read
    /// rather than a transfer. Costs one read per chunk, which is why it is not
    /// on by default for a fresh install into an empty directory.
    pub resume: bool,
    /// Reinstall even when the install record says the depot is already at this
    /// build.
    ///
    /// What `validate` turns on. Without it an update that finds nothing changed
    /// does nothing, which is the behaviour an operator wants by default.
    pub force: bool,
    /// How many chunks to fetch at once.
    ///
    /// 32 by default, which is where the measurements stop paying. Installing
    /// Garry's Mod (3.54 GB) on the same machine and link:
    ///
    /// | concurrency | wall clock | throughput |
    /// |---|---|---|
    /// | 16 | 29.5 s | 120 MB/s |
    /// | 32 | 21.1 s | 168 MB/s |
    /// | 64 | 19.5 s | 181 MB/s |
    ///
    /// Going from 32 to 64 buys 1.5 seconds for twice the request rate against
    /// Steam's CDN, and a throttled account costs more than a slow download.
    /// Raise it if you have measured your own link and want the tail.
    ///
    /// The earlier default of 16 was picked as "deliberately modest" without a
    /// measurement behind it, and the note explaining that choice described a
    /// link ceiling that turned out not to exist: 16 was leaving 40% of the
    /// available throughput unused.
    pub concurrency: usize,
    /// What permissions to give installed files.
    pub file_modes: FileModes,
}

/// What permissions installed files get.
///
/// This is a compatibility choice rather than a preference, and it was made
/// from a measurement. Installing Garry's Mod Dedicated Server with both tools
/// on 2026-08-26 gave two trees whose 2,329 files were byte-for-byte identical
/// and whose modes disagreed on 2,291 of them: steamcmd had set **every** file
/// to `0o755`, including text, models and sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileModes {
    /// `0o755` on everything, which is what steamcmd does.
    ///
    /// The default, because tapline is a drop-in replacement and the things
    /// that shell out to steamcmd — LinuxGSM, wings eggs, a decade of Docker
    /// images — were built against trees that look like this. A depot whose
    /// manifest forgets the executable flag on a start script still produces a
    /// runnable server under steamcmd, and swapping in a tool that is stricter
    /// would break it for a reason its operator cannot see.
    #[default]
    SteamCmd,
    /// `0o755` for files the manifest flags executable, `0o644` for the rest.
    ///
    /// What the depot actually describes, and the better answer everywhere the
    /// blunt one is not required for compatibility.
    Manifest,
}

impl FileModes {
    /// The mode for a file the manifest did or did not flag executable.
    #[must_use]
    pub const fn mode_for(self, executable: bool) -> u32 {
        match self {
            Self::SteamCmd => 0o755,
            Self::Manifest if executable => 0o755,
            Self::Manifest => 0o644,
        }
    }
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            install_dir: PathBuf::from("."),
            os: Os::host(),
            branch: "public".to_owned(),
            include_dlc: false,
            resume: true,
            force: false,
            concurrency: 32,
            file_modes: FileModes::default(),
        }
    }
}

impl InstallOptions {
    /// The depot filter these options describe.
    #[must_use]
    pub fn filter(&self) -> DepotFilter {
        DepotFilter {
            os: self.os,
            branch: self.branch.clone(),
            include_dlc: self.include_dlc,
        }
    }
}

/// What an install actually did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstallReport {
    /// The app installed.
    pub app: AppId,
    /// Which depots were taken.
    pub depots: Vec<DepotId>,
    /// How many files were written.
    pub files: u64,
    /// Bytes written to disk.
    pub bytes_written: u64,
    /// Bytes fetched from the CDN, before decompression.
    pub bytes_downloaded: u64,
    /// Chunks that were already correct on disk and so were not refetched.
    pub chunks_reused: u64,
    /// Depots that were already at the requested build and so were skipped.
    pub depots_unchanged: u64,
    /// Files the manifest named that were skipped, with the reason.
    ///
    /// Never silent: a path a manifest asked for and tapline refused to create
    /// is reported, because "the install succeeded" must not quietly mean
    /// "minus three files".
    pub skipped: Vec<(String, String)>,
}

/// What went wrong installing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallError {
    /// The Steam session failed.
    Net(NetError),
    /// PICS could not describe the app.
    Pics(PicsError),
    /// A manifest could not be read.
    Manifest(ManifestError),
    /// Content could not be fetched or verified.
    Cdn(CdnError),
    /// No CDN host is usable.
    Pool(PoolError),
    /// A path in the manifest was refused.
    ///
    /// Fatal rather than skipped. A manifest naming a path outside the install
    /// root is not a file to leave out; it is a manifest to stop trusting.
    UnsafePath {
        /// The path as the manifest wrote it.
        path: String,
        /// Why it was refused.
        reason: PathError,
    },
    /// The filesystem refused.
    Io(String),
    /// Steam granted no decryption key for a depot.
    ///
    /// For an anonymous session this usually means the depot is not anonymously
    /// accessible, which is a different thing from the app not existing.
    NoDepotKey {
        /// Which depot.
        depot: DepotId,
        /// Steam's own result code.
        eresult: i32,
    },
    /// The app resolved to no depots for the requested platform and branch.
    NothingToInstall {
        /// The app.
        app: AppId,
        /// The branch asked for.
        branch: String,
    },
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Net(e) => write!(f, "{e}"),
            Self::Pics(e) => write!(f, "{e}"),
            Self::Manifest(e) => write!(f, "{e}"),
            Self::Cdn(e) => write!(f, "{e}"),
            Self::Pool(e) => write!(f, "{e}"),
            Self::UnsafePath { path, reason } => {
                write!(f, "the manifest named an unsafe path {path:?}: {reason}")
            }
            Self::Io(message) => write!(f, "filesystem error: {message}"),
            Self::NoDepotKey { depot, eresult } => {
                write!(
                    f,
                    "Steam granted no key for depot {depot} (EResult {eresult})"
                )
            }
            Self::NothingToInstall { app, branch } => write!(
                f,
                "app {app} has nothing to install on branch {branch} for this platform"
            ),
        }
    }
}

impl std::error::Error for InstallError {}

impl From<NetError> for InstallError {
    fn from(error: NetError) -> Self {
        Self::Net(error)
    }
}
impl From<PicsError> for InstallError {
    fn from(error: PicsError) -> Self {
        Self::Pics(error)
    }
}
impl From<ManifestError> for InstallError {
    fn from(error: ManifestError) -> Self {
        Self::Manifest(error)
    }
}
impl From<CdnError> for InstallError {
    fn from(error: CdnError) -> Self {
        Self::Cdn(error)
    }
}
impl From<PoolError> for InstallError {
    fn from(error: PoolError) -> Self {
        Self::Pool(error)
    }
}
impl From<std::io::Error> for InstallError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_options_install_for_this_platform_from_public() {
        let options = InstallOptions::default();
        assert_eq!(options.branch, "public");
        assert!(!options.include_dlc);
        assert_eq!(options.os, Os::host());
    }

    #[test]
    fn the_filter_reflects_the_options() {
        let options = InstallOptions {
            os: Os::Windows,
            branch: "beta".to_owned(),
            include_dlc: true,
            ..InstallOptions::default()
        };
        let filter = options.filter();
        assert_eq!(filter.os, Os::Windows);
        assert_eq!(filter.branch, "beta");
        assert!(filter.include_dlc);
    }

    #[test]
    fn concurrency_defaults_to_something_a_cdn_will_tolerate() {
        // Not a round number chosen for looks: Steam rate-limits per host, and
        // a download that opens fifty connections gets throttled.
        let concurrency = InstallOptions::default().concurrency;
        assert!(
            (1..=32).contains(&concurrency),
            "default concurrency {concurrency} is outside what a CDN tolerates"
        );
    }

    #[test]
    fn an_unsafe_path_reads_as_a_refusal_rather_than_a_skip() {
        // The message matters: this is fatal, and an operator reading it should
        // understand the manifest was refused, not that a file was missed.
        let error = InstallError::UnsafePath {
            path: "../../etc/passwd".to_owned(),
            reason: PathError::ParentTraversal,
        };
        let rendered = error.to_string();
        assert!(rendered.contains("unsafe path"), "{rendered}");
        assert!(rendered.contains("../../etc/passwd"), "{rendered}");
    }
}
