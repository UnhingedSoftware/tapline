//! Where a streamed archive can go.
//!
//! Both of these sit on [`Splitter`], so they see the same entry events and
//! differ only in what they do with them. Adding a third target — an object
//! store, a tar, a hash manifest — means writing an [`EntrySink`] and nothing
//! else.
//!
//! [`Splitter`]: crate::Splitter

use crate::format::{Addon, Entry};
use crate::split::EntrySink;
use crate::zip;
use std::io::Write;
use std::path::{Path, PathBuf};
use tapline_ext::ExtensionError;

/// Validates every path in an index, up front.
///
/// Before a single byte is written, always. A Workshop item is published by
/// anyone, and an archive that gets half its files onto disk before the one
/// escaping the root is noticed has already done the damage.
fn validate(addon: &Addon, root: &Path) -> Result<Vec<PathBuf>, ExtensionError> {
    addon
        .entries
        .iter()
        .map(|entry| {
            tapline_fs::validate_path(&entry.path)
                .map(|safe| safe.resolve(root))
                .map_err(|reason| ExtensionError::UnsafePath {
                    path: entry.path.clone(),
                    reason: reason.to_string(),
                })
        })
        .collect()
}

/// Writes an archive's files into a directory as they arrive.
pub struct ToDirectory {
    dest: PathBuf,
    targets: Vec<PathBuf>,
    names: Vec<String>,
    current: Option<std::io::BufWriter<std::fs::File>>,
    at: usize,
    produced: Vec<String>,
}

impl ToDirectory {
    /// A sink writing into `dest`.
    #[must_use]
    pub fn new(dest: &Path) -> Self {
        Self {
            dest: dest.to_path_buf(),
            targets: Vec::new(),
            names: Vec::new(),
            current: None,
            at: 0,
            produced: Vec::new(),
        }
    }

    /// The paths written, relative to the destination.
    #[must_use]
    pub fn produced(&self) -> &[String] {
        &self.produced
    }

    /// Consumes the sink for its list of paths.
    #[must_use]
    pub fn into_produced(self) -> Vec<String> {
        self.produced
    }
}

impl EntrySink for ToDirectory {
    fn index(&mut self, addon: &Addon) -> Result<(), ExtensionError> {
        self.targets = validate(addon, &self.dest)?;
        self.names = addon
            .entries
            .iter()
            .map(|entry| entry.path.clone())
            .collect();
        Ok(())
    }

    fn begin(&mut self, _entry: &Entry, index: usize) -> Result<(), ExtensionError> {
        self.at = index;
        let Some(target) = self.targets.get(index) else {
            return Ok(());
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.current = Some(std::io::BufWriter::new(std::fs::File::create(target)?));
        Ok(())
    }

    fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        if let Some(writer) = self.current.as_mut() {
            writer.write_all(bytes)?;
        }
        Ok(())
    }

    fn end(&mut self) -> Result<(), ExtensionError> {
        if let Some(mut writer) = self.current.take() {
            writer.flush()?;
        }
        if let Some(name) = self.names.get(self.at) {
            self.produced.push(name.clone());
        }
        Ok(())
    }
}

/// How many bytes of completed entries to hold before compressing them.
///
/// Deflate is the cost of building a ZIP, and it parallelises across entries —
/// but only if there are several to hand. Compressing each entry the moment it
/// completed measured 2.29 s against 2.01 s for downloading and converting
/// afterwards, because it gave up the parallelism to save the disk. Batching
/// gets both.
///
/// Smaller than the seeking path's 32 MB: this is memory held *during* a
/// download rather than instead of one.
const BATCH_BYTES: usize = 8 << 20;

/// Writes an archive's files into a ZIP as they arrive.
///
/// A ZIP is written front to back — local header, data, repeat, then the
/// central directory — so it can be built from a stream without ever holding
/// the whole thing. What it holds is a batch of completed entries waiting to be
/// deflated together; peak memory is that batch, not the archive.
pub struct ToZip {
    writer: Option<zip::Writer<std::io::BufWriter<std::fs::File>>>,
    /// The validated, normalised name for each entry.
    names: Vec<String>,
    /// The entry being accumulated.
    buffer: Vec<u8>,
    /// Completed entries waiting to be compressed together.
    batch: Vec<(String, Vec<u8>)>,
    /// How many bytes those hold.
    batch_bytes: usize,
    /// How many threads to compress across.
    threads: usize,
    at: usize,
    compress: bool,
    entries: usize,
}

impl ToZip {
    /// A sink writing a ZIP at `dest`.
    ///
    /// `compress` deflates entries that get smaller for it; otherwise
    /// everything is stored, which is roughly four times faster and the right
    /// choice when the result goes somewhere that compresses on the wire.
    pub fn new(dest: &Path, compress: bool) -> Result<Self, ExtensionError> {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Self {
            writer: Some(zip::Writer::new(std::io::BufWriter::new(
                std::fs::File::create(dest)?,
            ))),
            names: Vec::new(),
            buffer: Vec::new(),
            batch: Vec::new(),
            batch_bytes: 0,
            threads: std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
            at: 0,
            compress,
            entries: 0,
        })
    }

    /// How many entries have been written.
    #[must_use]
    pub const fn entries(&self) -> usize {
        self.entries
    }

    /// Compresses everything queued and writes it, in order.
    fn flush_batch(&mut self) -> Result<(), ExtensionError> {
        if self.batch.is_empty() {
            return Ok(());
        }
        let batch = std::mem::take(&mut self.batch);
        self.batch_bytes = 0;
        for prepared in crate::compress_batch(batch, self.compress, self.threads) {
            if let Some(writer) = self.writer.as_mut() {
                writer.add_prepared(prepared)?;
            }
        }
        Ok(())
    }

    /// Closes the archive, writing its central directory.
    ///
    /// Required: a ZIP without one is a file most readers refuse.
    pub fn close(mut self) -> Result<usize, ExtensionError> {
        self.flush_batch()?;
        if let Some(writer) = self.writer.take() {
            let mut out = writer.finish()?;
            out.flush()?;
        }
        Ok(self.entries)
    }
}

impl EntrySink for ToZip {
    fn index(&mut self, addon: &Addon) -> Result<(), ExtensionError> {
        // Validated against a notional root: the names go inside a ZIP rather
        // than onto the filesystem, but an archive carrying `..` into whatever
        // unpacks it next is the same problem one step removed.
        let root = Path::new("");
        self.names = validate(addon, root)?
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect();
        Ok(())
    }

    fn begin(&mut self, entry: &Entry, index: usize) -> Result<(), ExtensionError> {
        self.at = index;
        self.buffer.clear();
        // The size is known from the index, so the buffer is allocated once
        // rather than grown as bytes arrive.
        self.buffer.reserve(entry.size as usize);
        Ok(())
    }

    fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn end(&mut self) -> Result<(), ExtensionError> {
        let name = self.names.get(self.at).cloned().unwrap_or_default();
        // Queued rather than compressed here: deflate parallelises across
        // entries and there is nothing to parallelise with one.
        let body = std::mem::take(&mut self.buffer);
        self.batch_bytes = self.batch_bytes.saturating_add(body.len());
        self.batch.push((name, body));
        self.entries += 1;

        if self.batch_bytes >= BATCH_BYTES {
            self.flush_batch()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split::Splitter;

    fn build(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(crate::MAGIC);
        out.push(3);
        out.extend_from_slice(&0_u64.to_le_bytes());
        out.extend_from_slice(&1_u64.to_le_bytes());
        out.push(0);
        out.extend_from_slice(b"Sink\0desc\0author\0");
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
        let path = PathBuf::from(base).join(format!("sink-{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("mkdir");
        Scratch(path)
    }

    #[test]
    fn a_directory_sink_writes_the_files() {
        let raw = build(&[("lua/a.lua", b"print(1)"), ("b.txt", b"two")]);
        let dir = scratch("dir");
        let mut splitter = Splitter::new(ToDirectory::new(&dir.0));
        for piece in raw.chunks(5) {
            splitter.push(piece).expect("push");
        }
        let produced = splitter.finish().expect("finish").into_produced();

        assert_eq!(produced, vec!["lua/a.lua", "b.txt"]);
        assert_eq!(
            std::fs::read(dir.0.join("lua/a.lua")).expect("read"),
            b"print(1)"
        );
    }

    #[test]
    fn a_zip_sink_writes_an_archive_a_reader_accepts() {
        let raw = build(&[("a.txt", &vec![b'a'; 5000]), ("b.txt", b"short")]);
        let dir = scratch("zip");
        let zip_path = dir.0.join("out.zip");

        let mut splitter = Splitter::new(ToZip::new(&zip_path, true).expect("create"));
        for piece in raw.chunks(7) {
            splitter.push(piece).expect("push");
        }
        let count = splitter.finish().expect("finish").close().expect("close");
        assert_eq!(count, 2);

        let bytes = std::fs::read(&zip_path).expect("read");
        assert_eq!(bytes.get(..2), Some(&b"PK"[..]), "no PK signature");
        // 5000 identical bytes must have compressed, so the archive is far
        // smaller than its contents.
        assert!(bytes.len() < 1000, "not compressed: {} bytes", bytes.len());

        // And an external reader must accept it.
        if let Ok(output) = std::process::Command::new("unzip")
            .arg("-t")
            .arg(&zip_path)
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            assert!(output.status.success(), "unzip rejected it:\n{text}");
        }
    }

    #[test]
    fn a_streamed_zip_holds_one_entry_not_the_archive() {
        // The memory claim, checked by construction rather than asserted: after
        // an entry is written its buffer is released, so a 10-entry archive
        // never holds more than its largest entry.
        let raw = build(&[("big", &vec![7_u8; 100_000]), ("small", b"x")]);
        let dir = scratch("zipmem");
        let mut splitter = Splitter::new(ToZip::new(&dir.0.join("m.zip"), false).expect("create"));
        for piece in raw.chunks(4096) {
            splitter.push(piece).expect("push");
        }
        let sink = splitter.finish().expect("finish");
        assert_eq!(sink.entries(), 2);
        // The in-progress buffer is handed to the batch rather than kept, so
        // nothing holds a second copy of the entry just finished.
        assert_eq!(
            sink.buffer.capacity(),
            0,
            "the last entry's buffer was kept"
        );
        // And the batch holds only what has not been written yet, which is
        // bounded by BATCH_BYTES rather than by the archive.
        assert!(sink.batch_bytes <= BATCH_BYTES, "batch grew past its bound");
        sink.close().expect("close");
    }

    #[test]
    fn an_escaping_path_is_refused_by_both_sinks() {
        let raw = build(&[("../../etc/passwd", b"no"), ("ok", b"yes")]);

        let dir = scratch("escape");
        let mut to_dir = Splitter::new(ToDirectory::new(&dir.0));
        let error = to_dir.push(&raw).expect_err("directory sink must refuse");
        assert!(
            matches!(error, ExtensionError::UnsafePath { .. }),
            "{error}"
        );
        assert!(!dir.0.join("ok").exists(), "a file was written anyway");

        let mut to_zip = Splitter::new(ToZip::new(&dir.0.join("e.zip"), false).expect("create"));
        let error = to_zip.push(&raw).expect_err("zip sink must refuse");
        assert!(
            matches!(error, ExtensionError::UnsafePath { .. }),
            "{error}"
        );
    }

    #[test]
    fn a_zip_without_close_is_not_silently_valid() {
        // Dropping the sink without closing leaves a file with no central
        // directory. The test records that this is the caller's mistake to
        // avoid, and that `close` is what makes the archive readable.
        let dir = scratch("noclose");
        let path = dir.0.join("partial.zip");
        let raw = build(&[("a", b"body")]);
        let mut splitter = Splitter::new(ToZip::new(&path, false).expect("create"));
        splitter.push(&raw).expect("push");
        drop(splitter);

        let bytes = std::fs::read(&path).expect("read");
        let end = 0x0605_4b50_u32.to_le_bytes();
        assert!(
            !bytes.windows(4).any(|w| w == end),
            "an unclosed archive should have no end record"
        );
    }
}
