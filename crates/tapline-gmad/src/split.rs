//! Turning a stream of archive bytes into a stream of entry events.
//!
//! The part both streaming consumers share. Feed it the archive from the start
//! and it calls a [`EntrySink`] as things happen: the index, then each entry
//! beginning, its bytes, and its end.
//!
//! Splitting this out is what makes the target pluggable. Writing files to a
//! directory and writing a ZIP are the same problem — read entries in order,
//! do something with each — differing only in what "something" is, and neither
//! wants its own copy of a byte-boundary state machine.

use crate::format::{Addon, parse_index};
use tapline_ext::ExtensionError;
use tapline_ext::{ArchiveEntry, Decoder, EntrySink};

/// The largest header this will buffer before refusing.
const HEADER_LIMIT: usize = 1 << 26;

/// Feeds archive bytes to an [`EntrySink`].
pub struct Splitter<S: EntrySink> {
    sink: S,
    /// The header, until the index parses. Dropped afterwards.
    header: Vec<u8>,
    /// The index, once known.
    addon: Option<Addon>,
    /// The same index in the format-neutral vocabulary the sinks speak.
    entries: Vec<ArchiveEntry>,
    /// Which entry is being fed.
    at: usize,
    /// How much of it is still to come.
    remaining: u64,
    /// Bytes fed in, for the error that needs it.
    seen: u64,
}

impl<S: EntrySink> Splitter<S> {
    /// A splitter feeding `sink`.
    pub const fn new(sink: S) -> Self {
        Self {
            sink,
            header: Vec::new(),
            addon: None,
            entries: Vec::new(),
            at: 0,
            remaining: 0,
            seen: 0,
        }
    }

    /// The index, once enough bytes have arrived to know it.
    pub const fn addon(&self) -> Option<&Addon> {
        self.addon.as_ref()
    }

    /// How many entries have been completed.
    pub const fn completed(&self) -> usize {
        self.at
    }

    /// Feeds the next bytes of the archive, in order.
    pub fn push(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        self.seen = self.seen.saturating_add(bytes.len() as u64);

        if self.addon.is_none() {
            self.header.extend_from_slice(bytes);
            if self.header.len() > HEADER_LIMIT {
                return Err(ExtensionError::Malformed {
                    extension: "gmad",
                    reason: format!("no index after {HEADER_LIMIT} bytes"),
                });
            }
            match parse_index(&self.header) {
                Ok(addon) => {
                    let start = addon
                        .entries
                        .first()
                        .map_or(self.header.len(), |entry| entry.offset);
                    self.entries = addon
                        .entries
                        .iter()
                        .map(|entry| ArchiveEntry {
                            path: entry.path.clone(),
                            size: entry.size,
                        })
                        .collect();
                    self.sink.index(&self.entries)?;
                    self.addon = Some(addon);
                    self.at = 0;
                    self.remaining = self
                        .addon
                        .as_ref()
                        .and_then(|addon| addon.entries.first())
                        .map_or(0, |entry| entry.size);
                    self.open()?;
                    self.close_empty()?;

                    // Whatever was buffered past the index is already content.
                    let leftover = std::mem::take(&mut self.header);
                    let content = leftover
                        .get(start.min(leftover.len())..)
                        .unwrap_or_default();
                    return self.feed(content);
                }
                Err(error) if is_incomplete(&error) => return Ok(()),
                Err(error) => return Err(error),
            }
        }

        self.feed(bytes)
    }

    /// Announces the current entry, if there is one.
    fn open(&mut self) -> Result<(), ExtensionError> {
        let Some(entry) = self.entries.get(self.at) else {
            return Ok(());
        };
        // Cloned so the sink call does not hold a borrow of `self.entries`
        // while `self.sink` is borrowed mutably.
        let entry = entry.clone();
        self.sink.begin(&entry, self.at)
    }

    /// Closes out entries with no bytes to wait for.
    fn close_empty(&mut self) -> Result<(), ExtensionError> {
        while self.remaining == 0 && self.has_current() {
            self.advance()?;
        }
        Ok(())
    }

    fn has_current(&self) -> bool {
        self.at < self.entries.len()
    }

    /// Ends the current entry and opens the next.
    fn advance(&mut self) -> Result<(), ExtensionError> {
        self.sink.end()?;
        self.at += 1;
        self.remaining = self
            .addon
            .as_ref()
            .and_then(|addon| addon.entries.get(self.at))
            .map_or(0, |entry| entry.size);
        self.open()
    }

    /// Routes content bytes across however many entries they span.
    fn feed(&mut self, mut bytes: &[u8]) -> Result<(), ExtensionError> {
        while !bytes.is_empty() && self.has_current() {
            let take = (self.remaining as usize).min(bytes.len());
            let (head, tail) = bytes.split_at(take);
            if !head.is_empty() {
                self.sink.data(head)?;
            }
            self.remaining -= take as u64;
            bytes = tail;
            if self.remaining == 0 {
                self.advance()?;
                self.close_empty()?;
            }
        }
        // Bytes past the last entry are the archive's trailing checksum, which
        // belongs to no entry.
        Ok(())
    }

    /// Finishes, returning the sink.
    ///
    /// A stream that ended early is an error: telling a caller the archive was
    /// processed when the last entry is short would be false.
    /// Ends the archive without consuming the splitter.
    fn finish_in_place(&mut self) -> Result<(), ExtensionError> {
        self.check_complete()?;
        self.sink.finish()
    }

    /// Whether every entry was fed.
    fn check_complete(&self) -> Result<(), ExtensionError> {
        let Some(addon) = &self.addon else {
            return Err(ExtensionError::Malformed {
                extension: "gmad",
                reason: format!(
                    "the stream ended after {} bytes, before the index",
                    self.seen
                ),
            });
        };
        if self.at < addon.entries.len() {
            let name = addon
                .entries
                .get(self.at)
                .map_or("?", |entry| entry.path.as_str());
            return Err(ExtensionError::Malformed {
                extension: "gmad",
                reason: format!(
                    "the stream ended {} bytes into {name:?}, with {} of {} entries done",
                    self.remaining,
                    self.at,
                    addon.entries.len()
                ),
            });
        }
        Ok(())
    }

    /// Finishes, returning the sink.
    ///
    /// A stream that ended early is an error: telling a caller the archive was
    /// processed when the last entry is short would be false.
    pub fn finish(mut self) -> Result<S, ExtensionError> {
        self.finish_in_place()?;
        Ok(self.sink)
    }
}

impl<S: EntrySink> Decoder for Splitter<S> {
    fn format(&self) -> &'static str {
        "gma"
    }

    fn push(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        Self::push(self, bytes)
    }

    fn finish(&mut self) -> Result<(), ExtensionError> {
        self.finish_in_place()
    }
}

/// Whether an error means "not enough bytes yet" rather than "wrong".
///
/// Not "past the archive": `parse_index` does not perform that check, and a
/// stream's contents are supposed to be missing while its index is read.
fn is_incomplete(error: &ExtensionError) -> bool {
    matches!(
        error,
        ExtensionError::Malformed { reason, .. }
            if reason.contains("ends in the middle") || reason.contains("not terminated")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records what it was told, so the ordering can be asserted.
    #[derive(Default, Debug)]
    struct Recorder {
        events: Vec<String>,
        current: Vec<u8>,
    }

    impl EntrySink for Recorder {
        fn index(&mut self, entries: &[ArchiveEntry]) -> Result<(), ExtensionError> {
            self.events.push(format!("index:{}", entries.len()));
            Ok(())
        }

        fn begin(&mut self, entry: &ArchiveEntry, index: usize) -> Result<(), ExtensionError> {
            self.events.push(format!("begin:{index}:{}", entry.path));
            self.current.clear();
            Ok(())
        }

        fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
            self.current.extend_from_slice(bytes);
            Ok(())
        }

        fn end(&mut self) -> Result<(), ExtensionError> {
            self.events
                .push(format!("end:{}", String::from_utf8_lossy(&self.current)));
            Ok(())
        }
    }

    fn build(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(crate::MAGIC);
        out.push(3);
        out.extend_from_slice(&0_u64.to_le_bytes());
        out.extend_from_slice(&1_u64.to_le_bytes());
        out.push(0);
        out.extend_from_slice(b"Split\0desc\0author\0");
        out.extend_from_slice(&1_i32.to_le_bytes());
        for (index, (path, body)) in files.iter().enumerate() {
            out.extend_from_slice(&(index as u32 + 1).to_le_bytes());
            out.extend_from_slice(path.as_bytes());
            out.push(0);
            out.extend_from_slice(&(body.len() as i64).to_le_bytes());
            out.extend_from_slice(&0_u32.to_le_bytes());
        }
        out.extend_from_slice(&0_u32.to_le_bytes());
        for (_, body) in files {
            out.extend_from_slice(body);
        }
        out
    }

    fn run(raw: &[u8], piece: usize) -> Vec<String> {
        let mut splitter = Splitter::new(Recorder::default());
        for slice in raw.chunks(piece.max(1)) {
            splitter.push(slice).expect("push");
        }
        splitter.finish().expect("finish").events
    }

    #[test]
    fn events_arrive_in_order() {
        let raw = build(&[("a.txt", b"one"), ("b.txt", b"two")]);
        assert_eq!(
            run(&raw, 4096),
            vec![
                "index:2",
                "begin:0:a.txt",
                "end:one",
                "begin:1:b.txt",
                "end:two",
            ]
        );
    }

    #[test]
    fn the_piece_size_does_not_change_the_events() {
        // A chunked download splits bytes arbitrarily, so the sink must see the
        // same sequence whatever the split.
        let raw = build(&[("a.txt", b"hello"), ("b/c.txt", b"world!")]);
        let reference = run(&raw, raw.len());
        for piece in [1, 2, 3, 7, 13, 64, 1024] {
            assert_eq!(run(&raw, piece), reference, "piece size {piece}");
        }
    }

    #[test]
    fn an_empty_entry_gets_a_begin_and_an_end_with_no_data() {
        let raw = build(&[("empty", b""), ("after", b"x")]);
        assert_eq!(
            run(&raw, 3),
            vec!["index:2", "begin:0:empty", "end:", "begin:1:after", "end:x",]
        );
    }

    #[test]
    fn an_archive_with_no_entries_reports_only_its_index() {
        assert_eq!(run(&build(&[]), 8), vec!["index:0"]);
    }

    #[test]
    fn a_short_stream_is_refused() {
        let raw = build(&[("a", &[1_u8; 50])]);
        let mut splitter = Splitter::new(Recorder::default());
        splitter
            .push(raw.get(..raw.len() - 20).expect("prefix"))
            .expect("push");
        let error = splitter.finish().expect_err("must refuse");
        assert!(error.to_string().contains("the stream ended"), "{error}");
    }

    #[test]
    fn a_sink_that_refuses_stops_the_stream() {
        // A sink rejecting the index — a path validator, say — must prevent any
        // entry bytes reaching it.
        struct Refuses;
        impl EntrySink for Refuses {
            fn index(&mut self, _entries: &[ArchiveEntry]) -> Result<(), ExtensionError> {
                Err(ExtensionError::UnsafePath {
                    path: "../x".to_owned(),
                    reason: "escapes".to_owned(),
                })
            }
            fn begin(
                &mut self,
                _entry: &ArchiveEntry,
                _index: usize,
            ) -> Result<(), ExtensionError> {
                panic!("begin must not be reached after index refused");
            }
            fn data(&mut self, _bytes: &[u8]) -> Result<(), ExtensionError> {
                panic!("data must not be reached after index refused");
            }
            fn end(&mut self) -> Result<(), ExtensionError> {
                panic!("end must not be reached after index refused");
            }
        }

        let raw = build(&[("a", b"x")]);
        let mut splitter = Splitter::new(Refuses);
        let error = splitter.push(&raw).expect_err("must refuse");
        assert!(
            matches!(error, ExtensionError::UnsafePath { .. }),
            "{error}"
        );
    }
}
