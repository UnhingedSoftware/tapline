//! Reading a ZIP that has not been downloaded.
//!
//! The format that motivated ranged reads. A ZIP keeps its index — the central
//! directory — at the **end**, so nothing sequential can know what is inside
//! until it has read all of it. Given the ability to fetch a range, that stops
//! being a limitation and becomes a read from the tail.
//!
//! # The three parts
//!
//! ```text
//! local header + data   per entry, in whatever order the writer chose
//! central directory     one record per entry, with each local header's offset
//! end of central dir    where the directory starts, and how many entries
//! ```
//!
//! Read the tail, find the end record, and the directory tells you every name,
//! size and offset. Which is why this needs two phases: the directory points at
//! a **local header**, and that header's own length depends on fields only it
//! carries, so an entry's data offset is one more read away. [`plan`] says what
//! it needs; [`finalize`] finishes the job.
//!
//! # What this is not
//!
//! Not a general ZIP library. No ZIP64, no encryption, no multi-disk, no
//! methods beyond stored and deflate. Each of those is refused by name rather
//! than guessed at, because a reader that half-supports a format produces
//! files that are wrong rather than missing.

#![forbid(unsafe_code)]

use tapline_ext::{ArchiveEntry, Compression, ExtensionError, IndexLocation, IndexPlan};

/// End of central directory signature.
const END_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
/// Central directory record signature.
const CENTRAL_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
/// Local file header signature.
const LOCAL_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];

/// Fixed part of a local header, before its name and extra fields.
const LOCAL_HEADER: u64 = 30;

/// Stored, no compression.
const STORE: u16 = 0;
/// Deflate.
const DEFLATE: u16 = 8;

/// How much of the tail to read to find the end record.
///
/// The end record is 22 bytes plus a comment of up to 65,535, so 64 KiB + a
/// little always contains it. That is one chunk, whatever the archive's size.
#[must_use]
pub const fn index_location() -> IndexLocation {
    IndexLocation::Tail(66 * 1024)
}

/// Finds the end-of-central-directory record.
///
/// Backwards, because the record is near the end and an arbitrary comment
/// follows it. Backwards alone is not enough: the comment may itself contain
/// the signature, and it sits **after** the real record, so taking the last
/// match takes the decoy. Every candidate is checked instead — a real record's
/// comment length reaches exactly the end of the file, and a decoy's does not.
///
/// Found by reading an archive built elsewhere with a comment containing
/// `PK\x05\x06`. A fixture written by this crate would never have contained one.
fn find_end_record(tail: &[u8]) -> Option<usize> {
    let mut at = tail.len().checked_sub(4)?;
    loop {
        if tail.get(at..at.saturating_add(4)) == Some(&END_SIGNATURE) {
            // A candidate that cannot even hold a full record is not one. This
            // must reject the candidate and keep looking, not abandon the
            // search: a `?` here walked out of the whole function the first
            // time a decoy appeared too near the end to read, and reported no
            // end record at all for an archive that plainly had one.
            let plausible = tail
                .get(at..)
                .and_then(|rest| u16_at(rest, 20))
                .and_then(|comment_len| {
                    // 22 bytes of record, then exactly the comment it declares.
                    at.checked_add(22)?
                        .checked_add(comment_len as usize)
                        .map(|end| end == tail.len())
                })
                .unwrap_or(false);
            if plausible {
                return Some(at);
            }
        }
        at = at.checked_sub(1)?;
    }
}

fn short(what: &str) -> ExtensionError {
    ExtensionError::Malformed {
        extension: "zip",
        reason: format!("the archive ends in the middle of its {what}"),
    }
}

fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let end = at.checked_add(2)?;
    Some(u16::from_le_bytes(bytes.get(at..end)?.try_into().ok()?))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let end = at.checked_add(4)?;
    Some(u32::from_le_bytes(bytes.get(at..end)?.try_into().ok()?))
}

/// Reads the index out of the archive's tail.
///
/// `tail` is the last bytes of the file and `tail_start` is where they begin,
/// so offsets can be translated into the whole file's coordinates.
///
/// The entries come back with their **local header** offsets and the ranges
/// needed to turn those into data offsets; see [`finalize`].
pub fn plan(tail: &[u8], tail_start: u64) -> Result<IndexPlan, ExtensionError> {
    let end_at = find_end_record(tail).ok_or(ExtensionError::Malformed {
        extension: "zip",
        reason: "no end-of-central-directory record in the last 66 KiB; \
                 not a ZIP, or a ZIP64 one"
            .to_owned(),
    })?;

    let end = tail.get(end_at..).ok_or_else(|| short("end record"))?;
    let entry_count = u16_at(end, 10).ok_or_else(|| short("end record"))? as usize;
    let directory_size = u32_at(end, 12).ok_or_else(|| short("end record"))? as u64;
    let directory_start = u64::from(u32_at(end, 16).ok_or_else(|| short("end record"))?);

    if directory_start == u64::from(u32::MAX) || entry_count == u16::MAX as usize {
        return Err(ExtensionError::Malformed {
            extension: "zip",
            reason: "this archive needs ZIP64, which this reader does not implement".to_owned(),
        });
    }

    // The directory may start before the tail we were given. Ask for it.
    if directory_start < tail_start {
        return Ok(IndexPlan {
            entries: Vec::new(),
            needs: vec![(directory_start, directory_size)],
        });
    }

    let offset_in_tail = (directory_start - tail_start) as usize;
    let directory = tail
        .get(offset_in_tail..)
        .ok_or_else(|| short("central directory"))?;
    read_directory(directory, entry_count)
}

/// Reads a central directory into entries plus the local-header ranges needed.
pub fn read_directory(directory: &[u8], expected: usize) -> Result<IndexPlan, ExtensionError> {
    let mut entries = Vec::with_capacity(expected);
    let mut needs = Vec::with_capacity(expected);
    let mut at = 0_usize;

    while entries.len() < expected {
        let record = directory
            .get(at..)
            .ok_or_else(|| short("central directory"))?;
        if record.get(..4) != Some(&CENTRAL_SIGNATURE) {
            return Err(ExtensionError::Malformed {
                extension: "zip",
                reason: format!(
                    "expected a central directory record at {at}, found {:?}",
                    record.get(..4).unwrap_or_default()
                ),
            });
        }

        let flags = u16_at(record, 8).ok_or_else(|| short("central directory"))?;
        let method = u16_at(record, 10).ok_or_else(|| short("central directory"))?;
        let stored_size = u64::from(u32_at(record, 20).ok_or_else(|| short("central directory"))?);
        let size = u64::from(u32_at(record, 24).ok_or_else(|| short("central directory"))?);
        let name_len = u16_at(record, 28).ok_or_else(|| short("central directory"))? as usize;
        let extra_len = u16_at(record, 30).ok_or_else(|| short("central directory"))? as usize;
        let comment_len = u16_at(record, 32).ok_or_else(|| short("central directory"))? as usize;
        let local = u64::from(u32_at(record, 42).ok_or_else(|| short("central directory"))?);

        // Bit 0 is encryption. A reader that ignored it would write ciphertext
        // to disk and call it a file.
        if flags & 1 != 0 {
            return Err(ExtensionError::Malformed {
                extension: "zip",
                reason: "the archive is encrypted, which this reader does not implement".to_owned(),
            });
        }

        let compression = match method {
            STORE => Compression::Stored,
            DEFLATE => Compression::Deflate,
            other => {
                return Err(ExtensionError::Malformed {
                    extension: "zip",
                    reason: format!(
                        "entry uses compression method {other}, and this reader knows \
                         only stored and deflate"
                    ),
                });
            }
        };

        let name_at = 46;
        let name_bytes = record
            .get(name_at..name_at + name_len)
            .ok_or_else(|| short("an entry name"))?;
        // Lossy on purpose: a name that is not UTF-8 is still a file that has
        // to be called something, and the path validator sees the result.
        let path = String::from_utf8_lossy(name_bytes).into_owned();

        entries.push(ArchiveEntry {
            path,
            size,
            // The local header for now; `finalize` turns it into the data.
            offset: local,
            stored_size,
            compression,
        });
        // Enough of the local header to read its own name and extra lengths.
        needs.push((local, LOCAL_HEADER));

        at = at
            .checked_add(name_at + name_len + extra_len + comment_len)
            .ok_or_else(|| short("central directory"))?;
    }

    Ok(IndexPlan { entries, needs })
}

/// Turns local-header offsets into data offsets.
///
/// `headers` are the ranges [`plan`] asked for, in the same order as the
/// entries.
pub fn finalize(
    mut entries: Vec<ArchiveEntry>,
    headers: &[Vec<u8>],
) -> Result<Vec<ArchiveEntry>, ExtensionError> {
    if headers.len() != entries.len() {
        return Err(ExtensionError::Malformed {
            extension: "zip",
            reason: format!(
                "asked for {} local headers and got {}",
                entries.len(),
                headers.len()
            ),
        });
    }

    for (entry, header) in entries.iter_mut().zip(headers.iter()) {
        if header.get(..4) != Some(&LOCAL_SIGNATURE) {
            return Err(ExtensionError::Malformed {
                extension: "zip",
                reason: format!(
                    "{:?} has no local header where the directory said it would",
                    entry.path
                ),
            });
        }
        // The local header's name and extra lengths are its own, and need not
        // match the central directory's — which is exactly why this second read
        // exists rather than assuming.
        let name_len = u64::from(u16_at(header, 26).ok_or_else(|| short("a local header"))?);
        let extra_len = u64::from(u16_at(header, 28).ok_or_else(|| short("a local header"))?);
        entry.offset = entry
            .offset
            .checked_add(LOCAL_HEADER + name_len + extra_len)
            .ok_or_else(|| short("a local header"))?;
    }
    Ok(entries)
}

/// Unpacks an entry's stored bytes.
pub fn decode(entry: &ArchiveEntry, stored: &[u8]) -> Result<Vec<u8>, ExtensionError> {
    match entry.compression {
        Compression::Stored => Ok(stored.to_vec()),
        Compression::Deflate => {
            let limit = usize::try_from(entry.size).map_err(|_| ExtensionError::Malformed {
                extension: "zip",
                reason: format!(
                    "{:?} claims {} bytes, which will not fit",
                    entry.path, entry.size
                ),
            })?;
            miniz_oxide::inflate::decompress_to_vec_with_limit(stored, limit).map_err(|error| {
                ExtensionError::Malformed {
                    extension: "zip",
                    reason: format!("{:?} would not inflate: {:?}", entry.path, error.status),
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a ZIP the way a writer would, so the reader can be tested against
    /// bytes rather than against itself.
    fn build(files: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut records = Vec::new();

        for (name, body, compress) in files {
            let offset = out.len() as u32;
            let deflated = compress
                .then(|| miniz_oxide::deflate::compress_to_vec(body, 6))
                .filter(|candidate| candidate.len() < body.len());
            let (method, payload): (u16, &[u8]) = match &deflated {
                Some(bytes) => (DEFLATE, bytes),
                None => (STORE, body),
            };
            let crc = crc32fast::hash(body);

            out.extend_from_slice(&LOCAL_SIGNATURE);
            out.extend_from_slice(&20_u16.to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
            out.extend_from_slice(&method.to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            // A non-empty extra field, because a reader that assumed the local
            // header matched the central one would pass without it.
            out.extend_from_slice(&4_u16.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);
            out.extend_from_slice(payload);

            records.push((
                name.to_string(),
                method,
                crc,
                payload.len() as u32,
                body.len() as u32,
                offset,
            ));
        }

        let directory_start = out.len() as u32;
        for (name, method, crc, compressed, uncompressed, offset) in &records {
            out.extend_from_slice(&CENTRAL_SIGNATURE);
            out.extend_from_slice(&20_u16.to_le_bytes());
            out.extend_from_slice(&20_u16.to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
            out.extend_from_slice(&method.to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&compressed.to_le_bytes());
            out.extend_from_slice(&uncompressed.to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
            out.extend_from_slice(&0_u16.to_le_bytes());
            out.extend_from_slice(&0_u32.to_le_bytes());
            out.extend_from_slice(&offset.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
        }
        let directory_size = out.len() as u32 - directory_start;

        out.extend_from_slice(&END_SIGNATURE);
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&(records.len() as u16).to_le_bytes());
        out.extend_from_slice(&(records.len() as u16).to_le_bytes());
        out.extend_from_slice(&directory_size.to_le_bytes());
        out.extend_from_slice(&directory_start.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out
    }

    /// Reads an archive the way the pipeline does: tail, plan, headers, finalize.
    fn read(raw: &[u8]) -> Vec<ArchiveEntry> {
        let IndexLocation::Tail(want) = index_location() else {
            panic!("zip should read from the tail");
        };
        let start = raw.len().saturating_sub(want as usize);
        let plan = plan(raw.get(start..).expect("tail"), start as u64).expect("plan");
        assert!(!plan.is_complete(), "a zip always needs its local headers");

        let headers: Vec<Vec<u8>> = plan
            .needs
            .iter()
            .map(|(offset, len)| {
                raw.get(*offset as usize..(*offset + *len) as usize)
                    .expect("header in range")
                    .to_vec()
            })
            .collect();
        finalize(plan.entries, &headers).expect("finalize")
    }

    #[test]
    fn the_index_is_read_from_the_tail() {
        let raw = build(&[("a.txt", b"one", false), ("b/c.txt", b"two", false)]);
        let entries = read(&raw);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "a.txt");
        assert_eq!(entries[1].path, "b/c.txt");
    }

    #[test]
    fn an_entry_can_be_cut_out_by_its_offset() {
        // The whole point: read only these bytes and get the file.
        let raw = build(&[("a.txt", b"hello", false), ("b.txt", b"world!", false)]);
        let entries = read(&raw);
        for (entry, expected) in entries.iter().zip([&b"hello"[..], &b"world!"[..]]) {
            let stored = raw
                .get(entry.offset as usize..(entry.offset + entry.stored_size) as usize)
                .expect("in range");
            assert_eq!(decode(entry, stored).expect("decode"), expected);
        }
    }

    #[test]
    fn a_deflated_entry_inflates_to_its_original() {
        let body = vec![b'x'; 10_000];
        let raw = build(&[("big.txt", &body, true)]);
        let entries = read(&raw);
        assert_eq!(entries[0].compression, Compression::Deflate);
        assert!(
            entries[0].stored_size < entries[0].size,
            "deflate did not shrink it"
        );

        let stored = raw
            .get(entries[0].offset as usize..(entries[0].offset + entries[0].stored_size) as usize)
            .expect("in range");
        assert_eq!(decode(&entries[0], stored).expect("decode"), body);
    }

    #[test]
    fn the_local_headers_extra_field_is_honoured() {
        // The fixture writes a 4-byte extra field into every local header and
        // none into the central directory. A reader that took the data offset
        // from the central record alone would be four bytes early on every
        // entry — and would produce files that are wrong rather than missing.
        let raw = build(&[("a.txt", b"exact", false)]);
        let entries = read(&raw);
        let stored = raw
            .get(entries[0].offset as usize..(entries[0].offset + entries[0].stored_size) as usize)
            .expect("in range");
        assert_eq!(stored, b"exact");
    }

    #[test]
    fn an_archive_with_no_end_record_is_refused() {
        let error = plan(b"not a zip at all", 0).expect_err("must refuse");
        assert!(
            error.to_string().contains("no end-of-central-directory"),
            "{error}"
        );
    }

    #[test]
    fn an_unknown_compression_method_names_itself() {
        let mut raw = build(&[("a.txt", b"body", false)]);
        // Method lives at offset 10 of the central record, which is at the
        // directory start recorded in the end record.
        let end_at = raw
            .windows(4)
            .rposition(|w| w == END_SIGNATURE)
            .expect("end record");
        let directory_start = u32_at(raw.get(end_at..).expect("end"), 16).expect("offset") as usize;
        raw.splice(
            directory_start + 10..directory_start + 12,
            12_u16.to_le_bytes(),
        );

        let start = raw.len().saturating_sub(66 * 1024);
        let error = plan(raw.get(start..).expect("tail"), start as u64).expect_err("must refuse");
        assert!(error.to_string().contains("method 12"), "{error}");
    }

    #[test]
    fn an_encrypted_archive_is_refused_rather_than_written_as_ciphertext() {
        let mut raw = build(&[("a.txt", b"body", false)]);
        let end_at = raw
            .windows(4)
            .rposition(|w| w == END_SIGNATURE)
            .expect("end record");
        let directory_start = u32_at(raw.get(end_at..).expect("end"), 16).expect("offset") as usize;
        // Flags at offset 8, bit 0.
        raw.splice(
            directory_start + 8..directory_start + 10,
            1_u16.to_le_bytes(),
        );

        let start = raw.len().saturating_sub(66 * 1024);
        let error = plan(raw.get(start..).expect("tail"), start as u64).expect_err("must refuse");
        assert!(error.to_string().contains("encrypted"), "{error}");
    }

    #[test]
    fn an_empty_archive_reads_as_empty() {
        let raw = build(&[]);
        let start = raw.len().saturating_sub(66 * 1024);
        let plan = plan(raw.get(start..).expect("tail"), start as u64).expect("plan");
        assert!(plan.entries.is_empty());
        assert!(plan.is_complete(), "nothing to resolve for no entries");
    }

    #[test]
    fn a_directory_before_the_tail_is_asked_for() {
        // A large archive keeps its directory far from the end. The reader must
        // say so rather than read past the buffer it was given.
        let raw = build(&[("a.txt", b"one", false)]);
        // Pretend the tail we hold starts after the directory does.
        let tail_start = raw.len() as u64 - 30;
        let plan = plan(raw.get(tail_start as usize..).expect("tail"), tail_start);
        match plan {
            Ok(plan) => {
                assert!(
                    !plan.needs.is_empty(),
                    "should have asked for the directory"
                );
                assert!(plan.entries.is_empty());
            }
            // Also acceptable: the end record itself was outside the window.
            Err(error) => assert!(error.to_string().contains("end-of-central-directory")),
        }
    }

    #[test]
    fn truncation_is_an_error_and_never_a_panic() {
        let raw = build(&[("a.txt", b"one", false), ("b.txt", b"two", true)]);
        for cut in 0..raw.len() {
            let prefix = raw.get(..cut).expect("in range");
            let _ = plan(prefix, 0);
        }
    }
}
