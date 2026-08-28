//! Structured download events, instead of steamcmd's scrapeable stdout lines.

use tapline_ids::{AppId, DepotId, ManifestId};

/// What a download is going to do, computed before any content is fetched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Plan {
    /// Bytes that must come from the CDN.
    pub download_bytes: u64,
    /// Bytes already on disk that will be reused; the delta engine's output.
    pub reused_bytes: u64,
    /// The install's total size once complete.
    pub total_bytes: u64,
    /// How many files the install will contain.
    pub file_count: u64,
    /// Distinct chunks to fetch; a chunk repeated across files is fetched once.
    pub chunk_count: u64,
}

impl Plan {
    /// The reused fraction, 0.0 to 1.0; an empty plan is 0.0.
    #[must_use]
    pub fn reuse_ratio(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "a display ratio, and f64 is exact below 8 PiB"
        )]
        {
            self.reused_bytes as f64 / self.total_bytes as f64
        }
    }
}

/// Why a fetch is being retried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryReason {
    /// The host refused or dropped the connection.
    Transport,
    /// The host rate-limited us.
    RateLimited,
    /// The chunk arrived but failed its SHA-1 check; refetched, never written.
    IntegrityFailure,
    /// The host returned an unexpected status.
    Status(u16),
}

/// Something that happened during a download; non-exhaustive.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Event {
    /// The plan was computed. Always the first event.
    Planned {
        /// What the download will cost.
        plan: Plan,
    },
    /// Work on a depot began.
    DepotStarted {
        /// Which depot.
        depot: DepotId,
        /// The exact build being installed.
        manifest: ManifestId,
        /// How many bytes this depot contributes.
        bytes: u64,
    },
    /// A depot finished.
    DepotCompleted {
        /// Which depot.
        depot: DepotId,
    },
    /// Bytes landed on disk, emitted per chunk written.
    Progress {
        /// Bytes written so far across the whole download.
        bytes_done: u64,
        /// Bytes that will be written in total.
        bytes_total: u64,
    },
    /// A file is complete and verified.
    FileCompleted {
        /// Path relative to the install root, forward slashes as the manifest uses.
        path: String,
        /// The file's size.
        bytes: u64,
    },
    /// A fetch failed and will be tried again.
    Retrying {
        /// The CDN host that failed.
        host: String,
        /// Why.
        reason: RetryReason,
        /// Which attempt this was.
        attempt: u32,
    },
    /// An extension acted on a file it claimed.
    Extended {
        /// Which extension.
        extension: String,
        /// The file it was given, relative to the install root.
        path: String,
        /// How many files it produced.
        produced: u64,
    },
    /// An existing file is being checked during `validate`.
    Verifying {
        /// Path relative to the install root.
        path: String,
    },
    /// The download finished.
    Completed {
        /// The app that was installed.
        app: AppId,
        /// What was actually transferred; a resume may find more on disk than planned.
        downloaded_bytes: u64,
        /// What was reused from disk.
        reused_bytes: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuse_ratio_reports_the_delta_engines_work() {
        let plan = Plan {
            download_bytes: 200 * 1024 * 1024,
            reused_bytes: 29_800 * 1024 * 1024,
            total_bytes: 30_000 * 1024 * 1024,
            file_count: 4_000,
            chunk_count: 210,
        };
        assert!((plan.reuse_ratio() - 0.9933).abs() < 0.001);
    }

    #[test]
    fn an_empty_plan_does_not_divide_by_zero() {
        assert_eq!(Plan::default().reuse_ratio(), 0.0);
    }

    #[test]
    fn an_integrity_failure_is_distinguishable_from_a_transport_failure() {
        assert_ne!(RetryReason::IntegrityFailure, RetryReason::Transport);
        assert_ne!(RetryReason::RateLimited, RetryReason::Transport);
    }
}
