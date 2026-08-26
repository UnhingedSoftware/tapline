//! The IO seam.
//!
//! Every crate above the leaves consumes these traits and none of them
//! constructs one. That buys two things worth more than the indirection costs:
//!
//! * **The protocol is testable without a network.** A recorded CM handshake
//!   replayed through an in-memory [`Stream`] exercises the framing and the
//!   channel crypto with no socket, no Steam account and no flakiness.
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

/// A bidirectional byte stream: the CM connection.
///
/// Deliberately narrower than `AsyncRead + AsyncWrite`. The CM protocol is
/// framed — a length prefix, then exactly that many bytes — so the connection
/// only ever needs "fill this buffer" and "send these bytes", and a narrower
/// trait is a smaller thing to implement for a test double.
pub trait Stream: Send {
    /// Reads exactly enough bytes to fill `buf`.
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] if the peer closes first. Short
    /// reads are handled by the implementation rather than the caller, because
    /// every caller in this workspace wants a whole frame and a partial one is
    /// never useful.
    fn read_exact(&mut self, buf: &mut [u8]) -> impl Future<Output = io::Result<()>> + Send;

    /// Writes all of `buf`.
    fn write_all(&mut self, buf: &[u8]) -> impl Future<Output = io::Result<()>> + Send;

    /// Flushes anything buffered and closes the write half.
    fn shutdown(&mut self) -> impl Future<Output = io::Result<()>> + Send;
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
    use crate::testing::{MemoryStream, block_on};

    #[test]
    fn a_memory_backed_stream_satisfies_the_trait() {
        // The point of this test is that the trait is implementable by something
        // with no runtime behind it — that is what makes the protocol testable
        // without a network.
        let mut stream = MemoryStream::new(vec![1, 2, 3, 4]);

        block_on(async {
            let mut buf = [0_u8; 2];
            stream.read_exact(&mut buf).await.expect("must read");
            assert_eq!(buf, [1, 2]);

            stream.write_all(&[9, 9]).await.expect("must write");
            assert_eq!(stream.written(), &[9, 9]);

            // Reading past the end reports EOF rather than a short read.
            let mut too_much = [0_u8; 4];
            let err = stream
                .read_exact(&mut too_much)
                .await
                .expect_err("must refuse to over-read");
            assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
        });
    }
}
