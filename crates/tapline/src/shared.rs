//! Resources that concurrent downloads should share rather than duplicate.
//!
//! A process installing three apps at once used to run three of everything: three
//! connection pools, and three independent budgets of 64 chunks in flight, for
//! 192 concurrent requests against Steam's CDN.
//!
//! That is not three times the throughput. Measured on Garry's Mod, a single
//! download at increasing concurrency:
//!
//! | chunks in flight | throughput |
//! |---|---|
//! | 32 | 168 MB/s |
//! | 64 | 184 MB/s |
//! | 128 | 163 MB/s |
//! | 256 | 157 MB/s |
//!
//! The curve turns over after 64. Two downloads at 64 each land where one
//! download at 128 lands — slower than either would be alone — and they do it
//! while also paying for a second connection pool whose warm sockets the other
//! download cannot use.
//!
//! So the budget belongs to the process, not to the download. [`Shared`] holds
//! it, along with the HTTP connection pool, and every [`Session`] built on the
//! same one draws from it. Three downloads then split 64 rather than
//! multiplying it, and a connection one of them warmed is one the others can
//! use.
//!
//! Measured, three concurrent Valheim installs (4.40 GB total) on one link:
//!
//! | total budget | wall clock | throughput | spread between finishes |
//! |---|---|---|---|
//! | 64 (shared) | 18.3–21.3 s | 197–230 MB/s | 0.9 s |
//! | 96 | 18.5–19.3 s | 218–227 MB/s | — |
//! | 128 | 22.9 s | 184 MB/s | — |
//! | 192 (a full budget each) | 24.7 s | 170 MB/s | 6.7 s |
//!
//! 64 and 96 are within each other's run-to-run variance, which is wide here —
//! 64 gave both 197 and 230 MB/s. What is not within variance is the top and
//! the bottom: three downloads that each take 64 land at 170 MB/s and finish
//! nearly seven seconds apart, and the same three sharing 64 land above 200 and
//! finish within a second of each other. The gain is from sharing, not from
//! picking a different number, which is why the default did not move.
//!
//! Three sharing 64 also beat a *single* download at 64, which peaks around
//! 184 MB/s. One download cannot keep 64 requests busy: it stalls on its own
//! per-file and per-depot ordering. Another download's chunks fill those gaps.
//!
//! [`Session`]: crate::Session
//!
//! ```no_run
//! # async fn example() -> Result<(), tapline::InstallError> {
//! use tapline::{Session, Shared};
//!
//! let shared = Shared::new(64);
//! let a = Session::anonymous_shared(shared.clone()).await?;
//! let b = Session::anonymous_shared(shared).await?;
//! // `a` and `b` now compete for one budget instead of each taking a full one.
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;
use tapline_rt_tokio::HttpClient;

/// What concurrent downloads share.
pub struct Shared {
    /// The connection pool. Per-host inside, so a host warmed by one download
    /// is warm for the next.
    pub(crate) http: Arc<HttpClient>,
    /// The process's total chunk budget.
    pub(crate) limit: Arc<tokio::sync::Semaphore>,
    /// What that budget was created with, for [`Shared::concurrency`].
    permits: usize,
}

impl std::fmt::Debug for Shared {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The connection pool has no Debug and would be noise if it did: what a
        // reader wants here is how much budget there is and how much is left.
        f.debug_struct("Shared")
            .field("concurrency", &self.permits)
            .field("available", &self.available())
            .finish_non_exhaustive()
    }
}

impl Shared {
    /// A budget of `concurrency` chunks in flight, shared by every session
    /// built on it.
    ///
    /// 64 is where the measurements stop paying; see the module docs.
    #[must_use]
    pub fn new(concurrency: usize) -> Arc<Self> {
        let permits = concurrency.max(1);
        Arc::new(Self {
            http: Arc::new(HttpClient::new()),
            limit: Arc::new(tokio::sync::Semaphore::new(permits)),
            permits,
        })
    }

    /// The total number of chunks that may be in flight across all downloads.
    #[must_use]
    pub const fn concurrency(&self) -> usize {
        self.permits
    }

    /// How much of the budget is free right now.
    ///
    /// Only useful for reporting: by the time a caller reads it, it has moved.
    #[must_use]
    pub fn available(&self) -> usize {
        self.limit.available_permits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_budget_starts_fully_available() {
        let shared = Shared::new(64);
        assert_eq!(shared.concurrency(), 64);
        assert_eq!(shared.available(), 64);
    }

    #[test]
    fn a_budget_of_zero_still_lets_one_chunk_through() {
        // Zero would deadlock every download rather than fail loudly, which is
        // the worse of the two.
        let shared = Shared::new(0);
        assert_eq!(shared.concurrency(), 1);
        assert_eq!(shared.available(), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn two_holders_draw_from_the_same_budget() {
        // The property the whole module exists for: a second download does not
        // get its own 64, it shares the one that is there.
        let shared = Shared::new(2);
        let first = Arc::clone(&shared.limit).acquire_owned().await;
        assert!(first.is_ok());
        assert_eq!(shared.available(), 1);

        let second = Arc::clone(&shared.limit).acquire_owned().await;
        assert!(second.is_ok());
        assert_eq!(shared.available(), 0);

        drop(first);
        assert_eq!(shared.available(), 1);
    }

    #[test]
    fn sessions_sharing_one_budget_share_one_connection_pool() {
        // Warm sockets are the other half: a host one download opened is a host
        // the next does not have to handshake with.
        let shared = Shared::new(8);
        let a = Arc::clone(&shared.http);
        let b = Arc::clone(&shared.http);
        assert!(Arc::ptr_eq(&a, &b));
    }
}
