//! Steam's chunk containers.

mod vsz;
mod vz;
mod zip;

use std::fmt;

/// The largest chunk we will decompress; Steam's are 1 MiB uncompressed.
pub const MAX_CHUNK: usize = 16 * 1024 * 1024;

/// What went wrong decoding a chunk container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkError {
    /// Too short to be a container.
    Truncated,
    /// A container magic this build does not know; carries the bytes found.
    UnknownContainer(Vec<u8>),
    /// A container version this build does not know.
    UnsupportedVersion(u8),
    /// The footer magic did not match the header's container.
    BadFooter(Vec<u8>),
    /// Decompression failed.
    Decompress(String),
    /// The decompressed length did not match the footer.
    SizeMismatch {
        /// What the footer claimed.
        expected: u32,
        /// What came out.
        actual: usize,
    },
    /// The CRC-32 did not match.
    ChecksumMismatch {
        /// What the container claimed.
        expected: u32,
        /// What the bytes hash to.
        actual: u32,
    },
    /// The header and footer disagreed about the CRC.
    InconsistentChecksum {
        /// The header's copy.
        header: u32,
        /// The footer's copy.
        footer: u32,
    },
    /// A ZIP entry used a compression method Steam does not use.
    UnsupportedZipMethod(u16),
    /// The claimed size exceeds the cap.
    TooLarge {
        /// What was claimed.
        claimed: u32,
    },
}

impl fmt::Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => f.write_str("chunk container is too short"),
            Self::UnknownContainer(found) => write!(
                f,
                "unknown chunk container {:02x?} ({:?}) — expected VZ or VSZ",
                found,
                String::from_utf8_lossy(found)
            ),
            Self::UnsupportedVersion(v) => {
                write!(f, "unsupported container version {:?}", char::from(*v))
            }
            Self::BadFooter(found) => write!(
                f,
                "bad container footer {:02x?} ({:?})",
                found,
                String::from_utf8_lossy(found)
            ),
            Self::Decompress(message) => write!(f, "decompression failed: {message}"),
            Self::SizeMismatch { expected, actual } => {
                write!(f, "container claims {expected} bytes, decoded {actual}")
            }
            Self::ChecksumMismatch { expected, actual } => write!(
                f,
                "CRC-32 mismatch: expected {expected:#010x}, got {actual:#010x}"
            ),
            Self::InconsistentChecksum { header, footer } => write!(
                f,
                "header CRC {header:#010x} disagrees with footer CRC {footer:#010x}"
            ),
            Self::UnsupportedZipMethod(method) => {
                write!(f, "unsupported ZIP compression method {method}")
            }
            Self::TooLarge { claimed } => write!(f, "chunk claims {claimed} bytes"),
        }
    }
}

impl std::error::Error for ChunkError {}

/// Decodes a decrypted chunk, whichever container it uses.
pub fn decode(input: &[u8]) -> Result<Vec<u8>, ChunkError> {
    decode_with_limit(input, MAX_CHUNK)
}

/// Decodes a chunk with an explicit output cap.
pub fn decode_with_limit(input: &[u8], max_output: usize) -> Result<Vec<u8>, ChunkError> {
    let mut out = Vec::new();
    decode_into(input, max_output, &mut out)?;
    Ok(out)
}

/// Decodes into a buffer the caller owns, for reuse across chunks.
pub fn decode_into(input: &[u8], max_output: usize, out: &mut Vec<u8>) -> Result<(), ChunkError> {
    // Ordered by magic specificity: VSZ's three bytes before VZ's two.
    if vsz::matches(input) {
        return vsz::decode_into(input, max_output, out);
    }
    if vz::matches(input) {
        return vz::decode_into(input, max_output, out);
    }
    if zip::matches(input) {
        let decoded = zip::decode(input, max_output)?;
        out.clear();
        out.extend_from_slice(&decoded);
        return Ok(());
    }

    Err(ChunkError::UnknownContainer(
        input.get(..4).unwrap_or(input).to_vec(),
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    Some(u32::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VZ_CHUNK: &[u8] =
        include_bytes!("../tests/fixtures/chunk_610f4c4e6d26a61f0a35ed66117a7e693cceb4b8.bin");
    const VSZ_CHUNK: &[u8] = include_bytes!(
        "../tests/fixtures/smallest_vsz_7395dfeef25971f3be265de414de08c61ec65563.bin"
    );

    #[test]
    fn both_containers_decode_through_the_same_entry_point() {
        let lzma = decode(VZ_CHUNK).expect("the VZ chunk must decode");
        assert_eq!(lzma.len(), 333);
        assert!(lzma.starts_with(b"whitelist"));

        let zstd = decode(VSZ_CHUNK).expect("the VSZ chunk must decode");
        assert!(!zstd.is_empty());
    }

    #[test]
    fn an_unknown_container_names_the_bytes_it_found() {
        let error = decode(b"XYZQand then some").expect_err("must refuse");
        let rendered = error.to_string();
        assert!(
            rendered.contains("XYZQ"),
            "the error did not name the bytes: {rendered}"
        );
        assert!(
            rendered.contains("VZ or VSZ"),
            "the error did not say what was expected"
        );
    }

    #[test]
    fn empty_input_is_refused() {
        assert!(decode(&[]).is_err());
    }

    #[test]
    fn a_low_limit_refuses_a_chunk_rather_than_allocating() {
        // The limit must be below the fixture's 61 decoded bytes.
        assert!(matches!(
            decode_with_limit(VSZ_CHUNK, 8),
            Err(ChunkError::TooLarge { .. })
        ));
    }
}
