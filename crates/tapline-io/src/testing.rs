//! Test doubles for the IO traits, and a runtime-free executor to drive them.
//!
//! Behind the `testing` feature so it never reaches a production build. Every
//! crate that speaks the protocol uses these to test against recorded bytes
//! instead of a live Steam connection: no account, no network, no flakiness, and
//! a failing test points at our framing rather than at Valve having a bad day.

use crate::{Sink, Transport};
use std::future::Future;
use std::io;
use std::sync::Mutex;
use std::task::{Context, Poll, Waker};

/// Drives a future to completion on the calling thread.
///
/// Adequate precisely because these doubles never suspend: there is no real IO
/// behind them, so every future is ready on its first poll. A double that did
/// suspend would spin here, which is why none of them do.
pub fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    loop {
        if let Poll::Ready(out) = future.as_mut().poll(&mut cx) {
            return out;
        }
    }
}

/// A [`Transport`] that serves queued messages and records what was sent.
///
/// This is how a recorded CM exchange is replayed: load the messages Steam sent,
/// run the real protocol code against them, and assert on what it tried to send
/// back. No socket, no account, no flakiness — a failing test points at our
/// framing rather than at Valve having a bad day.
#[derive(Debug, Default)]
pub struct MemoryTransport {
    incoming: std::collections::VecDeque<Vec<u8>>,
    sent: Vec<Vec<u8>>,
    closed: bool,
}

impl MemoryTransport {
    /// A transport that will hand `incoming` to its reader, in order.
    #[must_use]
    pub fn new(incoming: Vec<Vec<u8>>) -> Self {
        Self {
            incoming: incoming.into(),
            sent: Vec::new(),
            closed: false,
        }
    }

    /// Every message sent so far, in order.
    #[must_use]
    pub fn sent(&self) -> &[Vec<u8>] {
        &self.sent
    }

    /// How many queued messages have not been read.
    ///
    /// A replay test asserting this is zero proves the code consumed the whole
    /// recording rather than stopping early and passing by accident.
    #[must_use]
    pub fn unread(&self) -> usize {
        self.incoming.len()
    }

    /// Whether the connection was closed.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    /// Queues another message for the reader, mid-test.
    pub fn push_incoming(&mut self, message: Vec<u8>) {
        self.incoming.push_back(message);
    }
}

impl Transport for MemoryTransport {
    async fn send(&mut self, message: &[u8]) -> io::Result<()> {
        if self.closed {
            return Err(io::Error::from(io::ErrorKind::BrokenPipe));
        }
        self.sent.push(message.to_vec());
        Ok(())
    }

    async fn recv(&mut self) -> io::Result<Vec<u8>> {
        self.incoming
            .pop_front()
            .ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))
    }

    async fn close(&mut self) -> io::Result<()> {
        self.closed = true;
        Ok(())
    }
}

/// A [`Sink`] that assembles a file in memory.
///
/// Used to check that a download lands the right bytes at the right offsets
/// without touching a disk — which also means a chunk-ordering bug shows up as a
/// failed assertion rather than as a corrupt game install.
#[derive(Debug, Default)]
pub struct MemorySink {
    contents: Mutex<Vec<u8>>,
    synced: Mutex<bool>,
}

impl MemorySink {
    /// An empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The assembled file.
    ///
    /// Returns an empty vector if a previous panic poisoned the lock, since a
    /// test double has nothing useful to say at that point and must not panic
    /// again on the way out.
    #[must_use]
    pub fn contents(&self) -> Vec<u8> {
        self.contents
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Whether [`Sink::sync`] was called.
    #[must_use]
    pub fn was_synced(&self) -> bool {
        self.synced.lock().map(|g| *g).unwrap_or(false)
    }
}

impl Sink for MemorySink {
    async fn write_at(&self, offset: u64, data: &[u8]) -> io::Result<()> {
        let Ok(mut contents) = self.contents.lock() else {
            return Err(io::Error::other("memory sink lock poisoned"));
        };
        let offset =
            usize::try_from(offset).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        let end = offset
            .checked_add(data.len())
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;

        // Writing past the end grows the file, filling the gap with zeroes —
        // the same thing a sparse file does, so a test sees what the disk would
        // have held.
        if contents.len() < end {
            contents.resize(end, 0);
        }
        if let Some(slot) = contents.get_mut(offset..end) {
            slot.copy_from_slice(data);
        }
        Ok(())
    }

    async fn allocate(&self, len: u64) -> io::Result<()> {
        let Ok(mut contents) = self.contents.lock() else {
            return Err(io::Error::other("memory sink lock poisoned"));
        };
        let len = usize::try_from(len).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        contents.resize(len, 0);
        Ok(())
    }

    async fn sync(&self) -> io::Result<()> {
        if let Ok(mut synced) = self.synced.lock() {
            *synced = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_order_writes_assemble_the_same_file() {
        // Chunks arrive from several CDN hosts at once and land wherever their
        // offsets say. The order they arrive in must not matter.
        let sink = MemorySink::new();
        block_on(async {
            sink.allocate(6).await.expect("must allocate");
            sink.write_at(3, b"def").await.expect("must write tail");
            sink.write_at(0, b"abc").await.expect("must write head");
            sink.sync().await.expect("must sync");
        });
        assert_eq!(sink.contents(), b"abcdef");
        assert!(sink.was_synced());
    }

    #[test]
    fn a_gap_reads_back_as_zeroes() {
        let sink = MemorySink::new();
        block_on(async {
            sink.write_at(4, b"xy").await.expect("must write");
        });
        assert_eq!(sink.contents(), vec![0, 0, 0, 0, b'x', b'y']);
    }

    #[test]
    fn unread_messages_are_visible_to_the_test() {
        // A replay test that stopped reading early would otherwise pass.
        let mut transport = MemoryTransport::new(vec![vec![1, 2], vec![3, 4]]);
        block_on(async {
            let first = transport.recv().await.expect("must receive");
            assert_eq!(first, vec![1, 2]);
        });
        assert_eq!(transport.unread(), 1);
    }

    #[test]
    fn a_closed_transport_reports_eof_not_an_empty_message() {
        // An empty Vec would be indistinguishable from a real zero-length
        // message, and the caller would act on it.
        let mut transport = MemoryTransport::new(Vec::new());
        block_on(async {
            let error = transport.recv().await.expect_err("must report EOF");
            assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
        });
    }
}
