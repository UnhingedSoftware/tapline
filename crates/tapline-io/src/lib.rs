//! The IO seam: protocol crates consume these traits, only the leaves implement them.

mod fetch;
#[cfg(feature = "testing")]
pub mod testing;

pub use fetch::{Fetch, FetchError, Request, Response};

use std::future::Future;
use std::io;
use std::time::{Duration, SystemTime};

/// The CM connection as whole messages; one protocol message per WebSocket frame.
pub trait Transport: Send {
    /// Sends one whole message.
    fn send(&mut self, message: &[u8]) -> impl Future<Output = io::Result<()>> + Send;

    /// Receives the next whole message; a clean close is `UnexpectedEof`, not empty.
    fn recv(&mut self) -> impl Future<Output = io::Result<Vec<u8>>> + Send;

    /// Closes the connection.
    fn close(&mut self) -> impl Future<Output = io::Result<()>> + Send;
}

/// A file filled at arbitrary offsets; `&self` because chunks land concurrently.
pub trait Sink: Send + Sync {
    /// Writes `data` at `offset`.
    fn write_at(&self, offset: u64, data: &[u8]) -> impl Future<Output = io::Result<()>> + Send;

    /// Sets the file's length up front, so a full disk fails early.
    fn allocate(&self, len: u64) -> impl Future<Output = io::Result<()>> + Send;

    /// Flushes to durable storage.
    fn sync(&self) -> impl Future<Output = io::Result<()>> + Send;
}

/// Time, as a capability, so backoff and expiry are testable.
pub trait Clock: Send + Sync {
    /// The current wall-clock time; compared against absolute times like JWT expiry.
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
