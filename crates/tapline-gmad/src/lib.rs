//! Garry's Mod addon archives.
//!
//! A Workshop addon for app 4000 arrives as a single `.gma`. A server that
//! mounts a Workshop collection reads that directly, but anything that wants
//! the files — a content pipeline, a fast-download host, a human — needs it
//! unpacked. This crate reads the format, extracts it, and ships two
//! [`Extension`]s that do so as files land.
//!
//! ```no_run
//! # fn example() -> Result<(), tapline_ext::ExtensionError> {
//! use tapline_gmad::{Extract, ToZip};
//! // Registered with a session; see the tapline crate.
//! let unpack = Extract::new();
//! let zip = ToZip::new();
//! # let _ = (unpack, zip);
//! # Ok(())
//! # }
//! ```
//!
//! # The untrusted part
//!
//! Every path in an addon is chosen by whoever published it, and anyone can
//! publish a Workshop item. `lua/../../../etc/cron.d/x` is a legal string. Each
//! one goes through `tapline-fs`'s validator before it is used, and a path that
//! escapes fails the whole extraction rather than being skipped — a partially
//! extracted addon that quietly dropped the interesting file is worse than one
//! that refused.

#![forbid(unsafe_code)]

mod format;
mod stream;
mod zip;

pub use format::{Addon, Entry, MAGIC, parse, parse_index};
pub use stream::StreamingExtractor;

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use tapline_ext::{Extension, ExtensionError, Landed, Produced, unpack_dir};

/// How much of an archive to read before deciding the index did not fit.
///
/// The header and index of a real addon are kilobytes; PAC3's are under two.
/// Reading a bounded prefix and growing only if needed keeps a 400 MB addon
/// from being 400 MB of memory just to learn what is in it.
const INDEX_PROBE: usize = 1 << 16;

/// The largest header this will read before giving up.
///
/// An index that has not terminated within this is not a large addon, it is a
/// malformed one, and continuing to grow the buffer is how a 30-byte file
/// becomes an out-of-memory.
const INDEX_LIMIT: usize = 1 << 26;

/// Buffer used when streaming contents out.
const COPY_BUFFER: usize = 1 << 20;

/// How many bytes of entries to hold in memory while compressing a batch.
///
/// The bound that keeps a 400 MB addon from being 400 MB of resident memory.
const BATCH_BYTES: u64 = 32 << 20;

/// Reads an addon's header and index from a file without reading the contents.
///
/// Returns the addon and the byte offset its contents begin at.
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
                // Only "it ends too early" is worth reading further for. A wrong
                // magic or a bad version will not improve with more bytes.
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

/// Reads as much as it can into `buf`, returning how many bytes landed.
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

/// Validates every path in an addon before anything is written.
///
/// All of them, up front. Checking as it goes would mean a hostile addon gets
/// half of its files written before the one that escapes is refused.
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

/// Extracts an addon into `dest`.
///
/// Returns the paths written, relative to `dest`.
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
///
/// Returns the number of entries written.
pub fn to_zip(archive: &Path, dest: &Path, compress: bool) -> Result<usize, ExtensionError> {
    let (addon, _) = read_index(archive)?;
    let safe = safe_paths(&addon)?;

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::open(archive)?;
    let mut out = zip::Writer::new(std::io::BufWriter::new(std::fs::File::create(dest)?));

    // Deflate is the entire cost here: measured on a real addon it was 175 ms
    // against 10 ms to store the same bytes. So entries are read in batches,
    // compressed across every core, and written back in index order.
    //
    // Batched rather than all at once because an addon can be hundreds of
    // megabytes, and holding all of it plus all of its compressed output is a
    // way to turn a download into an out-of-memory.
    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let mut index = 0;

    while index < addon.entries.len() {
        let mut batch: Vec<(String, Vec<u8>)> = Vec::new();
        let mut batch_bytes = 0_u64;

        while let Some(entry) = addon.entries.get(index) {
            // Always take at least one, or an entry larger than the batch
            // budget would never be taken at all.
            if !batch.is_empty() && batch_bytes.saturating_add(entry.size) > BATCH_BYTES {
                break;
            }
            let safe_path = safe.get(index).ok_or_else(|| {
                ExtensionError::Io("the validated path list is shorter than the index".to_owned())
            })?;

            file.seek(SeekFrom::Start(entry.offset as u64))?;
            let mut contents = vec![0_u8; entry.size as usize];
            file.read_exact(&mut contents)?;

            // The validator's spelling of the path, not the archive's, so a ZIP
            // built from a hostile addon cannot carry `..` into whatever
            // unpacks it next.
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

/// Compresses a batch across every core, returning entries in input order.
///
/// Dynamic scheduling through a shared counter rather than splitting the batch
/// into equal slices: an addon's files differ in size by orders of magnitude,
/// and a static split leaves one thread with the 4 MB model while the others
/// finish their `.lua` files and idle.
fn compress_batch(
    batch: Vec<(String, Vec<u8>)>,
    compress: bool,
    threads: usize,
) -> Vec<zip::Prepared> {
    // Not worth a thread: storing is a memcpy, and one entry has nothing to
    // spread.
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
                // Collected locally and merged once, so the lock is taken a
                // handful of times rather than once per file.
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

    // Back into index order: a ZIP's entries and its central directory must
    // agree, and the order here is the order the offsets were assigned in.
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
    /// Whether to delete the `.gma` afterwards.
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
    ///
    /// Much faster, and the right choice when the result is going straight to a
    /// fast-download host that compresses on the wire anyway.
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

/// Case-insensitive extension check.
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
        // Case matters on the Workshop, where publishers name files by hand.
        assert!(extract.claims(&file("addons/X.GMA")));
        assert!(!extract.claims(&file("addons/x.zip")));
        assert!(!extract.claims(&file("addons/x.dupe")));
        assert!(!extract.claims(&file("gma")));

        assert!(ToZip::new().claims(&file("addons/x.gma")));
    }

    #[test]
    fn the_names_are_stable() {
        // They are how an extension is selected from a command line and across
        // the C ABI, so renaming one breaks callers.
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
