//! A minimal ZIP writer.
//!
//! Internal, and deliberately not a general ZIP library. It writes what this
//! crate needs — a flat set of files, stored or deflated — and refuses
//! everything it does not implement rather than producing an archive that only
//! some readers accept.
//!
//! Notably **no ZIP64**. The 32-bit fields cap an archive at 4 GiB and 65,535
//! entries, which no Garry's Mod addon approaches; the Workshop's own limit is
//! far below it. Exceeding either is an error that says so. Writing the 32-bit
//! format with truncated sizes and hoping is how an archive becomes one that
//! `unzip` opens and Windows does not.
//!
//! The layout, which is the same three parts in every ZIP:
//!
//! ```text
//! local header + data   per entry, in order
//! central directory     one record per entry, repeating the metadata
//! end of central dir    where the directory starts, and how many entries
//! ```

use std::io::Write;
use tapline_ext::ExtensionError;

/// Local file header signature.
const LOCAL_SIGNATURE: u32 = 0x0403_4b50;
/// Central directory record signature.
const CENTRAL_SIGNATURE: u32 = 0x0201_4b50;
/// End of central directory signature.
const END_SIGNATURE: u32 = 0x0605_4b50;

/// Stored, no compression.
const STORE: u16 = 0;
/// Deflate.
const DEFLATE: u16 = 8;

/// The version needed to extract: 2.0, which is what deflate requires.
const VERSION: u16 = 20;

/// What a 32-bit ZIP can address.
const MAX_SIZE: u64 = u32::MAX as u64;
/// What the end-of-central-directory record can count.
const MAX_ENTRIES: usize = u16::MAX as usize;

/// Deflate level. 6 is zlib's default: the knee of the ratio/time curve.
const LEVEL: u8 = 6;

/// A pre-compressed entry, ready to write.
pub struct Prepared {
    /// The entry's path inside the archive.
    pub name: String,
    /// CRC-32 of the *uncompressed* bytes, which is what a reader checks.
    pub crc: u32,
    /// Uncompressed length.
    pub uncompressed: usize,
    /// What actually goes in the file: deflated, or the original.
    pub payload: Vec<u8>,
    /// Which of those it is.
    pub method: u16,
}

/// Compresses one entry, deciding whether deflating it was worth it.
///
/// Pure and self-contained so it can run on any thread. Deflate is the whole
/// cost of building a ZIP — measured on PAC3 (348 files, 8.69 MB) it was
/// 175 ms against 10 ms to store the same bytes — so this is the function
/// worth running many of at once. Spreading it across cores took that to
/// 39 ms, for byte-identical output.
#[must_use]
pub fn prepare(name: String, body: &[u8], compress: bool) -> Prepared {
    let crc = crc32fast::hash(body);
    let deflated = compress
        .then(|| miniz_oxide::deflate::compress_to_vec(body, LEVEL))
        // Only when it actually helped. A Garry's Mod addon is largely .vtf and
        // .mdl, which are already compressed, and storing those keeps both the
        // archive and the time down.
        .filter(|candidate| candidate.len() < body.len());

    match deflated {
        Some(payload) => Prepared {
            name,
            crc,
            uncompressed: body.len(),
            payload,
            method: DEFLATE,
        },
        None => Prepared {
            name,
            crc,
            uncompressed: body.len(),
            payload: body.to_vec(),
            method: STORE,
        },
    }
}

/// One entry, remembered so the central directory can be written at the end.
struct Record {
    name: String,
    crc: u32,
    compressed: u32,
    uncompressed: u32,
    method: u16,
    offset: u32,
}

/// Writes a ZIP archive.
pub struct Writer<W: Write> {
    out: W,
    records: Vec<Record>,
    offset: u64,
}

impl<W: Write> Writer<W> {
    /// A writer over `out`.
    pub const fn new(out: W) -> Self {
        Self {
            out,
            records: Vec::new(),
            offset: 0,
        }
    }

    /// Adds an entry compressed elsewhere, possibly on another thread.
    pub fn add_prepared(&mut self, entry: Prepared) -> Result<(), ExtensionError> {
        let Prepared {
            name,
            crc,
            uncompressed,
            payload,
            method,
        } = entry;
        self.write_entry(&name, crc, uncompressed, &payload, method)
    }

    fn write_entry(
        &mut self,
        name: &str,
        crc: u32,
        uncompressed_len: usize,
        payload: &[u8],
        method: u16,
    ) -> Result<(), ExtensionError> {
        let body_len = uncompressed_len;
        let _ = &body_len;
        if self.records.len() >= MAX_ENTRIES {
            return Err(ExtensionError::Malformed {
                extension: "gmad-zip",
                reason: format!("more than {MAX_ENTRIES} files; this writer does not do ZIP64"),
            });
        }
        let uncompressed = u32::try_from(uncompressed_len).map_err(|_| ExtensionError::Malformed {
            extension: "gmad-zip",
            reason: format!(
                "{name:?} is {uncompressed_len} bytes, past the {MAX_SIZE} a 32-bit ZIP can address"
            ),
        })?;

        let compressed = u32::try_from(payload.len()).map_err(|_| ExtensionError::Malformed {
            extension: "gmad-zip",
            reason: format!("{name:?} does not fit a 32-bit ZIP"),
        })?;
        let offset = u32::try_from(self.offset).map_err(|_| ExtensionError::Malformed {
            extension: "gmad-zip",
            reason: format!(
                "the archive passed {MAX_SIZE} bytes at {name:?}; this writer does not do ZIP64"
            ),
        })?;

        let name_bytes = name.as_bytes();
        let name_len = u16::try_from(name_bytes.len()).map_err(|_| ExtensionError::Malformed {
            extension: "gmad-zip",
            reason: format!("the path {name:?} is longer than a ZIP name field allows"),
        })?;

        // Local header.
        self.u32(LOCAL_SIGNATURE)?;
        self.u16(VERSION)?;
        self.u16(0)?; // flags
        self.u16(method)?;
        self.u16(0)?; // time
        self.u16(0)?; // date
        self.u32(crc)?;
        self.u32(compressed)?;
        self.u32(uncompressed)?;
        self.u16(name_len)?;
        self.u16(0)?; // extra length
        self.write(name_bytes)?;
        self.write(payload)?;

        self.records.push(Record {
            name: name.to_owned(),
            crc,
            compressed,
            uncompressed,
            method,
            offset,
        });
        Ok(())
    }

    /// Writes the central directory and closes the archive.
    pub fn finish(mut self) -> Result<W, ExtensionError> {
        let directory_start = self.offset;

        // Taken out so the per-record write can borrow `self` mutably without
        // fighting the iteration over `self.records`.
        let records = std::mem::take(&mut self.records);
        for record in &records {
            self.write_central_header(record)?;
        }

        let directory_size = self.offset.saturating_sub(directory_start);
        let count = u16::try_from(records.len()).unwrap_or(u16::MAX);

        self.u32(END_SIGNATURE)?;
        self.u16(0)?; // this disk
        self.u16(0)?; // disk with the directory
        self.u16(count)?;
        self.u16(count)?;
        self.u32(u32::try_from(directory_size).unwrap_or(u32::MAX))?;
        self.u32(u32::try_from(directory_start).unwrap_or(u32::MAX))?;
        self.u16(0)?; // comment length

        self.out.flush()?;
        Ok(self.out)
    }

    /// Writes one entry's central-directory header — the fixed 46-byte record
    /// followed by the name. The whole ZIP central-directory layout lives here.
    fn write_central_header(&mut self, record: &Record) -> Result<(), ExtensionError> {
        let name_len = u16::try_from(record.name.len()).unwrap_or(u16::MAX);

        self.u32(CENTRAL_SIGNATURE)?;
        self.u16(VERSION)?; // version made by
        self.u16(VERSION)?; // version needed
        self.u16(0)?; // flags
        self.u16(record.method)?;
        self.u16(0)?; // time
        self.u16(0)?; // date
        self.u32(record.crc)?;
        self.u32(record.compressed)?;
        self.u32(record.uncompressed)?;
        self.u16(name_len)?;
        self.u16(0)?; // extra
        self.u16(0)?; // comment
        self.u16(0)?; // disk number
        self.u16(0)?; // internal attributes
        self.u32(0)?; // external attributes
        self.u32(record.offset)?;
        self.write(record.name.as_bytes())
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        self.out.write_all(bytes)?;
        self.offset = self.offset.saturating_add(bytes.len() as u64);
        Ok(())
    }

    fn u16(&mut self, value: u16) -> Result<(), ExtensionError> {
        self.write(&value.to_le_bytes())
    }

    fn u32(&mut self, value: u32) -> Result<(), ExtensionError> {
        self.write(&value.to_le_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(files: &[(&str, &[u8])], compress: bool) -> Vec<u8> {
        let mut writer = Writer::new(Vec::new());
        for (name, body) in files {
            writer
                .add_prepared(prepare((*name).to_owned(), body, compress))
                .expect("must add");
        }
        writer.finish().expect("must finish")
    }

    #[test]
    fn an_archive_has_the_three_signatures_in_order() {
        let zip = build(&[("a.txt", b"hello")], false);
        assert_eq!(
            zip.get(..4),
            Some(&LOCAL_SIGNATURE.to_le_bytes()[..]),
            "no local header"
        );
        let central = CENTRAL_SIGNATURE.to_le_bytes();
        let end = END_SIGNATURE.to_le_bytes();
        let central_at = zip
            .windows(4)
            .position(|w| w == central)
            .expect("no central directory");
        let end_at = zip
            .windows(4)
            .position(|w| w == end)
            .expect("no end record");
        assert!(central_at < end_at, "the directory must precede the end");
    }

    #[test]
    fn a_stored_entry_holds_its_bytes_verbatim() {
        let zip = build(&[("a.txt", b"hello world")], false);
        let at = zip
            .windows(11)
            .position(|w| w == b"hello world")
            .expect("the body is not in the archive");
        assert!(at > 0);
    }

    #[test]
    fn deflate_is_skipped_when_it_would_not_help() {
        // Random-ish bytes do not compress, and a ZIP that stored them as
        // "deflated but larger" is bigger for no reason.
        // xorshift32, which produces bytes deflate cannot model. An earlier
        // version of this test used a multiply-and-shift pattern that looked
        // random and compressed by 40%, so it asserted the opposite of what it
        // meant to.
        let mut state = 0x1234_5678_u32;
        let incompressible: Vec<u8> = (0..8192)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                (state & 0xFF) as u8
            })
            .collect();
        let zip = build(&[("a.bin", &incompressible)], true);
        // method is at offset 8 of the local header
        let method = u16::from_le_bytes([
            *zip.get(8).expect("in range"),
            *zip.get(9).expect("in range"),
        ]);
        assert_eq!(method, STORE, "incompressible data should be stored");
    }

    #[test]
    fn compressible_data_is_deflated_and_gets_smaller() {
        let body = vec![b'a'; 100_000];
        let zip = build(&[("a.txt", &body)], true);
        let method = u16::from_le_bytes([
            *zip.get(8).expect("in range"),
            *zip.get(9).expect("in range"),
        ]);
        assert_eq!(method, DEFLATE);
        assert!(
            zip.len() < body.len() / 10,
            "100k of one byte should compress hard, got {}",
            zip.len()
        );
    }

    #[test]
    fn the_end_record_counts_the_entries() {
        let zip = build(&[("a", b"1"), ("b", b"2"), ("c", b"3")], false);
        let end = END_SIGNATURE.to_le_bytes();
        let at = zip.windows(4).position(|w| w == end).expect("end record");
        let count = u16::from_le_bytes([
            *zip.get(at + 8).expect("in range"),
            *zip.get(at + 9).expect("in range"),
        ]);
        assert_eq!(count, 3);
    }

    #[test]
    fn an_empty_archive_is_still_a_valid_one() {
        // 22 bytes: just the end-of-central-directory record.
        let zip = build(&[], false);
        assert_eq!(zip.len(), 22);
        assert_eq!(zip.get(..4), Some(&END_SIGNATURE.to_le_bytes()[..]));
    }

    #[test]
    fn the_crc_is_of_the_uncompressed_bytes() {
        // A reader checks the CRC after inflating, so it must be over the
        // original — computing it over the deflated bytes produces an archive
        // that fails verification everywhere.
        let body = vec![b'z'; 5000];
        let zip = build(&[("a", &body)], true);
        let crc = u32::from_le_bytes([
            *zip.get(14).expect("in range"),
            *zip.get(15).expect("in range"),
            *zip.get(16).expect("in range"),
            *zip.get(17).expect("in range"),
        ]);
        assert_eq!(crc, crc32fast::hash(&body));
    }
}
