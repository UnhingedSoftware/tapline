//! Resources that concurrent downloads should share rather than duplicate.

use std::sync::Arc;
use tapline_rt_tokio::HttpClient;

/// What concurrent downloads share.
pub struct Shared {
    pub(crate) http: Arc<HttpClient>,
    pub(crate) limit: Arc<tokio::sync::Semaphore>,
    permits: usize,
}

impl std::fmt::Debug for Shared {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shared")
            .field("concurrency", &self.permits)
            .field("available", &self.available())
            .finish_non_exhaustive()
    }
}

impl Shared {
    /// A budget of `concurrency` chunks in flight, shared by every session built on it.
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
        // Zero would deadlock every download rather than fail loudly.
        let shared = Shared::new(0);
        assert_eq!(shared.concurrency(), 1);
        assert_eq!(shared.available(), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn two_holders_draw_from_the_same_budget() {
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
        let shared = Shared::new(8);
        let a = Arc::clone(&shared.http);
        let b = Arc::clone(&shared.http);
        assert!(Arc::ptr_eq(&a, &b));
    }
}
