//! What a download tells its caller while it runs.
//!
//! steamcmd's answer to this question is a line of text on stdout, which is why
//! every tool that drives it ends up with a regular expression and a bug report
//! about a Steam update that changed the wording. These are structured values
//! instead: a scheduler can act on them, a CLI can render them, and neither has
//! to guess.
//!
//! The events carry no timestamps and compute no rates. A consumer that wants a
//! transfer rate has a clock and knows when it received the last event; baking
//! one in here would mean picking a smoothing window on the caller's behalf.

use tapline_ids::{AppId, DepotId, ManifestId};

/// What a download is going to do, worked out before any content is fetched.
///
/// This is what makes `plan()` worth having as a separate step: a scheduler
/// deciding whether to start an install on a node wants the byte cost before it
/// commits the disk, and an operator wants to know that a "20 GB update" is
/// really 200 MB of changed chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Plan {
    /// Bytes that must come from the CDN.
    pub download_bytes: u64,
    /// Bytes already on disk that will be reused rather than refetched.
    ///
    /// Non-zero only for an update or a resume. This is the delta engine's whole
    /// output, and the number worth quoting in a benchmark.
    pub reused_bytes: u64,
    /// The install's total size once complete.
    pub total_bytes: u64,
    /// How many files the install will contain.
    pub file_count: u64,
    /// How many distinct chunks must be fetched.
    ///
    /// Distinct: a chunk repeated across files is fetched once and written to
    /// every offset that references it.
    pub chunk_count: u64,
}

impl Plan {
    /// The fraction of the install that will be reused, from 0.0 to 1.0.
    ///
    /// Returns 0.0 for an empty plan rather than dividing by zero.
    #[must_use]
    pub fn reuse_ratio(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        // Precision loss is acceptable and bounded: this is a progress figure,
        // and f64 holds every integer up to 2^53, which is 8 PiB.
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
    ///
    /// Reported separately because it is the one worth surfacing to an operator:
    /// it means backing off, and a download that hits it constantly is one
    /// configured to hammer a single host.
    RateLimited,
    /// The chunk arrived but failed its SHA-1 check.
    ///
    /// This is the interesting one. It means a CDN or a caching proxy served
    /// bytes that are not what the manifest named, and tapline refetched rather
    /// than writing them.
    IntegrityFailure,
    /// The host returned an unexpected status.
    Status(u16),
}

/// Something that happened during a download.
///
/// Non-exhaustive: new events are added as the pipeline grows, and a consumer
/// matching on this should not break when one appears.
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
    /// Bytes landed on disk.
    ///
    /// Emitted per chunk written, so a consumer wanting a smoother figure should
    /// aggregate rather than expect this to be throttled for it.
    Progress {
        /// Bytes written so far across the whole download.
        bytes_done: u64,
        /// Bytes that will be written in total.
        bytes_total: u64,
    },
    /// A file is complete and verified.
    FileCompleted {
        /// Path relative to the install root, using forward slashes as the
        /// manifest does.
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
    /// An existing file is being checked during `validate`.
    Verifying {
        /// Path relative to the install root.
        path: String,
    },
    /// The download finished.
    Completed {
        /// The app that was installed.
        app: AppId,
        /// What was actually transferred, which may be less than planned if a
        /// resume found more on disk than expected.
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
        // These mean very different things: one is a flaky network, the other is
        // a CDN or a cache serving bytes the manifest did not name.
        assert_ne!(RetryReason::IntegrityFailure, RetryReason::Transport);
        assert_ne!(RetryReason::RateLimited, RetryReason::Transport);
    }
}
