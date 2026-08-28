use std::io::Write;
use tapline_ext::ExtensionError;

const LOCAL_SIGNATURE: u32 = 0x0403_4b50;
const CENTRAL_SIGNATURE: u32 = 0x0201_4b50;
const END_SIGNATURE: u32 = 0x0605_4b50;

const STORE: u16 = 0;
const DEFLATE: u16 = 8;

const VERSION: u16 = 20;

const MAX_SIZE: u64 = u32::MAX as u64;
const MAX_ENTRIES: usize = u16::MAX as usize;

const LEVEL: u8 = 6;

pub struct Prepared {
    pub name: String,
    pub crc: u32,
    pub uncompressed: usize,
    pub payload: Vec<u8>,
    pub method: u16,
}

#[must_use]
pub fn prepare(name: String, body: &[u8], compress: bool) -> Prepared {
    let crc = crc32fast::hash(body);
    let deflated = compress
        .then(|| miniz_oxide::deflate::compress_to_vec(body, LEVEL))
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

struct Record {
    name: String,
    crc: u32,
    compressed: u32,
    uncompressed: u32,
    method: u16,
    offset: u32,
}

pub struct Writer<W: Write> {
    out: W,
    records: Vec<Record>,
    offset: u64,
}

impl<W: Write> Writer<W> {
    pub const fn new(out: W) -> Self {
        Self {
            out,
            records: Vec::new(),
            offset: 0,
        }
    }

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

        self.u32(LOCAL_SIGNATURE)?;
        self.u16(VERSION)?;
        self.u16(0)?;
        self.u16(method)?;
        self.u16(0)?;
        self.u16(0)?;
        self.u32(crc)?;
        self.u32(compressed)?;
        self.u32(uncompressed)?;
        self.u16(name_len)?;
        self.u16(0)?;
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

    pub fn finish(mut self) -> Result<W, ExtensionError> {
        let directory_start = self.offset;

        let records = std::mem::take(&mut self.records);
        for record in &records {
            self.write_central_header(record)?;
        }

        let directory_size = self.offset.saturating_sub(directory_start);
        let count = u16::try_from(records.len()).unwrap_or(u16::MAX);

        self.u32(END_SIGNATURE)?;
        self.u16(0)?;
        self.u16(0)?;
        self.u16(count)?;
        self.u16(count)?;
        self.u32(u32::try_from(directory_size).unwrap_or(u32::MAX))?;
        self.u32(u32::try_from(directory_start).unwrap_or(u32::MAX))?;
        self.u16(0)?;

        self.out.flush()?;
        Ok(self.out)
    }

    fn write_central_header(&mut self, record: &Record) -> Result<(), ExtensionError> {
        let name_len = u16::try_from(record.name.len()).unwrap_or(u16::MAX);

        self.u32(CENTRAL_SIGNATURE)?;
        self.u16(VERSION)?;
        self.u16(VERSION)?;
        self.u16(0)?;
        self.u16(record.method)?;
        self.u16(0)?;
        self.u16(0)?;
        self.u32(record.crc)?;
        self.u32(record.compressed)?;
        self.u32(record.uncompressed)?;
        self.u16(name_len)?;
        self.u16(0)?;
        self.u16(0)?;
        self.u16(0)?;
        self.u16(0)?;
        self.u32(0)?;
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
        let zip = build(&[], false);
        assert_eq!(zip.len(), 22);
        assert_eq!(zip.get(..4), Some(&END_SIGNATURE.to_le_bytes()[..]));
    }

    #[test]
    fn the_crc_is_of_the_uncompressed_bytes() {
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
