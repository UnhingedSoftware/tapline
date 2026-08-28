//! Delivering a file's bytes in order, while still fetching them in parallel.

use crate::install::InstallError;

/// How many chunks may be in flight for a streamed file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// Chunks fetched ahead of the one the consumer is waiting for.
    pub size: usize,
}

impl Default for Window {
    fn default() -> Self {
        // 16 MiB worst-case buffer; streamed items still share the process budget.
        Self { size: 16 }
    }
}

impl Window {
    /// A window of `size` chunks, never zero.
    #[must_use]
    pub const fn new(size: usize) -> Self {
        Self {
            size: if size == 0 { 1 } else { size },
        }
    }
}

/// Reorders chunks that arrive out of order into a single ordered stream.
pub struct Reorderer {
    next: usize,
    pending: std::collections::BTreeMap<usize, Vec<u8>>,
}

impl Default for Reorderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Reorderer {
    /// An empty reorderer, waiting for chunk 0.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next: 0,
            pending: std::collections::BTreeMap::new(),
        }
    }

    /// Accepts a chunk and returns everything now deliverable, in order.
    pub fn accept(&mut self, index: usize, bytes: Vec<u8>) -> Vec<Vec<u8>> {
        if index < self.next {
            // Already delivered; drop rather than deliver the same bytes twice.
            return Vec::new();
        }
        self.pending.insert(index, bytes);

        let mut ready = Vec::new();
        while let Some(bytes) = self.pending.remove(&self.next) {
            ready.push(bytes);
            self.next += 1;
        }
        ready
    }

    /// How many chunks are waiting on an earlier one.
    #[must_use]
    pub fn buffered(&self) -> usize {
        self.pending.len()
    }

    /// How many have been delivered.
    #[must_use]
    pub const fn delivered(&self) -> usize {
        self.next
    }

    /// Whether anything is still held back.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

/// What a streamed download did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StreamReport {
    /// Bytes fetched from the CDN.
    pub bytes_downloaded: u64,
    /// Bytes handed to the consumer.
    pub bytes_streamed: u64,
    /// Chunks fetched.
    pub chunks: u64,
    /// The largest number of chunks held back at once.
    pub peak_buffered: usize,
}

/// What a streamed download feeds its bytes to.
pub type Consumer<'a> = &'a mut (dyn FnMut(&[u8]) -> Result<(), InstallError> + Send);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_order_arrivals_pass_straight_through() {
        let mut reorderer = Reorderer::new();
        assert_eq!(reorderer.accept(0, vec![1]), vec![vec![1]]);
        assert_eq!(reorderer.accept(1, vec![2]), vec![vec![2]]);
        assert_eq!(reorderer.buffered(), 0);
        assert_eq!(reorderer.delivered(), 2);
    }

    #[test]
    fn an_early_chunk_waits_for_the_one_before_it() {
        let mut reorderer = Reorderer::new();
        assert!(reorderer.accept(2, vec![3]).is_empty());
        assert!(reorderer.accept(1, vec![2]).is_empty());
        assert_eq!(reorderer.buffered(), 2);

        assert_eq!(
            reorderer.accept(0, vec![1]),
            vec![vec![1], vec![2], vec![3]]
        );
        assert_eq!(reorderer.buffered(), 0);
        assert_eq!(reorderer.delivered(), 3);
    }

    #[test]
    fn every_arrival_order_produces_the_same_stream() {
        for start in 0..10 {
            let mut reorderer = Reorderer::new();
            let mut out = Vec::new();
            for step in 0..10 {
                let index = (start + step) % 10;
                out.extend(reorderer.accept(index, vec![index as u8]));
            }
            let flat: Vec<u8> = out.into_iter().flatten().collect();
            assert_eq!(
                flat,
                (0..10_u8).collect::<Vec<_>>(),
                "arrival order starting at {start} produced the wrong stream"
            );
            assert!(reorderer.is_empty());
        }
    }

    #[test]
    fn reversed_arrivals_still_come_out_forwards() {
        let mut reorderer = Reorderer::new();
        let mut out = Vec::new();
        for index in (0..8).rev() {
            out.extend(reorderer.accept(index, vec![index as u8]));
        }
        let flat: Vec<u8> = out.into_iter().flatten().collect();
        assert_eq!(flat, (0..8_u8).collect::<Vec<_>>());
    }

    #[test]
    fn a_duplicate_is_dropped_rather_than_delivered_twice() {
        let mut reorderer = Reorderer::new();
        assert_eq!(reorderer.accept(0, vec![1]), vec![vec![1]]);
        assert!(reorderer.accept(0, vec![9]).is_empty());
        assert_eq!(reorderer.delivered(), 1);
    }

    #[test]
    fn the_window_is_never_zero() {
        // Zero would deadlock: nothing could ever be in flight.
        assert_eq!(Window::new(0).size, 1);
        assert_eq!(Window::new(4).size, 4);
        assert_eq!(Window::default().size, 16);
    }
}
