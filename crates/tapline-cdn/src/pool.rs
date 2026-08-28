use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Host {
    pub host: String,
    pub vhost: String,
    pub load: u32,
    pub https_required: bool,
}

#[must_use]
pub fn usable_over_tls(https_support: Option<&str>) -> bool {
    !matches!(https_support, Some("unavailable"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolError {
    Empty,
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

const FAILURE_LIMIT: u32 = 3;

#[derive(Debug, Clone)]
pub struct HostPool {
    hosts: Vec<Host>,
    failures: HashMap<String, u32>,
    next: usize,
}

impl HostPool {
    #[must_use]
    pub fn new(mut hosts: Vec<Host>) -> Self {
        hosts.sort_by_key(|host| host.load);
        Self {
            hosts,
            failures: HashMap::new(),
            next: 0,
        }
    }

    #[must_use]
    pub fn healthy(&self) -> usize {
        self.hosts
            .iter()
            .filter(|host| self.failures.get(&host.host).copied().unwrap_or(0) < FAILURE_LIMIT)
            .count()
    }

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

    pub fn demote(&mut self, host: &str) {
        *self.failures.entry(host.to_owned()).or_insert(0) += 1;
    }

    pub fn succeed(&mut self, host: &str) {
        if let Some(count) = self.failures.get_mut(host) {
            *count = count.saturating_sub(1);
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> Vec<String> {
        self.hosts
            .iter()
            .filter(|host| !self.is_demoted(&host.host))
            .map(|host| host.host.clone())
            .collect()
    }

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
        assert!(usable_over_tls(None));
        assert!(usable_over_tls(Some("something-new")));
    }
}
