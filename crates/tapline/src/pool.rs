//! Sessions, managed for you.

use crate::{InstallError, Session, Shared};
use std::sync::{Arc, Mutex};

/// Steam wants a heartbeat roughly every nine seconds; three keeps comfortably inside that.
const KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(3);

/// An idle session this old has probably missed heartbeats; reconnecting now is cheaper.
const MAX_IDLE: std::time::Duration = std::time::Duration::from_secs(120);

struct Idle {
    session: Session,
    since: std::time::Instant,
    /// Sessions own sockets/timers of their creating runtime; using them elsewhere fails obscurely.
    runtime: tokio::runtime::Id,
}

/// Hands out sessions and takes them back.
pub struct SessionPool {
    idle: Mutex<Vec<Idle>>,
    shared: Arc<Shared>,
    max_idle: usize,
    keeper: std::sync::atomic::AtomicBool,
}

impl std::fmt::Debug for SessionPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionPool")
            .field("idle", &self.idle_count())
            .field("max_idle", &self.max_idle)
            .finish_non_exhaustive()
    }
}

impl SessionPool {
    /// The process-wide pool.
    #[must_use]
    pub fn shared() -> &'static Arc<Self> {
        static POOL: std::sync::OnceLock<Arc<SessionPool>> = std::sync::OnceLock::new();
        POOL.get_or_init(|| {
            Arc::new(SessionPool::with_shared(Shared::new(
                crate::InstallOptions::default().concurrency,
            )))
        })
    }

    /// A pool of sessions drawing on `shared`.
    #[must_use]
    pub fn with_shared(shared: Arc<Shared>) -> Self {
        Self {
            idle: Mutex::new(Vec::new()),
            shared,
            max_idle: 8,
            keeper: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// How many sessions are waiting to be used.
    #[must_use]
    pub fn idle_count(&self) -> usize {
        self.idle.lock().map(|idle| idle.len()).unwrap_or(0)
    }

    /// The resources every session here shares.
    #[must_use]
    pub fn budget(&self) -> &Arc<Shared> {
        &self.shared
    }

    /// Takes a session, connecting one if none is waiting.
    pub async fn acquire(self: &Arc<Self>) -> Result<SessionGuard, InstallError> {
        self.start_keeper();

        if let Some(session) = self.take_fresh() {
            return Ok(SessionGuard {
                session: Some(session),
                pool: Arc::clone(self),
                healthy: true,
            });
        }

        // Signed in when this machine has a saved token, anonymous otherwise.
        let session = Session::automatic_shared(None, Arc::clone(&self.shared)).await?;
        Ok(SessionGuard {
            session: Some(session),
            pool: Arc::clone(self),
            healthy: true,
        })
    }

    fn take_fresh(&self) -> Option<Session> {
        let here = tokio::runtime::Handle::try_current().ok()?.id();
        let mut idle = self.idle.lock().ok()?;

        // Searched, not popped: the newest entry may belong to another runtime.
        let found = idle
            .iter()
            .rposition(|entry| entry.runtime == here && entry.since.elapsed() < MAX_IDLE)?;
        let entry = idle.remove(found);

        idle.retain(|entry| entry.since.elapsed() < MAX_IDLE);
        Some(entry.session)
    }

    fn give_back(&self, session: Session) {
        // No runtime to attribute it to means it cannot be safely reused.
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let Ok(mut idle) = self.idle.lock() else {
            return;
        };
        if idle.len() >= self.max_idle {
            return;
        }
        idle.push(Idle {
            session,
            since: std::time::Instant::now(),
            runtime: handle.id(),
        });
    }

    fn start_keeper(self: &Arc<Self>) {
        use std::sync::atomic::Ordering;
        if self.keeper.swap(true, Ordering::SeqCst) {
            return;
        }
        // Weak: the keeper must not keep the pool alive.
        let weak = Arc::downgrade(self);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(KEEPALIVE).await;
                let Some(pool) = weak.upgrade() else {
                    return;
                };
                pool.keep_alive().await;
            }
        });
    }

    async fn keep_alive(&self) {
        // Taken out whole: a std lock must not be held across an await.
        let taken: Vec<Idle> = match self.idle.lock() {
            Ok(mut idle) => std::mem::take(&mut idle),
            Err(_) => return,
        };

        let here = tokio::runtime::Handle::try_current()
            .ok()
            .map(|handle| handle.id());
        let mut keep = Vec::with_capacity(taken.len());
        for mut entry in taken {
            if entry.since.elapsed() >= MAX_IDLE {
                continue;
            }
            // Only heartbeat sessions belonging to this runtime.
            if here != Some(entry.runtime) {
                keep.push(entry);
                continue;
            }
            if entry.session.keep_alive().await.is_ok() {
                keep.push(entry);
            }
        }

        if let Ok(mut idle) = self.idle.lock() {
            idle.extend(keep);
            let max = self.max_idle;
            if idle.len() > max {
                idle.truncate(max);
            }
        }
    }
}

/// A session borrowed from a pool; derefs to it and returns it on drop.
pub struct SessionGuard {
    session: Option<Session>,
    pool: Arc<SessionPool>,
    healthy: bool,
}

impl SessionGuard {
    /// Marks the session as not worth reusing.
    pub const fn poison(&mut self) {
        self.healthy = false;
    }

    /// Takes the session out, leaving the pool with nothing to reclaim.
    #[must_use]
    pub fn into_inner(mut self) -> Option<Session> {
        self.session.take()
    }
}

// `session` is `None` only after `into_inner` or in `Drop`; the expect is unreachable.
#[allow(clippy::expect_used)]
impl std::ops::Deref for SessionGuard {
    type Target = Session;

    fn deref(&self) -> &Self::Target {
        self.session
            .as_ref()
            .expect("a guard is only emptied by into_inner, which consumes it")
    }
}

#[allow(clippy::expect_used)]
impl std::ops::DerefMut for SessionGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.session
            .as_mut()
            .expect("a guard is only emptied by into_inner, which consumes it")
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        if let (Some(session), true) = (self.session.take(), self.healthy) {
            self.pool.give_back(session);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_pool_has_nothing_waiting() {
        let pool = SessionPool::with_shared(Shared::new(4));
        assert_eq!(pool.idle_count(), 0);
    }

    #[test]
    fn the_shared_pool_is_one_pool() {
        assert!(Arc::ptr_eq(SessionPool::shared(), SessionPool::shared()));
    }

    #[test]
    fn the_shared_pool_carries_the_default_budget() {
        assert_eq!(
            SessionPool::shared().budget().concurrency(),
            crate::InstallOptions::default().concurrency
        );
    }

    #[test]
    fn a_pool_reports_what_it_is_holding() {
        let pool = SessionPool::with_shared(Shared::new(4));
        let text = format!("{pool:?}");
        assert!(text.contains("idle"), "{text}");
        assert!(text.contains("max_idle"), "{text}");
    }
}
