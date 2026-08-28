//! Garry's Mod addon archives.

#![forbid(unsafe_code)]

mod format;
mod glob;
mod sink;
mod split;
mod stream;
mod zip;

pub use format::{Addon, Entry, MAGIC, parse, parse_index};
pub use glob::{Patterns, matches as glob_matches};
pub use sink::{Fanout, Filtered, ToDirectory, ToZip as ZipSink};
pub use split::Splitter;
pub use stream::StreamingExtractor;
pub use tapline_ext::{ArchiveEntry, Decoder, EntrySink, IndexLocation};

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use tapline_ext::{Extension, ExtensionError, Landed, Produced, unpack_dir};

const INDEX_PROBE: usize = 1 << 16;

/// An index unterminated after this is malformed; growing further invites out-of-memory.
const INDEX_LIMIT: usize = 1 << 26;

const COPY_BUFFER: usize = 1 << 20;

/// Caps resident memory while compressing a batch.
const BATCH_BYTES: u64 = 32 << 20;

/// Reads an addon's header and index from a file without reading the contents.
pub fn read_index(path: &Path) -> Result<(Addon, u64), ExtensionError> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = vec![0_u8; INDEX_PROBE];
    let mut filled = 0_usize;

    loop {
        let read = read_upto(&mut file, buffer.get_mut(filled..).unwrap_or_default())?;
        filled += read;
        let slice = buffer.get(..filled).unwrap_or_default();

        match parse(slice) {
            Ok(addon) => {
                let offset = addon
                    .entries
                    .first()
                    .map_or(filled as u64, |entry| entry.offset as u64);
                return Ok((addon, offset));
            }
            Err(error) => {
                // Only truncation improves with more bytes; wrong magic or version never will.
                let truncated = matches!(
                    &error,
                    ExtensionError::Malformed { reason, .. }
                        if reason.contains("ends in the middle")
                            || reason.contains("not terminated")
                            || reason.contains("past the archive")
                );
                if !truncated || read == 0 && filled < buffer.len() {
                    return Err(error);
                }
                if filled == buffer.len() {
                    if buffer.len() >= INDEX_LIMIT {
                        return Err(ExtensionError::Malformed {
                            extension: "gmad",
                            reason: format!(
                                "the index is still not complete after {} bytes",
                                buffer.len()
                            ),
                        });
                    }
                    buffer.resize(buffer.len().saturating_mul(2), 0);
                } else if read == 0 {
                    return Err(error);
                }
            }
        }
    }
}

fn read_upto(file: &mut std::fs::File, buf: &mut [u8]) -> Result<usize, ExtensionError> {
    let mut total = 0;
    while total < buf.len() {
        let read = file.read(buf.get_mut(total..).unwrap_or_default())?;
        if read == 0 {
            break;
        }
        total += read;
    }
    Ok(total)
}

/// Validates every path up front, before anything is written.
fn safe_paths(addon: &Addon) -> Result<Vec<tapline_fs::SafePath>, ExtensionError> {
    addon
        .entries
        .iter()
        .map(|entry| {
            tapline_fs::validate_path(&entry.path).map_err(|reason| ExtensionError::UnsafePath {
                path: entry.path.clone(),
                reason: reason.to_string(),
            })
        })
        .collect()
}

/// Extracts an addon into `dest`, returning the paths written.
pub fn extract(archive: &Path, dest: &Path) -> Result<Vec<String>, ExtensionError> {
    let (addon, _) = read_index(archive)?;
    let safe = safe_paths(&addon)?;

    let mut file = std::fs::File::open(archive)?;
    let mut buffer = vec![0_u8; COPY_BUFFER];
    let mut written = Vec::with_capacity(addon.entries.len());

    for (entry, safe_path) in addon.entries.iter().zip(safe.iter()) {
        let target = safe_path.resolve(dest);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }

        file.seek(SeekFrom::Start(entry.offset as u64))?;
        let mut out = std::io::BufWriter::new(std::fs::File::create(&target)?);
        let mut remaining = entry.size;
        while remaining > 0 {
            let want = remaining.min(buffer.len() as u64) as usize;
            let slot = buffer.get_mut(..want).unwrap_or_default();
            file.read_exact(slot)?;
            out.write_all(slot)?;
            remaining -= want as u64;
        }
        out.flush()?;
        written.push(entry.path.clone());
    }

    Ok(written)
}

/// Converts an addon to a ZIP archive at `dest`.
pub fn to_zip(archive: &Path, dest: &Path, compress: bool) -> Result<usize, ExtensionError> {
    let (addon, _) = read_index(archive)?;
    let safe = safe_paths(&addon)?;

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::open(archive)?;
    let mut out = zip::Writer::new(std::io::BufWriter::new(std::fs::File::create(dest)?));

    // Batched so a huge addon is never fully resident; compressed across cores, written in order.
    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let mut index = 0;

    while index < addon.entries.len() {
        let mut batch: Vec<(String, Vec<u8>)> = Vec::new();
        let mut batch_bytes = 0_u64;

        while let Some(entry) = addon.entries.get(index) {
            // Always take at least one, or an oversized entry would never be taken.
            if !batch.is_empty() && batch_bytes.saturating_add(entry.size) > BATCH_BYTES {
                break;
            }
            let safe_path = safe.get(index).ok_or_else(|| {
                ExtensionError::Io("the validated path list is shorter than the index".to_owned())
            })?;

            file.seek(SeekFrom::Start(entry.offset as u64))?;
            let mut contents = vec![0_u8; entry.size as usize];
            file.read_exact(&mut contents)?;

            // The validator's spelling, so a hostile path cannot carry `..` onward.
            let name = safe_path.as_path().to_string_lossy().replace('\\', "/");
            batch.push((name, contents));
            batch_bytes = batch_bytes.saturating_add(entry.size);
            index += 1;
        }

        for prepared in compress_batch(batch, compress, threads) {
            out.add_prepared(prepared)?;
        }
    }

    out.finish()?;
    Ok(addon.entries.len())
}

/// Compresses a batch across cores, returning entries in input order.
pub(crate) fn compress_batch(
    batch: Vec<(String, Vec<u8>)>,
    compress: bool,
    threads: usize,
) -> Vec<zip::Prepared> {
    if !compress || threads < 2 || batch.len() < 2 {
        return batch
            .into_iter()
            .map(|(name, body)| zip::prepare(name, &body, compress))
            .collect();
    }

    let next = std::sync::atomic::AtomicUsize::new(0);
    let done: std::sync::Mutex<Vec<(usize, zip::Prepared)>> =
        std::sync::Mutex::new(Vec::with_capacity(batch.len()));
    let workers = threads.min(batch.len());

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                let mut local = Vec::new();
                loop {
                    let at = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some((name, body)) = batch.get(at) else {
                        break;
                    };
                    local.push((at, zip::prepare(name.clone(), body, true)));
                }
                if let Ok(mut done) = done.lock() {
                    done.append(&mut local);
                }
            });
        }
    });

    // Back into index order: entries and central directory must agree.
    let mut collected = done.into_inner().unwrap_or_default();
    collected.sort_by_key(|(at, _)| *at);
    collected
        .into_iter()
        .map(|(_, prepared)| prepared)
        .collect()
}

/// Unpacks a `.gma` into a directory beside it.
#[derive(Debug, Clone, Copy, Default)]
pub struct Extract {
    remove_original: bool,
}

impl Extract {
    /// An extractor that keeps the original archive.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            remove_original: false,
        }
    }

    /// Delete the `.gma` once it has been unpacked.
    #[must_use]
    pub const fn removing_original(mut self) -> Self {
        self.remove_original = true;
        self
    }
}

impl Extension for Extract {
    fn name(&self) -> &'static str {
        "gmad"
    }

    fn claims(&self, file: &Landed<'_>) -> bool {
        has_extension(file.path, "gma")
    }

    fn run(&self, file: &Landed<'_>) -> Result<Produced, ExtensionError> {
        let dest = unpack_dir(file);
        let names = extract(file.full_path, &dest)?;
        let prefix = dest
            .strip_prefix(file.root)
            .unwrap_or(&dest)
            .to_string_lossy()
            .replace('\\', "/");

        Ok(Produced {
            files: names
                .into_iter()
                .map(|name| format!("{prefix}/{name}"))
                .collect(),
            remove_original: self.remove_original,
        })
    }
}

/// Converts a `.gma` into a `.zip` beside it.
#[derive(Debug, Clone, Copy)]
pub struct ToZip {
    compress: bool,
    remove_original: bool,
}

impl Default for ToZip {
    fn default() -> Self {
        Self::new()
    }
}

impl ToZip {
    /// A converter that deflates, and keeps the original archive.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            compress: true,
            remove_original: false,
        }
    }

    /// Store entries instead of deflating them.
    #[must_use]
    pub const fn stored(mut self) -> Self {
        self.compress = false;
        self
    }

    /// Delete the `.gma` once it has been converted.
    #[must_use]
    pub const fn removing_original(mut self) -> Self {
        self.remove_original = true;
        self
    }
}

impl Extension for ToZip {
    fn name(&self) -> &'static str {
        "gmad-zip"
    }

    fn claims(&self, file: &Landed<'_>) -> bool {
        has_extension(file.path, "gma")
    }

    fn run(&self, file: &Landed<'_>) -> Result<Produced, ExtensionError> {
        let dest = file.full_path.with_extension("zip");
        to_zip(file.full_path, &dest, self.compress)?;
        let relative = dest
            .strip_prefix(file.root)
            .unwrap_or(&dest)
            .to_string_lossy()
            .replace('\\', "/");

        Ok(Produced {
            files: vec![relative],
            remove_original: self.remove_original,
        })
    }
}

/// Where a streamed archive is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamTarget<'a> {
    /// Unpack into a directory.
    Directory(&'a Path),
    /// Write a ZIP, deflating entries that get smaller for it.
    Zip(&'a Path),
    /// Write a ZIP without deflating; roughly four times faster.
    ZipStored(&'a Path),
}

/// What a streamed archive produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Streamed {
    /// How many entries were written.
    pub entries: usize,
    /// Their paths, for a directory target; empty for a ZIP.
    pub files: Vec<String>,
}

/// Consumes an archive as it arrives, writing it to `target`.
pub struct StreamWriter {
    inner: Inner,
}

enum Inner {
    Directory(split::Splitter<sink::ToDirectory>),
    Zip(split::Splitter<sink::ToZip>),
}

impl StreamWriter {
    /// A writer for `target`.
    pub fn new(target: StreamTarget<'_>) -> Result<Self, ExtensionError> {
        let inner = match target {
            StreamTarget::Directory(dest) => {
                Inner::Directory(split::Splitter::new(sink::ToDirectory::new(dest)))
            }
            StreamTarget::Zip(dest) => {
                Inner::Zip(split::Splitter::new(sink::ToZip::new(dest, true)?))
            }
            StreamTarget::ZipStored(dest) => {
                Inner::Zip(split::Splitter::new(sink::ToZip::new(dest, false)?))
            }
        };
        Ok(Self { inner })
    }

    /// Feeds the next bytes of the archive, in order.
    pub fn push(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        match &mut self.inner {
            Inner::Directory(splitter) => splitter.push(bytes),
            Inner::Zip(splitter) => splitter.push(bytes),
        }
    }

    /// The archive's metadata, once its index has arrived.
    #[must_use]
    pub fn addon(&self) -> Option<&Addon> {
        match &self.inner {
            Inner::Directory(splitter) => splitter.addon(),
            Inner::Zip(splitter) => splitter.addon(),
        }
    }

    /// Finishes, closing whatever was being written.
    pub fn finish(self) -> Result<Streamed, ExtensionError> {
        match self.inner {
            Inner::Directory(splitter) => {
                let files = splitter.finish()?.into_produced();
                Ok(Streamed {
                    entries: files.len(),
                    files,
                })
            }
            Inner::Zip(splitter) => {
                // `close` writes the central directory; without it most readers refuse the file.
                let entries = splitter.finish()?.close()?;
                Ok(Streamed {
                    entries,
                    files: Vec::new(),
                })
            }
        }
    }
}

/// Where a GMAD's index lives.
#[must_use]
pub const fn index_location() -> IndexLocation {
    IndexLocation::Head(64 * 1024)
}

/// Reads an index out of the bytes [`index_location`] asked for.
pub fn plan(head: &[u8]) -> Result<Vec<tapline_ext::ArchiveEntry>, ExtensionError> {
    Ok(parse_index(head)?
        .entries
        .into_iter()
        .map(|entry| tapline_ext::ArchiveEntry::stored(entry.path, entry.offset as u64, entry.size))
        .collect())
}

fn has_extension(path: &str, extension: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|found| found.eq_ignore_ascii_case(extension))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_gma_files_are_claimed() {
        let root = Path::new("/srv");
        let file = |path: &'static str| Landed {
            app: tapline_ids::AppId(4000),
            root,
            path,
            full_path: Path::new(path),
            bytes: 1,
        };
        let extract = Extract::new();
        assert!(extract.claims(&file("addons/x.gma")));
        assert!(extract.claims(&file("addons/X.GMA")));
        assert!(!extract.claims(&file("addons/x.zip")));
        assert!(!extract.claims(&file("addons/x.dupe")));
        assert!(!extract.claims(&file("gma")));

        assert!(ToZip::new().claims(&file("addons/x.gma")));
    }

    #[test]
    fn the_names_are_stable() {
        assert_eq!(Extract::new().name(), "gmad");
        assert_eq!(ToZip::new().name(), "gmad-zip");
    }

    #[test]
    fn keeping_the_original_is_the_default() {
        assert!(!Extract::new().remove_original);
        assert!(!ToZip::new().remove_original);
        assert!(Extract::new().removing_original().remove_original);
        assert!(ToZip::new().removing_original().remove_original);
    }
}
