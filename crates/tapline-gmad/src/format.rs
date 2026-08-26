//! The GMAD container.
//!
//! Read off a real addon — PAC3, Workshop item 104691717 — on 2026-08-26 rather
//! than from a description of the format:
//!
//! ```text
//! 0000  47 4d 41 44  "GMAD"
//! 0004  03           version
//! 0005  00 x8        steamid, u64
//! 000d  3d ec 8c 6a 00 00 00 00   timestamp, u64
//! 0015  00           required content: the empty string that ends the list
//! 0016  "PAC3\0"                  addon name
//! 001b  "No description provided\0"
//! 0033  "No author provided\0"
//! 0046  01 00 00 00               addon version, i32
//! 004a  01 00 00 00               file index 1
//! 004e  "lua/pac3/extra/client/wire_expression_extension.lua\0"
//! 0082  68 08 00 00 00 00 00 00   size, i64 = 2152
//! 008a  00 00 00 00               crc32
//! 008e  02 00 00 00               file index 2 ...
//! ```
//!
//! After the index terminator (`0`), the file contents follow back to back in
//! index order, and the archive may end with a CRC of everything before it.
//!
//! The sizes are `i64` and the count is unbounded, both attacker-chosen, so
//! every one of them is checked against what is actually there rather than
//! trusted. A Workshop item is published by anyone.

use tapline_ext::ExtensionError;

/// The magic every addon starts with.
pub const MAGIC: &[u8; 4] = b"GMAD";

/// Versions this reader understands.
///
/// 1 has no required-content list; 2 and 3 do. Anything higher is refused by
/// name rather than parsed hopefully — a format that changed is one whose
/// offsets we do not know.
const MAX_VERSION: u8 = 3;

/// One file inside an addon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Path inside the addon, as the archive spells it.
    pub path: String,
    /// Size in bytes.
    pub size: u64,
    /// The CRC-32 the archive claims. Often zero, which means "not computed".
    pub crc: u32,
    /// Where the contents start in the archive.
    pub offset: usize,
}

/// An addon's metadata and index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addon {
    /// Format version.
    pub version: u8,
    /// The publishing Steam ID, often zero.
    pub steam_id: u64,
    /// Build timestamp.
    pub timestamp: u64,
    /// The addon's name.
    pub name: String,
    /// Its description. Newer addons put JSON here.
    pub description: String,
    /// Its author. Usually "No author provided".
    pub author: String,
    /// The addon's own version number.
    pub addon_version: i32,
    /// Every file, in index order.
    pub entries: Vec<Entry>,
}

/// Reads bytes without ever indexing past the end.
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

    /// A NUL-terminated string.
    ///
    /// Decoded lossily on purpose: an addon filename that is not valid UTF-8 is
    /// a file that still has to be named something, and refusing the whole
    /// archive over one byte would be worse than writing it with a replacement
    /// character. The path validator sees the result either way.
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

/// Parses an addon's header and index, and checks the contents are all there.
///
/// Does not copy any file contents; [`Entry::offset`] points into `bytes`.
pub fn parse(bytes: &[u8]) -> Result<Addon, ExtensionError> {
    let addon = parse_index(bytes)?;

    // Every entry must lie inside the archive. This is the check that stops a
    // 40-byte file claiming to hold four gigabytes, and it is why the seeking
    // reader can allocate an entry's size without further thought.
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

/// Parses the header and index alone, without requiring the contents.
///
/// What a stream needs. The index is complete long before the files are — that
/// is the whole reason the format can be extracted as it arrives — so the
/// bounds check [`parse`] performs cannot apply here, and would reject a
/// perfectly good prefix.
///
/// Skipping it is safe for a streaming consumer precisely because it never
/// allocates an entry's size: it writes bytes as they arrive, and a size that
/// lied simply means the stream ends early, which is an error the consumer
/// raises when it runs out.
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

    // Version 1 has no required-content list. Later versions carry one,
    // terminated by an empty string.
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

    // Read the index first, then work out where the contents begin: the offsets
    // are not stored, they are implied by the order and the sizes.
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

    // Place them. The offsets are not stored anywhere: they are implied by the
    // index order and the sizes, which is also what makes the format
    // streamable.
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
    /// One entry's contents.
    #[must_use]
    pub fn contents<'a>(&self, entry: &Entry, bytes: &'a [u8]) -> Option<&'a [u8]> {
        let end = entry.offset.checked_add(entry.size as usize)?;
        bytes.get(entry.offset..end)
    }

    /// Whether an entry's contents match the CRC the archive claims.
    ///
    /// A zero CRC means the addon did not compute one, which is common, and is
    /// reported as `None` rather than as a failure.
    #[must_use]
    pub fn verify(&self, entry: &Entry, bytes: &[u8]) -> Option<bool> {
        if entry.crc == 0 {
            return None;
        }
        let contents = self.contents(entry, bytes)?;
        Some(crc32fast::hash(contents) == entry.crc)
    }

    /// The addon's total unpacked size.
    #[must_use]
    pub fn unpacked_size(&self) -> u64 {
        self.entries.iter().map(|entry| entry.size).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a valid archive, so the tests below can damage it one field at a
    /// time and see which check catches it.
    fn build(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.push(3);
        out.extend_from_slice(&0_u64.to_le_bytes());
        out.extend_from_slice(&1_u64.to_le_bytes());
        out.push(0); // no required content
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
        // The check that matters: this is how a 40-byte archive claims to hold
        // four gigabytes, and how a reader that trusted it would allocate them.
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
            // Every prefix either parses (the index can be complete before the
            // contents are) or errors. Neither may panic.
            let _ = parse(raw.get(..cut).expect("in range"));
        }
    }

    #[test]
    fn an_unterminated_string_is_an_error() {
        let mut raw = build(&[("a", b"b")]);
        // Remove every NUL after the header so the name never terminates.
        for byte in raw.iter_mut().skip(21) {
            if *byte == 0 {
                *byte = b'x';
            }
        }
        assert!(parse(&raw).is_err());
    }

    #[test]
    fn a_zero_crc_is_not_a_failed_check() {
        // Most real addons leave it zero. Reporting that as corruption would
        // reject almost everything on the Workshop.
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
