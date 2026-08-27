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
    /// 48 by default: the fewest slots that reach full speed, and not one more.
    ///
    /// This is the only thing bounding peak memory and most of what decides
    /// install speed. A chunk in flight holds its compressed bytes and its
    /// plaintext at once, because the plaintext has to be complete before its
    /// SHA-1 can be checked and nothing reaches the disk before that check.
    /// That is roughly 1.1 MB per slot and it is a floor, not an inefficiency:
    /// chunks are written straight to their offset as they pass, so nothing
    /// accumulates per file. Memory is flat against install size — 6.8 GB costs
    /// what 1.5 GB does — and only this number moves it.
    ///
    /// Measured on both dedicated servers, allocator pinned as [`retune`] does
    /// it. Medians of interleaved repeats, because a single sweep cannot tell a
    /// 2% difference from the link having a bad minute:
    ///
    /// | in flight | Valheim 1.5 GB | GMod 6.8 GB | peak RSS |
    /// |---|---|---|---|
    /// | 12 | 13.6 s | 40.0 s | 28 MB |
    /// | 24 | 12.4 s | 35.8 s | 42 MB |
    /// | 32 | 12.3 s | 35.1 s | 50 MB |
    /// | 40 | 12.8 s | 34.3 s | 55–60 MB |
    /// | **48** | **11.6 s** | **33.5 s** | **61–68 MB** |
    /// | 64 | 12.2 s | 35.3 s | 76–86 MB |
    ///
    /// The curve rises to 48 and turns over after it. 64 is slower than 48 on
    /// both workloads *and* costs 15–18 MB more, so nothing above 48 is worth
    /// buying: past the plateau the extra requests cost more than they carry,
    /// and at 128 it collapses outright to 29 s on Valheim.
    ///
    /// Peak RSS is close to `15 + 1.1 × concurrency` MB, and the fit holds from
    /// 4 slots to 128. So the memory a default costs is predictable from the
    /// number, which is what makes this a choice rather than a surprise.
    ///
    /// # Why the fewest slots rather than the cheapest
    ///
    /// The rule is: use the minimum memory required for full speed. That is
    /// 48 — 40 is consistently ~2% behind it across two independent sweeps, and
    /// 64 is slower. An earlier default of 24 came from a different rule, the
    /// fastest setting under a 50 MB ceiling, and it is 5% off the plateau on
    /// GMod and 7% on Valheim. The ceiling has been retired rather than quietly
    /// kept, because it was answering a question nobody is asking now.
    ///
    /// A smaller footprint is still one flag away: 10 holds a download to
    /// ~25 MB and costs about 25% on Valheim and 39% on GMod.
    ///
    /// The allocator pinning is what makes any of this reproducible. Without it
    /// the same runs measure 46–57 MB and drift by ±20% between repeats; see
    /// [`retune`].
    ///
    /// # This is not the only limit
    ///
    /// A chunk needs a permit from here *and* one from the process-wide budget
    /// in [`Shared`], so what actually runs is the smaller of the two.
    /// [`Session::anonymous`] builds its budget from this default, which means
    /// raising this alone does nothing:
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), tapline::InstallError> {
    /// use tapline::{Session, Shared};
    ///
    /// // Caps at the default budget, not at 96.
    /// let session = Session::anonymous().await?;
    ///
    /// // Actually 96.
    /// let session = Session::anonymous_shared(Shared::new(96)).await?;
    /// # let _ = session;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The cap is deliberate — it is what stops three concurrent downloads
    /// opening three full budgets against Steam. But it silently bounds a
    /// number a caller just set, and that is how the tables here were wrong
    /// twice: a sweep from 8 to 64 that moved neither time nor memory, because
    /// every run in it was really the default. A flat curve is evidence of a
    /// broken experiment at least as often as it is evidence about the system.
    ///
    /// # The default has been wrong before
    ///
    /// 16, chosen as "deliberately modest" with no measurement. Then 32, from a
    /// single sweep. Then 64, from paired runs. Then 8 and 10, when memory
    /// appeared to scale with this number — it did, but the slope was mostly
    /// the allocator's, and pinning it moved the whole curve down. Then 24,
    /// under a memory ceiling. Every wrong answer came from one of two things:
    /// explaining a measurement rather than isolating what produced it, or
    /// optimising a rule nobody had actually asked for.
    ///
    /// Around 200 MB/s the link stops being ours: more CDN hosts and fewer CDN
    /// hosts were each measured and neither moves it, on a 2.5 Gb link with a
    /// 1.9 GB/s disk. It appears to be what Steam serves one client from one
    /// cell.
    ///
    /// [`retune`]: crate::retune
    /// [`Shared`]: crate::Shared
    /// [`Session::anonymous`]: crate::Session::anonymous
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
            concurrency: 48,
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
        // Everything above 64 measured slower, and 64 itself is slower than
        // the plateau at 48, so the bound stays: it exists to catch someone
        // raising the default on a hunch, in either direction.
        let concurrency = InstallOptions::default().concurrency;
        assert!(
            (1..=64).contains(&concurrency),
            "default concurrency {concurrency} is outside the measured range"
        );
    }

    #[test]
    fn the_default_concurrency_is_the_measured_plateau() {
        // The rule is: the minimum memory required for full speed. Not the
        // cheapest setting, and not the fastest at any price.
        //
        // Measured as medians of interleaved repeats on both dedicated servers,
        // the curve rises to 48 and turns over: 64 is slower on both *and*
        // costs 15-18 MB more, and 40 is ~2% behind 48 in two independent
        // sweeps. So the plateau is a single value, and this pins it.
        //
        // There used to be a 50 MB ceiling asserted here with 5 MB of headroom.
        // It has been retired deliberately: the default that satisfies it (24)
        // is 5-7% off full speed, so the ceiling was answering a question that
        // is no longer being asked. Peak RSS is still predictable — about
        // `15 + 1.1 x concurrency` MB, which puts the default near 68 MB — it
        // is simply no longer bounded by a number this test enforces.
        const PLATEAU: usize = 48;

        let concurrency = InstallOptions::default().concurrency;
        assert_eq!(
            concurrency, PLATEAU,
            "the default should sit at the measured plateau; \
             above it is slower and dearer, below it is slower and cheaper"
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
