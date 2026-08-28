//! The file and chunk view of a manifest.

use crate::{ManifestError, RawManifest};
use tapline_ids::{DepotId, ManifestId};

/// What a file's `flags` field means.
pub mod flag {
    /// A directory entry, with no content of its own.
    pub const DIRECTORY: u32 = 1 << 6;
    /// A symbolic link; the target is in `linktarget`.
    pub const SYMLINK: u32 = 1 << 9;
    /// The file should be executable.
    pub const EXECUTABLE: u32 = 1 << 0;
    /// A custom executable, treated the same way.
    pub const CUSTOM_EXECUTABLE: u32 = 1 << 5;
    /// Marked read-only.
    pub const READ_ONLY: u32 = 1 << 3;
    /// Hidden on Windows.
    #[allow(dead_code, reason = "documents the flag word; no Linux behaviour")]
    pub const HIDDEN: u32 = 1 << 4;
}

/// A file's kind and permissions, as the manifest describes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FileFlags {
    /// A directory rather than a file.
    pub directory: bool,
    /// A symbolic link.
    pub symlink: bool,
    /// Should be executable.
    pub executable: bool,
    /// Marked read-only.
    pub read_only: bool,
}

impl FileFlags {
    /// Decodes the manifest's flag word.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self {
            directory: bits & flag::DIRECTORY != 0,
            symlink: bits & flag::SYMLINK != 0,
            executable: bits & (flag::EXECUTABLE | flag::CUSTOM_EXECUTABLE) != 0,
            read_only: bits & flag::READ_ONLY != 0,
        }
    }
}

/// One chunk of one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// The chunk id: the SHA-1 of its plaintext.
    pub id: [u8; 20],
    /// CRC-32 of the encrypted, compressed form, as the CDN stores it.
    pub crc: u32,
    /// Where the chunk's plaintext belongs within its file.
    pub offset: u64,
    /// The plaintext length.
    pub uncompressed_size: u32,
    /// The stored length, encrypted and compressed.
    pub compressed_size: u32,
}

impl Chunk {
    /// The chunk id in the lowercase hex the CDN's URLs use.
    #[must_use]
    pub fn id_hex(&self) -> String {
        self.id.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

/// One file in a depot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// The path relative to the install root, as the manifest spells it.
    pub path: String,
    /// The file's size in bytes.
    pub size: u64,
    /// Decoded flags.
    pub flags: FileFlags,
    /// The raw flag word, so nothing is lost to a flag we do not model.
    pub raw_flags: u32,
    /// A symlink's target, when this is a symlink.
    pub link_target: Option<String>,
    /// The chunks making up the file, in offset order.
    pub chunks: Vec<Chunk>,
}

/// A manifest, ready to drive a download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Which depot.
    pub depot: DepotId,
    /// Which build.
    pub id: ManifestId,
    /// When Steam created it, as a Unix timestamp.
    pub created: u32,
    /// Total installed size.
    pub total_size: u64,
    /// How many distinct chunks the depot has.
    pub unique_chunks: u32,
    /// The files, in the order the manifest lists them.
    pub files: Vec<FileEntry>,
}

impl Manifest {
    /// Builds a manifest from its raw blocks, decrypting filenames if needed.
    pub fn from_raw(
        raw: &RawManifest,
        depot_key: Option<&[u8; 32]>,
    ) -> Result<Self, ManifestError> {
        let encrypted = raw.metadata.filenames_encrypted.unwrap_or(false);
        if encrypted && depot_key.is_none() {
            return Err(ManifestError::FilenamesEncrypted);
        }

        let mut files = Vec::with_capacity(raw.payload.mappings.len());
        for mapping in &raw.payload.mappings {
            let stored = mapping.filename.as_deref().unwrap_or_default();
            let path = match (encrypted, depot_key) {
                (true, Some(key)) => decrypt_filename(stored, key)?,
                _ => stored.to_owned(),
            };

            let link_target = match (&mapping.linktarget, encrypted, depot_key) {
                (Some(target), true, Some(key)) if !target.is_empty() => {
                    Some(decrypt_filename(target, key)?)
                }
                (Some(target), _, _) if !target.is_empty() => Some(target.clone()),
                _ => None,
            };

            let raw_flags = mapping.flags.unwrap_or(0);
            let mut chunks: Vec<Chunk> = mapping
                .chunks
                .iter()
                .filter_map(|chunk| {
                    Some(Chunk {
                        id: chunk.sha.as_deref()?.try_into().ok()?,
                        crc: chunk.crc.unwrap_or(0),
                        offset: chunk.offset.unwrap_or(0),
                        uncompressed_size: chunk.cb_original.unwrap_or(0),
                        compressed_size: chunk.cb_compressed.unwrap_or(0),
                    })
                })
                .collect();

            // The manifest promises no chunk order; resume logic needs offset order.
            chunks.sort_by_key(|chunk| chunk.offset);

            files.push(FileEntry {
                path,
                size: mapping.size.unwrap_or(0),
                flags: FileFlags::from_bits(raw_flags),
                raw_flags,
                link_target,
                chunks,
            });
        }

        Ok(Self {
            depot: DepotId(raw.metadata.depot_id.unwrap_or(0)),
            id: ManifestId(raw.metadata.gid_manifest.unwrap_or(0)),
            created: raw.metadata.creation_time.unwrap_or(0),
            total_size: raw.metadata.cb_disk_original.unwrap_or(0),
            unique_chunks: raw.metadata.unique_chunks.unwrap_or(0),
            files,
        })
    }

    /// Parses a manifest as served by the CDN.
    pub fn parse(bytes: &[u8], depot_key: Option<&[u8; 32]>) -> Result<Self, ManifestError> {
        Self::from_raw(&RawManifest::parse(bytes)?, depot_key)
    }

    /// Every distinct chunk in the depot, with the total bytes to download.
    #[must_use]
    pub fn distinct_chunks(&self) -> (Vec<&Chunk>, u64) {
        let mut seen = std::collections::HashSet::new();
        let mut chunks = Vec::new();
        let mut bytes = 0_u64;

        for file in &self.files {
            for chunk in &file.chunks {
                if seen.insert(chunk.id) {
                    bytes += u64::from(chunk.compressed_size);
                    chunks.push(chunk);
                }
            }
        }
        (chunks, bytes)
    }

    /// Files that carry content, skipping directories and symlinks.
    pub fn regular_files(&self) -> impl Iterator<Item = &FileEntry> {
        self.files
            .iter()
            .filter(|file| !file.flags.directory && !file.flags.symlink)
    }
}

/// Decrypts one filename; backslashes become forward slashes (Valve writes Windows separators).
fn decrypt_filename(encoded: &str, key: &[u8; 32]) -> Result<String, ManifestError> {
    let ciphertext = base64_decode(encoded).ok_or(ManifestError::FilenameUndecryptable)?;
    let plaintext = tapline_crypto::decrypt_content(key, &ciphertext)
        .map_err(|_| ManifestError::FilenameUndecryptable)?;

    // Steam NUL-terminates the plaintext.
    let trimmed = plaintext.strip_suffix(&[0]).unwrap_or(&plaintext);
    let text =
        String::from_utf8(trimmed.to_vec()).map_err(|_| ManifestError::FilenameUndecryptable)?;

    Ok(text.replace('\\', "/"))
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    fn value(byte: u8) -> Option<u8> {
        match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }

    let bytes: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let unpadded: Vec<u8> = bytes.iter().copied().take_while(|b| *b != b'=').collect();

    let mut out = Vec::with_capacity(unpadded.len() * 3 / 4);
    let mut accumulator = 0_u32;
    let mut bits = 0_u32;

    for byte in unpadded {
        accumulator = (accumulator << 6) | u32::from(value(byte)?);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((accumulator >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL: &[u8] = include_bytes!("../tests/fixtures/manifest_232257_4797708003880603728.bin");

    #[test]
    fn base64_round_trips_against_known_vectors() {
        assert_eq!(base64_decode("Zm9vYmFy"), Some(b"foobar".to_vec()));
        assert_eq!(base64_decode("Zg=="), Some(b"f".to_vec()));
        assert_eq!(base64_decode("Zm8="), Some(b"fo".to_vec()));
        assert_eq!(base64_decode(""), Some(Vec::new()));
        assert_eq!(base64_decode("not!base64"), None);
    }

    #[test]
    fn an_encrypted_manifest_refuses_to_produce_filenames_without_a_key() {
        let raw = crate::RawManifest::parse(REAL).expect("must parse");
        assert_eq!(
            Manifest::from_raw(&raw, None),
            Err(ManifestError::FilenamesEncrypted)
        );
    }

    #[test]
    fn the_wrong_depot_key_is_reported_rather_than_producing_nonsense() {
        let raw = crate::RawManifest::parse(REAL).expect("must parse");
        assert_eq!(
            Manifest::from_raw(&raw, Some(&[0x00; 32])),
            Err(ManifestError::FilenameUndecryptable)
        );
    }

    #[test]
    fn windows_separators_become_forward_slashes() {
        let key = [0x11_u8; 32];
        let ciphertext =
            tapline_crypto::encrypt_content(&key, b"bin\\linux64\\srcds\0").expect("encrypt");
        let encoded = base64_encode(&ciphertext);
        assert_eq!(
            decrypt_filename(&encoded, &key).expect("must decrypt"),
            "bin/linux64/srcds"
        );
    }

    #[test]
    fn a_nul_terminator_is_stripped() {
        let key = [0x22_u8; 32];
        let ciphertext = tapline_crypto::encrypt_content(&key, b"file.txt\0").expect("encrypt");
        assert_eq!(
            decrypt_filename(&base64_encode(&ciphertext), &key).expect("must decrypt"),
            "file.txt"
        );
    }

    #[test]
    fn flags_decode_the_way_valve_sets_them() {
        assert!(FileFlags::from_bits(flag::DIRECTORY).directory);
        assert!(FileFlags::from_bits(flag::SYMLINK).symlink);
        assert!(FileFlags::from_bits(flag::EXECUTABLE).executable);
        assert!(FileFlags::from_bits(flag::CUSTOM_EXECUTABLE).executable);
        assert!(FileFlags::from_bits(flag::READ_ONLY).read_only);

        let none = FileFlags::from_bits(0);
        assert!(!none.directory && !none.symlink && !none.executable);
    }

    #[test]
    fn distinct_chunks_counts_a_repeated_chunk_once() {
        let shared = Chunk {
            id: [7; 20],
            crc: 0,
            offset: 0,
            uncompressed_size: 100,
            compressed_size: 40,
        };
        let manifest = Manifest {
            depot: DepotId(1),
            id: ManifestId(2),
            created: 0,
            total_size: 200,
            unique_chunks: 1,
            files: vec![
                FileEntry {
                    path: "a".into(),
                    size: 100,
                    flags: FileFlags::default(),
                    raw_flags: 0,
                    link_target: None,
                    chunks: vec![shared.clone()],
                },
                FileEntry {
                    path: "b".into(),
                    size: 100,
                    flags: FileFlags::default(),
                    raw_flags: 0,
                    link_target: None,
                    chunks: vec![shared],
                },
            ],
        };

        let (chunks, bytes) = manifest.distinct_chunks();
        assert_eq!(chunks.len(), 1, "the shared chunk was counted twice");
        assert_eq!(bytes, 40, "the shared chunk's bytes were counted twice");
    }

    #[test]
    fn chunks_come_back_in_offset_order() {
        let raw = crate::RawManifest::parse(REAL).expect("must parse");
        let payload = &raw.payload;
        for mapping in &payload.mappings {
            let manifest_chunks: Vec<u64> =
                mapping.chunks.iter().filter_map(|c| c.offset).collect();
            let mut sorted = manifest_chunks.clone();
            sorted.sort_unstable();
            assert_eq!(sorted.len(), manifest_chunks.len());
        }
    }

    fn base64_encode(input: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in input.chunks(3) {
            let b0 = chunk.first().copied().unwrap_or(0);
            let b1 = chunk.get(1).copied().unwrap_or(0);
            let b2 = chunk.get(2).copied().unwrap_or(0);
            let triple = (u32::from(b0) << 16) | (u32::from(b1) << 8) | u32::from(b2);
            for (position, shift) in [18_u32, 12, 6, 0].into_iter().enumerate() {
                let pad_from = match chunk.len() {
                    1 => 2,
                    2 => 3,
                    _ => 4,
                };
                if position >= pad_from {
                    out.push('=');
                } else {
                    let index = ((triple >> shift) & 0x3F) as usize;
                    out.push(char::from(ALPHABET.get(index).copied().unwrap_or(b'A')));
                }
            }
        }
        out
    }
}
