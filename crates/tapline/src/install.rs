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
    /// 64 by default. Garry's Mod (3.54 GB), same machine and link:
    ///
    /// | concurrency | wall clock | throughput |
    /// |---|---|---|
    /// | 16 | 29.5 s | 120 MB/s |
    /// | 32 | 21.1 s | 168 MB/s |
    /// | 64 | 18.3 s | 184 MB/s |
    /// | 128 | 20.7 s | 163 MB/s |
    /// | 256 | 21.5 s | 157 MB/s |
    ///
    /// It is not monotonic: past 64 the extra requests cost more than they
    /// carry. 64 also won all four paired Valheim runs (mean 8.3 s against
    /// 9.4 s at 32), which is why the default moved off 32 — the first pass
    /// read the 32-to-64 step as noise, and repeating it showed it was not.
    ///
    /// This has now been wrong twice in opposite directions. It was 16, chosen
    /// as "deliberately modest" with no measurement, then 32, chosen from a
    /// single sweep. Both times the number was defended with a story about
    /// where the bottleneck must be rather than a measurement of where it was.
    ///
    /// Around 184 MB/s the ceiling stops being ours: more concurrency, more CDN
    /// hosts and fewer CDN hosts were each measured and none of them move it,
    /// on a 2.5 Gb link with a 1.9 GB/s disk. It appears to be what Steam serves
    /// one client from one cell.
    pub concurrency: usize,
    /// What permissions to give installed files.
    pub file_modes: FileModes,
    /// Where a Workshop item's files land. Ignored by app installs.
    pub workshop_layout: WorkshopLayout,
}

/// Where a Workshop item's files are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkshopLayout {
    /// `<dir>/steamapps/workshop/content/<app>/<item>/`, which is what
    /// steamcmd does.
    ///
    /// The default, because it is the layout the Steam client, LinuxGSM and
    /// every wings egg already expect to find items in, and because a server
    /// configured with a Workshop collection looks there.
    #[default]
    SteamCmd,
    /// Straight into the directory given, with no path built underneath it.
    ///
    /// What you want when the destination is already the right folder — a
    /// Garry's Mod addon belongs in `garrysmod/addons`, and an item downloaded
    /// there under the steamcmd layout would sit four directories below where
    /// the server looks.
    ///
    /// An item is one or more named files, so several items downloaded flat
    /// into one directory sit side by side. They collide only if two items ship
    /// a file of the same name, which is why this is not the default.
    Flat,
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
            concurrency: 64,
            file_modes: FileModes::default(),
            workshop_layout: WorkshopLayout::default(),
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
        // The upper bound is measured, not assumed. This test used to cap the
        // default at 32 on the reasoning that "a download that opens fifty
        // connections gets throttled" — and 64 turned out to be both the
        // fastest setting and entirely untroubled, with no 429 or 403 in any
        // run. The throttling actually observed came from pulling ~100 GB in an
        // hour, which is a volume limit and not a connection-count one.
        //
        // 128 and 256 were measured too and are slower than 64, so the bound
        // stays: it exists to catch someone raising the default on a hunch, in
        // either direction.
        let concurrency = InstallOptions::default().concurrency;
        assert!(
            (1..=64).contains(&concurrency),
            "default concurrency {concurrency} is outside the measured range"
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
