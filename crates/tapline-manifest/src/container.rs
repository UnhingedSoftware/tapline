//! Unwrapping the manifest container.
//!
//! Two layers: a ZIP holding one deflated entry, and inside it a sequence of
//! magic-delimited protobuf blocks. Both layouts were read off a real manifest
//! rather than taken from a description — see the crate docs for the dump.

use std::fmt;
use tapline_proto::content_manifest::{
    ContentManifestMetadata, ContentManifestPayload, ContentManifestSignature,
};
use tapline_wire::{Message, WireError};

/// Block magics, in the order they appear.
const MAGIC_PAYLOAD: u32 = 0x71F6_17D0;
const MAGIC_METADATA: u32 = 0x1F48_12BE;
const MAGIC_SIGNATURE: u32 = 0x1B81_B817;
const MAGIC_END: u32 = 0x32C4_15AB;

/// The ZIP local file header magic.
const ZIP_LOCAL_HEADER: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

/// The largest manifest we will decompress.
///
/// A depot with a million files produces a manifest of tens of megabytes. This
/// is well above that and well below anything that would trouble a small node.
pub const MAX_MANIFEST: usize = 512 * 1024 * 1024;

/// What went wrong reading a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The bytes are not a manifest container.
    NotAManifest,
    /// The ZIP entry used a compression method other than store or deflate.
    UnsupportedCompression(u16),
    /// Decompression failed.
    Decompress(String),
    /// The ZIP entry's CRC-32 did not match its contents.
    ChecksumMismatch,
    /// A length or offset ran past the end of the data.
    Truncated,
    /// A block magic we do not know.
    ///
    /// Reported rather than skipped: a block we cannot identify may be the one
    /// that says what the depot contains.
    UnknownBlock(u32),
    /// A required block was absent.
    MissingBlock(&'static str),
    /// A protobuf block did not decode.
    Wire(WireError),
    /// The manifest is for a different depot or build than the one requested.
    ///
    /// A CDN or a cache serving the wrong manifest would otherwise produce a
    /// confidently wrong install.
    WrongManifest {
        /// What was asked for.
        expected: u64,
        /// What arrived.
        actual: u64,
    },
    /// Filenames are encrypted and no key was supplied.
    FilenamesEncrypted,
    /// A filename failed to decrypt, or was not valid base64.
    ///
    /// Almost always the wrong depot key.
    FilenameUndecryptable,
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAManifest => f.write_str("not a manifest container"),
            Self::UnsupportedCompression(m) => write!(f, "unsupported ZIP compression {m}"),
            Self::Decompress(e) => write!(f, "decompression failed: {e}"),
            Self::ChecksumMismatch => f.write_str("the manifest failed its CRC-32 check"),
            Self::Truncated => f.write_str("the manifest ended mid-structure"),
            Self::UnknownBlock(magic) => write!(f, "unknown manifest block {magic:#010x}"),
            Self::MissingBlock(name) => write!(f, "the manifest has no {name} block"),
            Self::Wire(e) => write!(f, "malformed manifest block: {e}"),
            Self::WrongManifest { expected, actual } => {
                write!(f, "asked for manifest {expected}, received {actual}")
            }
            Self::FilenamesEncrypted => {
                f.write_str("the manifest's filenames are encrypted and no depot key was given")
            }
            Self::FilenameUndecryptable => {
                f.write_str("a filename did not decrypt — usually the wrong depot key")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

impl From<WireError> for ManifestError {
    fn from(error: WireError) -> Self {
        Self::Wire(error)
    }
}

/// A manifest's three protobuf blocks, decoded but not yet interpreted.
#[derive(Debug, Clone, PartialEq)]
pub struct RawManifest {
    /// The files and their chunks.
    pub payload: ContentManifestPayload,
    /// Depot id, build sizes, and whether filenames are encrypted.
    pub metadata: ContentManifestMetadata,
    /// Valve's signature over the manifest, when present.
    ///
    /// Not verified: the public key is not published, so a check here could only
    /// ever be theatre. Integrity comes from the manifest id being named over an
    /// authenticated session and every chunk being content-addressed.
    pub signature: Option<ContentManifestSignature>,
}

impl RawManifest {
    /// Parses a manifest as served by the CDN.
    pub fn parse(bytes: &[u8]) -> Result<Self, ManifestError> {
        let inner = unzip(bytes)?;
        Self::parse_blocks(&inner)
    }

    /// Parses the block sequence, after the ZIP layer has been removed.
    pub fn parse_blocks(bytes: &[u8]) -> Result<Self, ManifestError> {
        let mut payload = None;
        let mut metadata = None;
        let mut signature = None;
        let mut cursor = 0_usize;

        loop {
            let magic = read_u32(bytes, cursor).ok_or(ManifestError::Truncated)?;
            cursor += 4;

            if magic == MAGIC_END {
                break;
            }

            let length = read_u32(bytes, cursor).ok_or(ManifestError::Truncated)? as usize;
            cursor += 4;
            let end = cursor.checked_add(length).ok_or(ManifestError::Truncated)?;
            let block = bytes.get(cursor..end).ok_or(ManifestError::Truncated)?;
            cursor = end;

            match magic {
                MAGIC_PAYLOAD => payload = Some(ContentManifestPayload::decode(block)?),
                MAGIC_METADATA => metadata = Some(ContentManifestMetadata::decode(block)?),
                MAGIC_SIGNATURE => signature = Some(ContentManifestSignature::decode(block)?),
                // Skipping an unrecognised block would mean claiming to have
                // read a manifest whose contents we do not know.
                other => return Err(ManifestError::UnknownBlock(other)),
            }
        }

        Ok(Self {
            payload: payload.ok_or(ManifestError::MissingBlock("payload"))?,
            metadata: metadata.ok_or(ManifestError::MissingBlock("metadata"))?,
            signature,
        })
    }
}

/// Extracts the single entry from the manifest's ZIP wrapper.
///
/// Deliberately not a general ZIP reader. A manifest holds exactly one entry, so
/// the local file header is enough and the central directory is never consulted
/// — which also sidesteps every ZIP-parsing trick that depends on the two
/// disagreeing.
fn unzip(bytes: &[u8]) -> Result<Vec<u8>, ManifestError> {
    if bytes.get(..4) != Some(&ZIP_LOCAL_HEADER) {
        // Some paths hand us an already-unwrapped block sequence.
        if read_u32(bytes, 0) == Some(MAGIC_PAYLOAD) {
            return Ok(bytes.to_vec());
        }
        return Err(ManifestError::NotAManifest);
    }

    let method = read_u16(bytes, 8).ok_or(ManifestError::Truncated)?;
    let expected_crc = read_u32(bytes, 14).ok_or(ManifestError::Truncated)?;
    let compressed_size = read_u32(bytes, 18).ok_or(ManifestError::Truncated)? as usize;
    let uncompressed_size = read_u32(bytes, 22).ok_or(ManifestError::Truncated)? as usize;
    let name_len = read_u16(bytes, 26).ok_or(ManifestError::Truncated)? as usize;
    let extra_len = read_u16(bytes, 28).ok_or(ManifestError::Truncated)? as usize;

    if uncompressed_size > MAX_MANIFEST {
        return Err(ManifestError::Decompress(format!(
            "manifest claims {uncompressed_size} bytes"
        )));
    }

    let data_start = 30_usize
        .checked_add(name_len)
        .and_then(|v| v.checked_add(extra_len))
        .ok_or(ManifestError::Truncated)?;
    let data_end = data_start
        .checked_add(compressed_size)
        .ok_or(ManifestError::Truncated)?;
    let data = bytes
        .get(data_start..data_end)
        .ok_or(ManifestError::Truncated)?;

    let out = match method {
        0 => data.to_vec(),
        8 => miniz_oxide::inflate::decompress_to_vec_with_limit(data, MAX_MANIFEST)
            .map_err(|e| ManifestError::Decompress(format!("{:?}", e.status)))?,
        other => return Err(ManifestError::UnsupportedCompression(other)),
    };

    // The ZIP header carries a CRC-32, and checking it costs nothing next to
    // the download that produced these bytes.
    if crc32fast::hash(&out) != expected_crc {
        return Err(ManifestError::ChecksumMismatch);
    }

    Ok(out)
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    Some(u32::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let end = offset.checked_add(2)?;
    Some(u16::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real manifest for depot 232257, fetched from the CDN on 2026-08-26.
    const REAL: &[u8] = include_bytes!("../tests/fixtures/manifest_232257_4797708003880603728.bin");

    #[test]
    fn a_real_manifest_parses() {
        let manifest = RawManifest::parse(REAL).expect("the captured manifest must parse");

        assert_eq!(manifest.metadata.depot_id, Some(232_257));
        assert_eq!(
            manifest.metadata.gid_manifest,
            Some(4_797_708_003_880_603_728)
        );
        // This depot really does encrypt its filenames, which is why the
        // decryption path exists.
        assert_eq!(manifest.metadata.filenames_encrypted, Some(true));
        assert!(!manifest.payload.mappings.is_empty());
        assert!(manifest.signature.is_some());
    }

    #[test]
    fn the_files_and_chunks_come_through() {
        let manifest = RawManifest::parse(REAL).expect("must parse");
        let mapping = manifest.payload.mappings.first().expect("a file");

        // Encrypted, so the name is a base64 blob rather than a path.
        let name = mapping.filename.as_deref().expect("a filename");
        assert!(
            !name.contains('/'),
            "an encrypted name should not look like a path"
        );

        assert!(mapping.size.is_some());
        assert!(
            !mapping.chunks.is_empty(),
            "a file with no chunks is not downloadable"
        );

        let chunk = mapping.chunks.first().expect("a chunk");
        // The chunk id is a SHA-1, which is what makes content addressing work.
        assert_eq!(chunk.sha.as_deref().map(<[u8]>::len), Some(20));
        assert!(chunk.cb_original.is_some());
        assert!(chunk.cb_compressed.is_some());
    }

    #[test]
    fn the_metadata_sizes_are_plausible() {
        let manifest = RawManifest::parse(REAL).expect("must parse");
        let original = manifest.metadata.cb_disk_original.unwrap_or(0);
        let compressed = manifest.metadata.cb_disk_compressed.unwrap_or(0);

        // PICS said this depot is 9,989 bytes installed.
        assert_eq!(original, 9_989);
        assert!(compressed > 0 && compressed <= original);
        assert!(manifest.metadata.unique_chunks.unwrap_or(0) > 0);
    }

    #[test]
    fn a_corrupted_manifest_fails_its_checksum() {
        // The ZIP CRC-32 is free to check and catches a truncated or damaged
        // download before any of it is believed.
        let mut damaged = REAL.to_vec();
        let middle = damaged.len() / 2;
        if let Some(byte) = damaged.get_mut(middle) {
            *byte ^= 0xFF;
        }
        assert!(matches!(
            RawManifest::parse(&damaged),
            Err(ManifestError::ChecksumMismatch | ManifestError::Decompress(_))
        ));
    }

    #[test]
    fn truncation_before_the_data_ends_is_an_error_and_never_a_panic() {
        // The compressed entry ends before the file does: what follows is the
        // ZIP central directory, which this reader never consults on purpose.
        // So a prefix reaching the end of the deflate stream is a complete
        // manifest, and only shorter ones must fail.
        let name_len = read_u16(REAL, 26).expect("a header") as usize;
        let extra_len = read_u16(REAL, 28).expect("a header") as usize;
        let compressed = read_u32(REAL, 18).expect("a header") as usize;
        let data_end = 30 + name_len + extra_len + compressed;
        assert!(data_end < REAL.len(), "the fixture should have a trailer");

        for cut in 0..data_end {
            let prefix = REAL.get(..cut).expect("in range");
            assert!(
                RawManifest::parse(prefix).is_err(),
                "a {cut}-byte prefix, cut inside the entry, parsed"
            );
        }
        // And every prefix from there on is the whole manifest.
        for cut in [data_end, REAL.len()] {
            let prefix = REAL.get(..cut).expect("in range");
            RawManifest::parse(prefix).expect("a complete entry must parse");
        }
    }

    #[test]
    fn non_manifest_input_is_rejected() {
        assert_eq!(RawManifest::parse(b""), Err(ManifestError::NotAManifest));
        assert_eq!(
            RawManifest::parse(b"not a manifest at all"),
            Err(ManifestError::NotAManifest)
        );
    }

    #[test]
    fn an_unknown_block_is_refused_rather_than_skipped() {
        // A block we cannot identify may be the one saying what the depot
        // contains, so carrying on would mean claiming to have read a manifest
        // whose contents we do not know.
        let mut blocks = Vec::new();
        blocks.extend_from_slice(&0xDEAD_BEEF_u32.to_le_bytes());
        blocks.extend_from_slice(&4_u32.to_le_bytes());
        blocks.extend_from_slice(&[0; 4]);
        blocks.extend_from_slice(&MAGIC_END.to_le_bytes());

        assert_eq!(
            RawManifest::parse_blocks(&blocks),
            Err(ManifestError::UnknownBlock(0xDEAD_BEEF))
        );
    }

    #[test]
    fn a_manifest_without_a_payload_is_not_a_manifest() {
        let mut blocks = Vec::new();
        blocks.extend_from_slice(&MAGIC_END.to_le_bytes());
        assert_eq!(
            RawManifest::parse_blocks(&blocks),
            Err(ManifestError::MissingBlock("payload"))
        );
    }
}
