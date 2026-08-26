//! Sessions, managed for you.
//!
//! A [`Session`] is `&mut` because one CM connection carries one request at a
//! time: it allocates a job id, writes the frame, and reads until that job's
//! reply comes back. That is honest about what the code does, and it means a
//! caller who wants two downloads at once needs two sessions.
//!
//! Which is fine — it is just not something anyone should have to think about.
//! This hands them out and takes them back:
//!
//! ```no_run
//! # async fn example() -> Result<(), tapline::InstallError> {
//! use tapline::SessionPool;
//!
//! // No session in sight. One is created, used, and kept for the next caller.
//! let mut session = SessionPool::shared().acquire().await?;
//! let info = session.app_info(tapline::AppId(4020)).await?;
//! # let _ = info;
//! # Ok(())
//! # }
//! ```
//!
//! Concurrent callers get different sessions and never wait on each other. They
//! still share one chunk budget and one connection pool through [`Shared`], so
//! spreading work across sessions does not multiply the load on Steam's CDN.
//!
//! # Why sessions are kept rather than reconnected
//!
//! Logging on costs a WebSocket, a TLS handshake and a round trip. Measured
//! with sixteen concurrent downloads it is cheap — 16/16 in 1.9 s — but paying
//! it per operation when the last one just finished is waste, and an idle
//! session is a socket and a few kilobytes.
//!
//! # Why they do not live forever
//!
//! Steam drops a session that stops heartbeating and does not say why; the
//! failure surfaces much later as an unrelated request returning
//! "disconnected". An idle session in a pool is exactly that case, so a keeper
//! task heartbeats what is idle and discards anything that has gone quiet for
//! too long to trust.
//!
//! [`Shared`]: crate::Shared

use crate::{InstallError, Session, Shared};
use std::sync::{Arc, Mutex};

/// How often the keeper heartbeats idle sessions.
///
/// Steam asks for roughly nine seconds. Three is comfortably inside that even
/// if the keeper is late, and an idle heartbeat is one small frame.
const KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(3);

/// How long an unused session may sit before it is thrown away rather than
/// trusted.
///
/// Not because it is certainly dead, but because a session that has been idle
/// this long has probably missed heartbeats, and finding out costs a failed
/// operation much later rather than a reconnect now.
const MAX_IDLE: std::time::Duration = std::time::Duration::from_secs(120);

/// A session waiting to be used again.
struct Idle {
    session: Session,
    since: std::time::Instant,
}

/// Hands out sessions and takes them back.
pub struct SessionPool {
    idle: Mutex<Vec<Idle>>,
    shared: Arc<Shared>,
    /// How many to keep. Beyond this, a returned session is dropped.
    max_idle: usize,
    /// Whether the keeper task has been started.
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
    ///
    /// What everything uses unless told otherwise, so two parts of a program
    /// that never met still share sessions, the chunk budget and the connection
    /// pool.
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
    ///
    /// Named `budget` rather than `shared` because [`SessionPool::shared`] is
    /// the process-wide pool, and two things called `shared` on one type is one
    /// too many.
    #[must_use]
    pub fn budget(&self) -> &Arc<Shared> {
        &self.shared
    }

    /// Takes a session, connecting one if none is waiting.
    ///
    /// The returned guard puts it back when dropped, so a caller who does
    /// nothing special gets pooling for free.
    pub async fn acquire(self: &Arc<Self>) -> Result<SessionGuard, InstallError> {
        self.start_keeper();

        if let Some(session) = self.take_fresh() {
            return Ok(SessionGuard {
                session: Some(session),
                pool: Arc::clone(self),
                healthy: true,
            });
        }

        let session = Session::anonymous_shared(Arc::clone(&self.shared)).await?;
        Ok(SessionGuard {
            session: Some(session),
            pool: Arc::clone(self),
            healthy: true,
        })
    }

    /// Pops a session that has not been idle too long.
    fn take_fresh(&self) -> Option<Session> {
        let mut idle = self.idle.lock().ok()?;
        while let Some(entry) = idle.pop() {
            if entry.since.elapsed() < MAX_IDLE {
                return Some(entry.session);
            }
            // Too old to trust. Dropped, and the next one tried.
        }
        None
    }

    /// Takes a session back.
    fn give_back(&self, session: Session) {
        let Ok(mut idle) = self.idle.lock() else {
            return;
        };
        if idle.len() >= self.max_idle {
            return;
        }
        idle.push(Idle {
            session,
            since: std::time::Instant::now(),
        });
    }

    /// Starts the keeper, once.
    fn start_keeper(self: &Arc<Self>) {
        use std::sync::atomic::Ordering;
        if self.keeper.swap(true, Ordering::SeqCst) {
            return;
        }
        // A weak reference: the keeper must not be the reason the pool lives
        // forever, and a process-wide pool lives forever anyway.
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

    /// Heartbeats every idle session, discarding any that refuses.
    async fn keep_alive(&self) {
        // Taken out entirely rather than held under the lock across an await:
        // the lock is a std one, and a caller acquiring a session must not wait
        // on the network.
        let taken: Vec<Idle> = match self.idle.lock() {
            Ok(mut idle) => std::mem::take(&mut idle),
            Err(_) => return,
        };

        let mut keep = Vec::with_capacity(taken.len());
        for mut entry in taken {
            if entry.since.elapsed() >= MAX_IDLE {
                continue;
            }
            if entry.session.keep_alive().await.is_ok() {
                keep.push(entry);
            }
            // A session that would not heartbeat is one Steam has already
            // dropped, or is about to. Letting it go costs a reconnect; keeping
            // it costs a confusing failure in whatever picks it up next.
        }

        if let Ok(mut idle) = self.idle.lock() {
            // Anything returned while the keeper was working stays.
            idle.extend(keep);
            let max = self.max_idle;
            if idle.len() > max {
                idle.truncate(max);
            }
        }
    }
}

/// A session borrowed from a pool.
///
/// Derefs to the session, and returns it when dropped.
pub struct SessionGuard {
    session: Option<Session>,
    pool: Arc<SessionPool>,
    healthy: bool,
}

impl SessionGuard {
    /// Marks the session as not worth reusing.
    ///
    /// Call this when an operation failed in a way that suggests the connection
    /// is gone. The session is dropped instead of pooled, so the next caller
    /// does not inherit the problem.
    pub const fn poison(&mut self) {
        self.healthy = false;
    }

    /// Takes the session out, leaving the pool with nothing to reclaim.
    ///
    /// For a caller who wants to own it — the manual path.
    #[must_use]
    pub fn into_inner(mut self) -> Option<Session> {
        self.session.take()
    }
}

// The option is `None` only after `into_inner`, which consumes the guard, and
// inside `Drop`. Neither can be observed through these, so the expect is
// unreachable by construction — and an unreachable expect is better than an
// `abort` that would take someone's server down for a bug in this file.
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
        // Two parts of a program that never met must land on the same sessions,
        // the same chunk budget and the same connection pool.
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
