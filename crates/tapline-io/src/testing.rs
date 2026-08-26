//! Test doubles for the IO traits, and a runtime-free executor to drive them.
//!
//! Behind the `testing` feature so it never reaches a production build. Every
//! crate that speaks the protocol uses these to test against recorded bytes
//! instead of a live Steam connection: no account, no network, no flakiness, and
//! a failing test points at our framing rather than at Valve having a bad day.

use crate::{Sink, Stream};
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

/// A [`Stream`] that reads from a buffer and records what was written.
///
/// This is how a recorded CM handshake is replayed: load the bytes Steam sent,
/// run the real handshake code against them, and assert on what it tried to
/// send back.
#[derive(Debug, Default)]
pub struct MemoryStream {
    input: Vec<u8>,
    read_pos: usize,
    written: Vec<u8>,
    shutdown: bool,
}

impl MemoryStream {
    /// A stream that will serve `input` to its reader.
    #[must_use]
    pub fn new(input: Vec<u8>) -> Self {
        Self {
            input,
            read_pos: 0,
            written: Vec::new(),
            shutdown: false,
        }
    }

    /// Everything written so far.
    #[must_use]
    pub fn written(&self) -> &[u8] {
        &self.written
    }

    /// How many bytes of the input have not been read.
    ///
    /// A replay test asserting this is zero proves the parser consumed the whole
    /// recording rather than stopping early and passing by accident.
    #[must_use]
    pub fn unread(&self) -> usize {
        self.input.len().saturating_sub(self.read_pos)
    }

    /// Whether the writer closed the stream.
    #[must_use]
    pub const fn is_shutdown(&self) -> bool {
        self.shutdown
    }
}

impl Stream for MemoryStream {
    async fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let end = self
            .read_pos
            .checked_add(buf.len())
            .ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?;
        let src = self
            .input
            .get(self.read_pos..end)
            .ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?;
        buf.copy_from_slice(src);
        self.read_pos = end;
        Ok(())
    }

    async fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.written.extend_from_slice(buf);
        Ok(())
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        self.shutdown = true;
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
    fn unread_input_is_visible_to_the_test() {
        // A replay test that stopped parsing early would otherwise pass.
        let mut stream = MemoryStream::new(vec![1, 2, 3, 4]);
        block_on(async {
            let mut buf = [0_u8; 2];
            stream.read_exact(&mut buf).await.expect("must read");
        });
        assert_eq!(stream.unread(), 2);
    }
}
