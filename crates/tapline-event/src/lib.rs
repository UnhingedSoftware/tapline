use tapline_ids::{AppId, DepotId, ManifestId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Plan {
    pub download_bytes: u64,
    pub reused_bytes: u64,
    pub total_bytes: u64,
    pub file_count: u64,
    pub chunk_count: u64,
}

impl Plan {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryReason {
    Transport,
    RateLimited,
    IntegrityFailure,
    Status(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Event {
    Planned {
        plan: Plan,
    },
    DepotStarted {
        depot: DepotId,
        manifest: ManifestId,
        bytes: u64,
    },
    DepotCompleted {
        depot: DepotId,
    },
    Progress {
        bytes_done: u64,
        bytes_total: u64,
    },
    FileCompleted {
        path: String,
        bytes: u64,
    },
    Retrying {
        host: String,
        reason: RetryReason,
        attempt: u32,
    },
    Extended {
        extension: String,
        path: String,
        produced: u64,
    },
    Verifying {
        path: String,
    },
    Completed {
        app: AppId,
        downloaded_bytes: u64,
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
