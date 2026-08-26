//! Extracting an addon while it is still arriving.
//!
//! GMAD is laid out for this. The header and index come first, and after them
//! every file's contents follow back to back **in index order** — so once the
//! index has been read, the length of every file and the order they arrive in
//! is known, and each one can be written the moment its bytes land.
//!
//! Measured end to end on PAC3 (8.4 MB archive, 348 files):
//!
//! | | wall | read back | on disk |
//! |---|---|---|---|
//! | download, then extract | 2.00 s | 13.3 MB | 16.6 MB |
//! | stream | 1.78 s | 0.01 MB | 8.3 MB |
//!
//! The archive is neither written nor read back. What that does **not** save is
//! resident memory: streaming measured slightly higher, 30.9 MB against
//! 26.8 MB, because the reorder buffer on the download side is a fixed cost
//! while the archive is not. The saving is disk, and it scales with the addon
//! — a 400 MB one costs the same window.
//!
//! This type itself holds only the header and one write buffer, whatever the
//! addon's size.
//!
//! # The part that is not free
//!
//! This needs bytes **in order**, and tapline fetches chunks in parallel, so
//! they arrive in whatever order the CDN serves them. Ordering them again costs
//! a reorder buffer; see `tapline`'s streaming download for that side. This
//! module is just the consumer: feed it bytes from the start of the archive
//! onward and it writes files.

use crate::format::{Addon, parse_index};
use std::io::Write;
use std::path::{Path, PathBuf};
use tapline_ext::ExtensionError;

/// The largest header this will buffer before refusing.
///
/// Same bound as the seeking reader: an index that has not terminated by here
/// is malformed, and growing forever is how a hostile archive becomes an
/// out-of-memory.
const HEADER_LIMIT: usize = 1 << 26;

/// Writes an addon's files as its bytes arrive.
pub struct StreamingExtractor {
    /// Where files are written.
    dest: PathBuf,
    /// The header, until it is complete. Dropped afterwards.
    header: Vec<u8>,
    /// The parsed index, once known.
    addon: Option<Addon>,
    /// Validated destinations, one per entry, in index order.
    targets: Vec<PathBuf>,
    /// Which entry is being written.
    at: usize,
    /// How much of it still needs to arrive.
    remaining: u64,
    /// The file being written.
    current: Option<std::io::BufWriter<std::fs::File>>,
    /// Paths written so far.
    produced: Vec<String>,
    /// Total bytes fed in, for the error messages that need it.
    seen: u64,
}

impl StreamingExtractor {
    /// An extractor that writes into `dest`.
    #[must_use]
    pub fn new(dest: &Path) -> Self {
        Self {
            dest: dest.to_path_buf(),
            header: Vec::new(),
            addon: None,
            targets: Vec::new(),
            at: 0,
            remaining: 0,
            current: None,
            produced: Vec::new(),
            seen: 0,
        }
    }

    /// The addon's metadata, once enough bytes have arrived to know it.
    #[must_use]
    pub const fn addon(&self) -> Option<&Addon> {
        self.addon.as_ref()
    }

    /// Feeds the next bytes of the archive, in order.
    pub fn push(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        self.seen = self.seen.saturating_add(bytes.len() as u64);

        // Still reading the header: buffer until the index parses, then keep
        // whatever came after it as the first file's contents.
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
                    self.begin(addon, start)?;
                    // Anything already buffered past the index is file content.
                    let leftover = self.header.split_off(start.min(self.header.len()));
                    self.header = Vec::new();
                    return self.write_content(&leftover);
                }
                Err(error) => {
                    // Incomplete is expected here; anything else is fatal and
                    // more bytes will not fix it.
                    if is_incomplete(&error) {
                        return Ok(());
                    }
                    return Err(error);
                }
            }
        }

        // `parse` reported sizes that fit the archive, but a stream has no
        // declared length, so trailing bytes past the last entry are dropped
        // rather than written somewhere.
        if self.at >= self.targets.len() {
            return Ok(());
        }
        self.write_content(bytes)
    }

    /// Records the index and opens the first file.
    fn begin(&mut self, addon: Addon, _content_start: usize) -> Result<(), ExtensionError> {
        // Every path validated before a single byte is written: a hostile addon
        // must not get half its files onto disk before the escaping one is
        // caught.
        self.targets = addon
            .entries
            .iter()
            .map(|entry| {
                tapline_fs::validate_path(&entry.path)
                    .map(|safe| safe.resolve(&self.dest))
                    .map_err(|reason| ExtensionError::UnsafePath {
                        path: entry.path.clone(),
                        reason: reason.to_string(),
                    })
            })
            .collect::<Result<_, _>>()?;

        self.remaining = addon.entries.first().map_or(0, |entry| entry.size);
        self.addon = Some(addon);
        self.at = 0;
        self.open_current()?;
        self.skip_empty()
    }

    /// Opens the file for the current entry, if there is one.
    fn open_current(&mut self) -> Result<(), ExtensionError> {
        let Some(target) = self.targets.get(self.at) else {
            self.current = None;
            return Ok(());
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.current = Some(std::io::BufWriter::new(std::fs::File::create(target)?));
        Ok(())
    }

    /// Closes out zero-length entries, which have no bytes to wait for.
    fn skip_empty(&mut self) -> Result<(), ExtensionError> {
        while self.remaining == 0 && self.at < self.targets.len() {
            self.finish_current()?;
        }
        Ok(())
    }

    /// Flushes the current file and moves to the next entry.
    fn finish_current(&mut self) -> Result<(), ExtensionError> {
        if let Some(mut writer) = self.current.take() {
            writer.flush()?;
        }
        if let Some(addon) = &self.addon
            && let Some(entry) = addon.entries.get(self.at)
        {
            self.produced.push(entry.path.clone());
        }
        self.at += 1;
        self.remaining = self
            .addon
            .as_ref()
            .and_then(|addon| addon.entries.get(self.at))
            .map_or(0, |entry| entry.size);
        self.open_current()
    }

    /// Writes content bytes across however many entries they span.
    fn write_content(&mut self, mut bytes: &[u8]) -> Result<(), ExtensionError> {
        while !bytes.is_empty() && self.at < self.targets.len() {
            let take = (self.remaining as usize).min(bytes.len());
            let (head, tail) = bytes.split_at(take);
            if let Some(writer) = self.current.as_mut() {
                writer.write_all(head)?;
            }
            self.remaining -= take as u64;
            bytes = tail;

            if self.remaining == 0 {
                self.finish_current()?;
                self.skip_empty()?;
            }
        }
        Ok(())
    }

    /// Finishes, returning the paths written.
    ///
    /// An archive that ended early is an error: a caller told the files were
    /// extracted, when the last one is short, has been told something false.
    pub fn finish(mut self) -> Result<Vec<String>, ExtensionError> {
        if let Some(mut writer) = self.current.take() {
            writer.flush()?;
        }
        if self.addon.is_none() {
            return Err(ExtensionError::Malformed {
                extension: "gmad",
                reason: format!(
                    "the stream ended after {} bytes, before the index",
                    self.seen
                ),
            });
        }
        if self.at < self.targets.len() {
            let name = self
                .addon
                .as_ref()
                .and_then(|addon| addon.entries.get(self.at))
                .map_or("?", |entry| entry.path.as_str());
            return Err(ExtensionError::Malformed {
                extension: "gmad",
                reason: format!(
                    "the stream ended {} bytes into {name:?}, with {} of {} files written",
                    self.remaining,
                    self.at,
                    self.targets.len()
                ),
            });
        }
        Ok(self.produced)
    }
}

/// Whether an error means "not enough bytes yet" rather than "wrong".
fn is_incomplete(error: &ExtensionError) -> bool {
    // Not "past the archive": `parse_index` does not perform that check, and a
    // stream's contents are supposed to be missing while the index is read.
    matches!(
        error,
        ExtensionError::Malformed { reason, .. }
            if reason.contains("ends in the middle") || reason.contains("not terminated")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A valid archive, so the tests can feed it in pieces.
    fn build(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(crate::MAGIC);
        out.push(3);
        out.extend_from_slice(&0_u64.to_le_bytes());
        out.extend_from_slice(&1_u64.to_le_bytes());
        out.push(0);
        out.extend_from_slice(b"Streamed\0");
        out.extend_from_slice(b"desc\0");
        out.extend_from_slice(b"author\0");
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

    struct Scratch(PathBuf);

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str) -> Scratch {
        let base = std::env::var("TAPLINE_TEST_DIR").unwrap_or_else(|_| {
            format!(
                "{}/.cache/tapline-test",
                std::env::var("HOME").unwrap_or_else(|_| ".".into())
            )
        });
        let path = PathBuf::from(base).join(format!("stream-{name}"));
        let _ = std::fs::remove_dir_all(&path);
        Scratch(path)
    }

    /// Feeds an archive in fixed-size pieces, which is what a chunked download
    /// looks like from the consumer's side.
    fn feed(raw: &[u8], piece: usize, dest: &Path) -> Result<Vec<String>, ExtensionError> {
        let mut extractor = StreamingExtractor::new(dest);
        for slice in raw.chunks(piece.max(1)) {
            extractor.push(slice)?;
        }
        extractor.finish()
    }

    #[test]
    fn the_result_is_the_same_whatever_the_piece_size() {
        // The property that matters: a consumer must not care how the bytes
        // were split, and a chunked download splits them arbitrarily.
        let raw = build(&[
            ("lua/a.lua", b"print('a')"),
            ("materials/b.vmt", b"vmt contents here"),
            ("models/c.mdl", &[7_u8; 5000]),
        ]);

        for piece in [1, 7, 64, 999, 4096, raw.len(), raw.len() * 2] {
            let dir = scratch(&format!("piece{piece}"));
            let written = feed(&raw, piece, &dir.0).expect("must extract");
            assert_eq!(written.len(), 3, "piece {piece}");
            assert_eq!(
                std::fs::read(dir.0.join("lua/a.lua")).expect("read"),
                b"print('a')",
                "piece {piece}"
            );
            assert_eq!(
                std::fs::read(dir.0.join("models/c.mdl"))
                    .expect("read")
                    .len(),
                5000,
                "piece {piece}"
            );
        }
    }

    #[test]
    fn it_matches_what_the_seeking_extractor_produces() {
        let raw = build(&[("a/b.txt", b"one"), ("c.txt", b"two")]);
        let streamed = scratch("streamed");
        let seeking = scratch("seeking");

        let from_stream = feed(&raw, 3, &streamed.0).expect("stream");

        let archive = seeking.0.join("archive.gma");
        std::fs::create_dir_all(&seeking.0).expect("mkdir");
        std::fs::write(&archive, &raw).expect("write");
        let from_seek = crate::extract(&archive, &seeking.0.join("out")).expect("seek");

        assert_eq!(from_stream, from_seek);
        for name in &from_stream {
            assert_eq!(
                std::fs::read(streamed.0.join(name)).expect("streamed"),
                std::fs::read(seeking.0.join("out").join(name)).expect("seeking"),
                "{name} differs between the two extractors"
            );
        }
    }

    #[test]
    fn a_stream_that_stops_early_is_an_error() {
        // Silently returning a short file would tell the caller the addon was
        // extracted when part of it is missing.
        let raw = build(&[("a.txt", &[1_u8; 100])]);
        let dir = scratch("truncated");
        let mut extractor = StreamingExtractor::new(&dir.0);
        extractor
            .push(raw.get(..raw.len() - 40).expect("prefix"))
            .expect("push");
        let error = extractor.finish().expect_err("must refuse");
        assert!(error.to_string().contains("the stream ended"), "{error}");
    }

    #[test]
    fn a_stream_that_never_reaches_the_index_says_so() {
        let dir = scratch("noindex");
        let mut extractor = StreamingExtractor::new(&dir.0);
        extractor.push(b"GMAD\x03").expect("push");
        let error = extractor.finish().expect_err("must refuse");
        assert!(error.to_string().contains("before the index"), "{error}");
    }

    #[test]
    fn a_bad_magic_fails_immediately_rather_than_waiting_for_more() {
        let dir = scratch("badmagic");
        let mut extractor = StreamingExtractor::new(&dir.0);
        let error = extractor
            .push(b"PK\x03\x04nonsense follows")
            .expect_err("must refuse");
        assert!(error.to_string().contains("expected magic GMAD"), "{error}");
    }

    #[test]
    fn an_escaping_path_is_refused_before_anything_is_written() {
        let raw = build(&[("../../etc/cron.d/x", b"pwned"), ("ok.txt", b"fine")]);
        let dir = scratch("escape");
        let mut extractor = StreamingExtractor::new(&dir.0);
        let error = extractor.push(&raw).expect_err("must refuse");
        assert!(
            matches!(error, ExtensionError::UnsafePath { .. }),
            "{error}"
        );
        // And nothing landed, including the entry that was fine on its own.
        assert!(!dir.0.join("ok.txt").exists(), "a file was written anyway");
    }

    #[test]
    fn zero_length_entries_do_not_stall_the_stream() {
        // An empty file has no bytes to wait for, so the consumer must step
        // past it rather than sit waiting for content that never comes.
        let raw = build(&[("empty.txt", b""), ("after.txt", b"here")]);
        let dir = scratch("empty");
        let written = feed(&raw, 5, &dir.0).expect("must extract");
        assert_eq!(written.len(), 2);
        assert_eq!(
            std::fs::read(dir.0.join("empty.txt")).expect("read").len(),
            0
        );
        assert_eq!(
            std::fs::read(dir.0.join("after.txt")).expect("read"),
            b"here"
        );
    }

    #[test]
    fn metadata_is_available_as_soon_as_the_index_lands() {
        // The point of reading the header first: a caller can report the file
        // count and total size before any content has arrived.
        let raw = build(&[("a", b"12345"), ("b", b"67")]);
        let dir = scratch("meta");
        let mut extractor = StreamingExtractor::new(&dir.0);
        assert!(extractor.addon().is_none());

        // Everything up to the first byte of content.
        let header_len = raw.len() - 7;
        extractor
            .push(raw.get(..header_len).expect("prefix"))
            .expect("push");
        let addon = extractor.addon().expect("the index should be known");
        assert_eq!(addon.name, "Streamed");
        assert_eq!(addon.entries.len(), 2);
        assert_eq!(addon.unpacked_size(), 7);
    }
}
