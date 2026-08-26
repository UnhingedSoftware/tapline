//! The IO seam.
//!
//! Every crate above the leaves consumes these traits and none of them
//! constructs one. That buys two things worth more than the indirection costs:
//!
//! * **The protocol is testable without a network.** A recorded CM exchange
//!   replayed through an in-memory [`Transport`] exercises the framing and the
//!   message routing with no socket, no Steam account and no flakiness.
//! * **`tokio` is a dependency of one crate.** A service that already runs its
//!   own reactor links `tapline` without inheriting ours.
//!
//! The traits use `impl Future` rather than `async fn` in trait so that `Send`
//! bounds are explicit: everything here is driven from a work-stealing runtime,
//! and a future that is accidentally `!Send` fails to spawn at the call site
//! rather than here.
//!
//! There is deliberately no `dyn`-compatible variant. Concrete types at the
//! seams keep monomorphisation shallow, which is a compile-time decision as much
//! as a performance one.

mod fetch;
#[cfg(feature = "testing")]
pub mod testing;

pub use fetch::{Fetch, FetchError, Request, Response};

use std::future::Future;
use std::io;
use std::time::{Duration, SystemTime};

/// The CM connection, as a sequence of whole messages.
///
/// Message-oriented rather than byte-oriented, because that is what the
/// transport actually is. Measured 2026-08-26:
/// `ISteamDirectory/GetCMListForConnect` offers 52 `websockets` servers, 6
/// `netfilter`, and no TCP at all, and Steam puts exactly one protocol message
/// in each WebSocket binary frame. A byte-stream trait would force the
/// implementation to throw the frame boundaries away and the caller to invent a
/// length prefix to get them back.
///
/// So there is no `read_exact` here. There is nothing to length-prefix: the
/// frame is the message.
pub trait Transport: Send {
    /// Sends one whole message.
    fn send(&mut self, message: &[u8]) -> impl Future<Output = io::Result<()>> + Send;

    /// Receives the next whole message.
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] when the peer closes cleanly, so
    /// a disconnect is an error the caller handles rather than an empty message
    /// it might mistake for a real one.
    fn recv(&mut self) -> impl Future<Output = io::Result<Vec<u8>>> + Send;

    /// Closes the connection.
    fn close(&mut self) -> impl Future<Output = io::Result<()>> + Send;
}

/// A destination for downloaded content: a file being filled at arbitrary
/// offsets.
///
/// Takes `&self`, not `&mut self`, and that is the whole design. Chunks arrive
/// from many tasks at once and land at unrelated offsets, so serialising them
/// behind a single cursor would throw away the parallelism the CDN pool exists
/// to create. Implementations are expected to use positional writes.
pub trait Sink: Send + Sync {
    /// Writes `data` at `offset`.
    fn write_at(&self, offset: u64, data: &[u8]) -> impl Future<Output = io::Result<()>> + Send;

    /// Sets the file's length, allocating space up front.
    ///
    /// Called once before any chunk is written. Allocating the whole file first
    /// keeps it contiguous and means a full disk is discovered before the
    /// download rather than 90% through it.
    fn allocate(&self, len: u64) -> impl Future<Output = io::Result<()>> + Send;

    /// Flushes to durable storage.
    fn sync(&self) -> impl Future<Output = io::Result<()>> + Send;
}

/// Time, as a capability.
///
/// Rate-limit backoff and token expiry both depend on the clock, and neither is
/// pleasant to test against the real one.
pub trait Clock: Send + Sync {
    /// The current wall-clock time.
    ///
    /// Wall clock rather than a monotonic instant because the things it is
    /// compared against — JWT expiry, `LastUpdated` in an appmanifest — are
    /// absolute times that outlive the process.
    fn now(&self) -> SystemTime;

    /// Waits for `duration`.
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send;
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::testing::{MemoryTransport, block_on};

    #[test]
    fn a_memory_backed_transport_satisfies_the_trait() {
        // The point of this test is that the trait is implementable by something
        // with no runtime behind it — that is what makes the protocol testable
        // without a network.
        let mut transport = MemoryTransport::new(vec![b"hello".to_vec()]);

        block_on(async {
            let message = transport.recv().await.expect("must receive");
            assert_eq!(message, b"hello");

            transport.send(b"reply").await.expect("must send");
            assert_eq!(transport.sent(), &[b"reply".to_vec()]);

            // Reading past the end reports EOF rather than an empty message.
            let error = transport.recv().await.expect_err("must report EOF");
            assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
        });
    }
}
