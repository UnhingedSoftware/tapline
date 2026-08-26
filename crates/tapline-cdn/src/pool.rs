//! Choosing which CDN host to ask.
//!
//! Steam offers a dozen or so hosts per cell and reports a load figure for each.
//! The pool spreads work across them and stops asking one that has started
//! refusing — which is as much about not being throttled as about resilience.
//! A download that hammers a single host is a download that gets rate-limited,
//! and being fast must never look like abuse.

use std::collections::HashMap;
use std::fmt;

/// One CDN host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Host {
    /// The hostname to connect to.
    pub host: String,
    /// The `Host:` header value, when it differs — a lancache serves many
    /// origins from one address and needs the vhost to know what was asked for.
    pub vhost: String,
    /// Steam's load figure, lower being better.
    pub load: u32,
    /// Whether Steam says TLS is mandatory for this host.
    ///
    /// Measured 2026-08-26: every host Steam returned said `mandatory`. A host
    /// that says otherwise is almost certainly a local cache.
    pub https_required: bool,
}

/// Whether a host Steam offered can be used, given that every fetch is TLS.
///
/// Steam's directory mixes its own caches with third-party CDN entries, and one
/// of those declares `https_support: "unavailable"`. It is not a hostname with a
/// missing certificate — it is a hostname that resolves into CloudFront,
/// CacheFly or Alibaba depending on the lookup, none of which present a
/// certificate for `steamcontent.com`. Connecting to it over TLS fails
/// verification, correctly, and takes the whole install with it.
///
/// Asking Steam for 20 servers never surfaced this. Asking for 100 did.
///
/// Such a host is skipped rather than fetched over plaintext. Chunk integrity
/// would survive the downgrade — every chunk is checked against the SHA-1 that
/// names it — but this is one host out of eighty-three, and a plaintext path
/// that exists for a 1.2% widening of the pool is a plaintext path that will
/// eventually be used for something else.
#[must_use]
pub fn usable_over_tls(https_support: Option<&str>) -> bool {
    // Absent means an older directory response that predates the field; those
    // hosts spoke TLS fine, so absence is not a reason to drop one.
    !matches!(https_support, Some("unavailable"))
}

/// Why the pool could not serve a host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolError {
    /// The pool was constructed with no hosts.
    Empty,
    /// Every host has failed too many times.
    AllDemoted,
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the CDN pool has no hosts"),
            Self::AllDemoted => f.write_str("every CDN host has been demoted"),
        }
    }
}

impl std::error::Error for PoolError {}

/// How many failures retire a host for the rest of the download.
///
/// Low on purpose. A host failing repeatedly is either broken or rate-limiting
/// us, and both are answered by going elsewhere rather than by trying harder.
const FAILURE_LIMIT: u32 = 3;

/// A rotating set of CDN hosts.
#[derive(Debug, Clone)]
pub struct HostPool {
    hosts: Vec<Host>,
    failures: HashMap<String, u32>,
    next: usize,
}

impl HostPool {
    /// Builds a pool, best host first.
    ///
    /// Hosts that require TLS are kept as-is; the caller decides scheme. Order
    /// is by Steam's load figure, so the head of the list is the host Steam
    /// thinks is least busy.
    #[must_use]
    pub fn new(mut hosts: Vec<Host>) -> Self {
        hosts.sort_by_key(|host| host.load);
        Self {
            hosts,
            failures: HashMap::new(),
            next: 0,
        }
    }

    /// How many hosts are still usable.
    #[must_use]
    pub fn healthy(&self) -> usize {
        self.hosts
            .iter()
            .filter(|host| self.failures.get(&host.host).copied().unwrap_or(0) < FAILURE_LIMIT)
            .count()
    }

    /// The next host to use.
    ///
    /// Round-robins rather than always returning the best one: concentrating a
    /// depot's tens of thousands of requests on a single host is what triggers
    /// rate limiting in the first place.
    pub fn acquire(&mut self) -> Result<Host, PoolError> {
        if self.hosts.is_empty() {
            return Err(PoolError::Empty);
        }

        for _ in 0..self.hosts.len() {
            let index = self.next % self.hosts.len();
            self.next = self.next.wrapping_add(1);

            if let Some(host) = self.hosts.get(index) {
                let failures = self.failures.get(&host.host).copied().unwrap_or(0);
                if failures < FAILURE_LIMIT {
                    return Ok(host.clone());
                }
            }
        }
        Err(PoolError::AllDemoted)
    }

    /// Records that a host failed.
    ///
    /// Three strikes and it is out for the rest of the download.
    pub fn demote(&mut self, host: &str) {
        *self.failures.entry(host.to_owned()).or_insert(0) += 1;
    }

    /// Records that a host succeeded, forgiving one earlier failure.
    ///
    /// A single timeout on an otherwise healthy host should not retire it over
    /// the course of a long download.
    pub fn succeed(&mut self, host: &str) {
        if let Some(count) = self.failures.get_mut(host) {
            *count = count.saturating_sub(1);
        }
    }

    /// The hostnames still worth using, best first.
    ///
    /// Taken once before a download and handed to the fetch tasks, so they need
    /// no lock on the pool to pick a host. Health is folded back afterwards.
    #[must_use]
    pub fn snapshot(&self) -> Vec<String> {
        self.hosts
            .iter()
            .filter(|host| !self.is_demoted(&host.host))
            .map(|host| host.host.clone())
            .collect()
    }

    /// Whether a host has been retired.
    #[must_use]
    pub fn is_demoted(&self, host: &str) -> bool {
        self.failures.get(host).copied().unwrap_or(0) >= FAILURE_LIMIT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(name: &str, load: u32) -> Host {
        Host {
            host: name.to_owned(),
            vhost: name.to_owned(),
            load,
            https_required: true,
        }
    }

    fn pool() -> HostPool {
        HostPool::new(vec![
            host("cache12.invalid", 25),
            host("cache8.invalid", 24),
            host("cache1.invalid", 30),
        ])
    }

    #[test]
    fn the_least_loaded_host_comes_first() {
        let mut pool = pool();
        assert_eq!(pool.acquire().expect("a host").host, "cache8.invalid");
    }

    #[test]
    fn requests_rotate_rather_than_piling_onto_one_host() {
        // Concentrating a depot's tens of thousands of requests on the best
        // host is what gets a download rate-limited.
        let mut pool = pool();
        let picked: Vec<String> = (0..3)
            .filter_map(|_| pool.acquire().ok())
            .map(|host| host.host)
            .collect();

        assert_eq!(picked.len(), 3);
        let distinct: std::collections::HashSet<_> = picked.iter().collect();
        assert_eq!(
            distinct.len(),
            3,
            "the pool returned the same host repeatedly"
        );
    }

    #[test]
    fn a_failing_host_is_retired_after_three_strikes() {
        let mut pool = pool();
        for _ in 0..FAILURE_LIMIT {
            pool.demote("cache8.invalid");
        }
        assert!(pool.is_demoted("cache8.invalid"));
        assert_eq!(pool.healthy(), 2);

        for _ in 0..6 {
            let picked = pool.acquire().expect("a host");
            assert_ne!(picked.host, "cache8.invalid", "a retired host came back");
        }
    }

    #[test]
    fn a_success_forgives_an_earlier_failure() {
        // One timeout on an otherwise healthy host should not retire it over a
        // long download.
        let mut pool = pool();
        pool.demote("cache8.invalid");
        pool.demote("cache8.invalid");
        pool.succeed("cache8.invalid");
        pool.demote("cache8.invalid");

        assert!(!pool.is_demoted("cache8.invalid"));
    }

    #[test]
    fn exhausting_the_pool_is_reported_rather_than_looping() {
        let mut pool = pool();
        for name in ["cache8.invalid", "cache12.invalid", "cache1.invalid"] {
            for _ in 0..FAILURE_LIMIT {
                pool.demote(name);
            }
        }
        assert_eq!(pool.healthy(), 0);
        assert_eq!(pool.acquire(), Err(PoolError::AllDemoted));
    }

    #[test]
    fn an_empty_pool_is_reported_as_empty() {
        assert_eq!(HostPool::new(Vec::new()).acquire(), Err(PoolError::Empty));
    }

    #[test]
    fn a_host_without_https_is_not_offered_to_a_tls_only_client() {
        assert!(!usable_over_tls(Some("unavailable")));
    }

    #[test]
    fn https_hosts_and_older_responses_are_kept() {
        assert!(usable_over_tls(Some("mandatory")));
        assert!(usable_over_tls(Some("optional")));
        // A field Steam has not sent is not a refusal.
        assert!(usable_over_tls(None));
        // An unrecognised value is not assumed hostile: the connection either
        // verifies or it does not, and that check is the real gate.
        assert!(usable_over_tls(Some("something-new")));
    }
}
