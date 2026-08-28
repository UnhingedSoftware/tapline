use tapline_ext::ExtensionError;

pub const MAGIC: &[u8; 4] = b"GMAD";

const MAX_VERSION: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub size: u64,
    pub crc: u32,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addon {
    pub version: u8,
    pub steam_id: u64,
    pub timestamp: u64,
    pub name: String,
    pub description: String,
    pub author: String,
    pub addon_version: i32,
    pub entries: Vec<Entry>,
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn short(what: &str) -> ExtensionError {
        ExtensionError::Malformed {
            extension: "gmad",
            reason: format!("the archive ends in the middle of its {what}"),
        }
    }

    fn take(&mut self, len: usize, what: &str) -> Result<&'a [u8], ExtensionError> {
        let end = self.at.checked_add(len).ok_or_else(|| Self::short(what))?;
        let slice = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| Self::short(what))?;
        self.at = end;
        Ok(slice)
    }

    fn u8(&mut self, what: &str) -> Result<u8, ExtensionError> {
        Ok(*self
            .take(1, what)?
            .first()
            .ok_or_else(|| Self::short(what))?)
    }

    fn u32(&mut self, what: &str) -> Result<u32, ExtensionError> {
        let bytes: [u8; 4] = self
            .take(4, what)?
            .try_into()
            .map_err(|_| Self::short(what))?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn i64(&mut self, what: &str) -> Result<i64, ExtensionError> {
        let bytes: [u8; 8] = self
            .take(8, what)?
            .try_into()
            .map_err(|_| Self::short(what))?;
        Ok(i64::from_le_bytes(bytes))
    }

    fn u64(&mut self, what: &str) -> Result<u64, ExtensionError> {
        Ok(self.i64(what)? as u64)
    }

    fn string(&mut self, what: &str) -> Result<String, ExtensionError> {
        let rest = self.bytes.get(self.at..).ok_or_else(|| Self::short(what))?;
        let end =
            rest.iter()
                .position(|byte| *byte == 0)
                .ok_or_else(|| ExtensionError::Malformed {
                    extension: "gmad",
                    reason: format!("the {what} is not terminated"),
                })?;
        let text = String::from_utf8_lossy(rest.get(..end).unwrap_or_default()).into_owned();
        self.at = self.at.saturating_add(end).saturating_add(1);
        Ok(text)
    }
}

pub fn parse(bytes: &[u8]) -> Result<Addon, ExtensionError> {
    let addon = parse_index(bytes)?;

    let total = bytes.len() as u64;
    for entry in &addon.entries {
        let end = (entry.offset as u64)
            .checked_add(entry.size)
            .ok_or_else(|| ExtensionError::Malformed {
                extension: "gmad",
                reason: format!("{:?} has a size that overflows the archive", entry.path),
            })?;
        if end > total {
            return Err(ExtensionError::Malformed {
                extension: "gmad",
                reason: format!(
                    "{:?} claims {} bytes at offset {}, past the archive's {total}",
                    entry.path, entry.size, entry.offset
                ),
            });
        }
    }
    Ok(addon)
}

pub fn parse_index(bytes: &[u8]) -> Result<Addon, ExtensionError> {
    let mut reader = Reader::new(bytes);

    let magic = reader.take(4, "magic")?;
    if magic != MAGIC {
        return Err(ExtensionError::Malformed {
            extension: "gmad",
            reason: format!(
                "expected magic GMAD, found {:?}",
                String::from_utf8_lossy(magic)
            ),
        });
    }

    let version = reader.u8("version")?;
    if version == 0 || version > MAX_VERSION {
        return Err(ExtensionError::Malformed {
            extension: "gmad",
            reason: format!(
                "unsupported GMAD version {version}; this reader knows 1 to {MAX_VERSION}"
            ),
        });
    }

    let steam_id = reader.u64("steam id")?;
    let timestamp = reader.u64("timestamp")?;

    if version > 1 {
        loop {
            let required = reader.string("required content list")?;
            if required.is_empty() {
                break;
            }
        }
    }

    let name = reader.string("addon name")?;
    let description = reader.string("addon description")?;
    let author = reader.string("addon author")?;
    let addon_version = reader.u32("addon version")? as i32;

    let mut index = Vec::new();
    loop {
        let number = reader.u32("file index")?;
        if number == 0 {
            break;
        }
        let path = reader.string("file name")?;
        let size = reader.i64("file size")?;
        let crc = reader.u32("file checksum")?;

        if size < 0 {
            return Err(ExtensionError::Malformed {
                extension: "gmad",
                reason: format!("{path:?} claims a negative size of {size}"),
            });
        }
        index.push((path, size as u64, crc));
    }

    let mut offset = reader.at;
    let mut entries = Vec::with_capacity(index.len());
    for (path, size, crc) in index {
        let end = (offset as u64)
            .checked_add(size)
            .ok_or_else(|| ExtensionError::Malformed {
                extension: "gmad",
                reason: format!("{path:?} has a size that overflows the archive"),
            })?;
        entries.push(Entry {
            path,
            size,
            crc,
            offset,
        });
        offset = usize::try_from(end).map_err(|_| ExtensionError::Malformed {
            extension: "gmad",
            reason: format!(
                "{:?} places the next entry past addressable memory",
                entries.last().map(|e| &e.path)
            ),
        })?;
    }

    Ok(Addon {
        version,
        steam_id,
        timestamp,
        name,
        description,
        author,
        addon_version,
        entries,
    })
}

impl Addon {
    #[must_use]
    pub fn contents<'a>(&self, entry: &Entry, bytes: &'a [u8]) -> Option<&'a [u8]> {
        let end = entry.offset.checked_add(entry.size as usize)?;
        bytes.get(entry.offset..end)
    }

    #[must_use]
    pub fn verify(&self, entry: &Entry, bytes: &[u8]) -> Option<bool> {
        if entry.crc == 0 {
            return None;
        }
        let contents = self.contents(entry, bytes)?;
        Some(crc32fast::hash(contents) == entry.crc)
    }

    #[must_use]
    pub fn unpacked_size(&self) -> u64 {
        self.entries.iter().map(|entry| entry.size).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.push(3);
        out.extend_from_slice(&0_u64.to_le_bytes());
        out.extend_from_slice(&1_u64.to_le_bytes());
        out.push(0);
        out.extend_from_slice(b"Test Addon\0");
        out.extend_from_slice(b"A description\0");
        out.extend_from_slice(b"An author\0");
        out.extend_from_slice(&1_i32.to_le_bytes());
        for (index, (path, body)) in files.iter().enumerate() {
            out.extend_from_slice(&(index as u32 + 1).to_le_bytes());
            out.extend_from_slice(path.as_bytes());
            out.push(0);
            out.extend_from_slice(&(body.len() as i64).to_le_bytes());
            out.extend_from_slice(&crc32fast::hash(body).to_le_bytes());
        }
        out.extend_from_slice(&0_u32.to_le_bytes());
        for (_, body) in files {
            out.extend_from_slice(body);
        }
        out
    }

    #[test]
    fn a_well_formed_addon_round_trips() {
        let raw = build(&[
            ("lua/autorun/init.lua", b"print('hi')"),
            ("materials/x.vmt", b"vmt"),
        ]);
        let addon = parse(&raw).expect("must parse");

        assert_eq!(addon.version, 3);
        assert_eq!(addon.name, "Test Addon");
        assert_eq!(addon.author, "An author");
        assert_eq!(addon.entries.len(), 2);
        assert_eq!(addon.entries[0].path, "lua/autorun/init.lua");
        assert_eq!(addon.unpacked_size(), 14);
        assert_eq!(
            addon.contents(&addon.entries[0], &raw),
            Some(&b"print('hi')"[..])
        );
        assert_eq!(addon.contents(&addon.entries[1], &raw), Some(&b"vmt"[..]));
        assert_eq!(addon.verify(&addon.entries[0], &raw), Some(true));
    }

    #[test]
    fn the_wrong_magic_says_what_it_found() {
        let error = parse(b"PK\x03\x04and then some").expect_err("must refuse");
        let text = error.to_string();
        assert!(text.contains("expected magic GMAD"), "{text}");
        assert!(text.contains("PK"), "{text}");
    }

    #[test]
    fn a_future_version_is_refused_by_number() {
        let mut raw = build(&[("a", b"b")]);
        raw[4] = 9;
        let error = parse(&raw).expect_err("must refuse");
        assert!(error.to_string().contains("version 9"), "{error}");
    }

    #[test]
    fn a_size_past_the_end_of_the_archive_is_caught() {
        let mut raw = build(&[("a", b"body")]);
        let marker = (4_i64).to_le_bytes();
        let at = raw
            .windows(8)
            .position(|window| window == marker)
            .expect("the size field is in there");
        raw.splice(at..at + 8, (1_i64 << 40).to_le_bytes());

        let error = parse(&raw).expect_err("must refuse");
        assert!(error.to_string().contains("past the archive"), "{error}");
    }

    #[test]
    fn a_negative_size_is_refused_rather_than_wrapped() {
        let mut raw = build(&[("a", b"body")]);
        let marker = (4_i64).to_le_bytes();
        let at = raw
            .windows(8)
            .position(|window| window == marker)
            .expect("size field");
        raw.splice(at..at + 8, (-1_i64).to_le_bytes());

        let error = parse(&raw).expect_err("must refuse");
        assert!(error.to_string().contains("negative size"), "{error}");
    }

    #[test]
    fn truncation_anywhere_is_an_error_and_never_a_panic() {
        let raw = build(&[("lua/a.lua", b"contents here"), ("b.txt", b"more")]);
        for cut in 0..raw.len() {
            let _ = parse(raw.get(..cut).expect("in range"));
        }
    }

    #[test]
    fn an_unterminated_string_is_an_error() {
        let mut raw = build(&[("a", b"b")]);
        for byte in raw.iter_mut().skip(21) {
            if *byte == 0 {
                *byte = b'x';
            }
        }
        assert!(parse(&raw).is_err());
    }

    #[test]
    fn a_zero_crc_is_not_a_failed_check() {
        let mut raw = build(&[("a", b"body")]);
        let crc = crc32fast::hash(b"body").to_le_bytes();
        let at = raw
            .windows(4)
            .position(|window| window == crc)
            .expect("crc field");
        raw.splice(at..at + 4, 0_u32.to_le_bytes());

        let addon = parse(&raw).expect("must parse");
        assert_eq!(addon.verify(&addon.entries[0], &raw), None);
    }

    #[test]
    fn a_wrong_crc_is_reported() {
        let mut raw = build(&[("a", b"body")]);
        let crc = crc32fast::hash(b"body").to_le_bytes();
        let at = raw
            .windows(4)
            .position(|window| window == crc)
            .expect("crc field");
        raw.splice(at..at + 4, 0xDEAD_BEEF_u32.to_le_bytes());

        let addon = parse(&raw).expect("must parse");
        assert_eq!(addon.verify(&addon.entries[0], &raw), Some(false));
    }

    #[test]
    fn an_empty_addon_is_valid() {
        let addon = parse(&build(&[])).expect("must parse");
        assert!(addon.entries.is_empty());
        assert_eq!(addon.unpacked_size(), 0);
    }
}
