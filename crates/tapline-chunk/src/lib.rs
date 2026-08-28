//! Steam's chunk containers.
//!
//! Every byte of depot content arrives wrapped in one of two containers, and
//! which one is decided per chunk rather than per depot:
//!
//! * **`VZ`** — LZMA, footer magic `zv`.
//! * **`VSZ`** — zstd, footer magic `zsv`.
//! * **ZIP** — `PK\x03\x04`, one deflated entry named `z`, no Steam footer.
//!
//! Both were read off real chunks rather than from a description, and both
//! carry the CRC-32 of their decompressed bytes twice, in the header and again
//! in the footer.
//!
//! # A measurement that was right and a conclusion that was wrong
//!
//! An earlier probe fetched three chunks from Team Fortress 2's smallest depot,
//! saw `VZ` all three times, and concluded that nothing serves the zstd
//! container — so the zstd decoder was left unwritten and an unrecognised magic
//! was made to name itself in the error.
//!
//! A full Valheim install then failed with `expected a VZ container, found
//! magic "VS"`. Sampling that depot properly gives **32 `VSZ` to every 8 `VZ`**:
//! zstd is the majority, and the original sample was three chunks of one small
//! depot, which is not the same thing as a depot.
//!
//! The measurement was sound and the inference from it was not. What made the
//! difference in the end was the error naming the bytes it found instead of
//! saying "malformed chunk" — five seconds to diagnose instead of an afternoon.
//! That is the argument for reporting unknown input rather than skipping it,
//! stated by example.
//!
//! # And then it happened again
//!
//! Installing Garry's Mod Dedicated Server failed outright on `PK\x03\x04`: a
//! third container, plain ZIP. The probe that had found `VSZ` sampled depot
//! 1006 — which contains no ZIP chunks at all — and stopped there. A census
//! across every GMod depot afterwards:
//!
//! | depot | ZIP | VSZ | VZ  |
//! |-------|-----|-----|-----|
//! | 1006  |   0 |  32 |   8 |
//! | 4021  |  14 |  36 |  30 |
//! | 4023  |  18 |  53 |  49 |
//!
//! Twice now, a sample of one depot has given a confident wrong answer about a
//! whole app. All three containers coexist within a single depot, so the choice
//! is per chunk and the dispatch below cannot be hoisted out of the loop.

mod vsz;
mod vz;
mod zip;

use std::fmt;

/// The largest chunk we will decompress.
///
/// Steam's chunks are 1 MiB uncompressed. The cap is set well above that so a
/// change in Valve's chunker does not break downloads, and far below anything
/// that would trouble a small node.
pub const MAX_CHUNK: usize = 16 * 1024 * 1024;

/// What went wrong decoding a chunk container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkError {
    /// Too short to be a container.
    Truncated,
    /// A container magic this build does not know.
    ///
    /// Carries the bytes found. This is the error that turned a mystery into a
    /// five-second diagnosis when `VSZ` first appeared.
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

/// Decodes a chunk, whichever container it uses.
///
/// The input is the *decrypted* container: chunks arrive AES-encrypted and are
/// decrypted with the depot key before reaching here.
pub fn decode(input: &[u8]) -> Result<Vec<u8>, ChunkError> {
    decode_with_limit(input, MAX_CHUNK)
}

/// Decodes a chunk with an explicit output cap.
pub fn decode_with_limit(input: &[u8], max_output: usize) -> Result<Vec<u8>, ChunkError> {
    let mut out = Vec::new();
    decode_into(input, max_output, &mut out)?;
    Ok(out)
}

/// Decodes into a buffer the caller owns.
///
/// The form a download uses: a chunk's plaintext is a megabyte, and allocating
/// one per chunk only to free it is what drives an allocator to grow its heap
/// and hold on to it. Reusing a buffer keeps peak memory at the number of
/// chunks in flight rather than at whatever the allocator decided to retain.
pub fn decode_into(input: &[u8], max_output: usize, out: &mut Vec<u8>) -> Result<(), ChunkError> {
    // VSZ is checked first: its magic is three bytes and VZ's is two, so
    // checking VZ first would match the "VZ" inside... nothing, in fact — the
    // two magics differ at byte 1 — but ordering by specificity keeps that true
    // if a third container ever appears.
    if vsz::matches(input) {
        return vsz::decode_into(input, max_output, out);
    }
    if vz::matches(input) {
        return vz::decode_into(input, max_output, out);
    }
    if zip::matches(input) {
        // The rarest container by far, and miniz_oxide has no decode-into-a-Vec
        // form, so this one still allocates and copies.
        let decoded = zip::decode(input, max_output)?;
        out.clear();
        out.extend_from_slice(&decoded);
        return Ok(());
    }

    Err(ChunkError::UnknownContainer(
        input.get(..4).unwrap_or(input).to_vec(),
    ))
}

/// Reads a little-endian `u32`, shared by both containers.
fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    Some(u32::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VZ_CHUNK: &[u8] =
        include_bytes!("../tests/fixtures/chunk_610f4c4e6d26a61f0a35ed66117a7e693cceb4b8.bin");
    const VSZ_CHUNK: &[u8] = include_bytes!("../tests/fixtures/smallest_vsz_7395dfeef25971f3be265de414de08c61ec65563.bin");

    #[test]
    fn both_containers_decode_through_the_same_entry_point() {
        // A caller does not know which container a chunk uses until it arrives,
        // so dispatch has to happen here rather than at the call site.
        let lzma = decode(VZ_CHUNK).expect("the VZ chunk must decode");
        assert_eq!(lzma.len(), 333);
        assert!(lzma.starts_with(b"whitelist"));

        let zstd = decode(VSZ_CHUNK).expect("the VSZ chunk must decode");
        assert!(!zstd.is_empty());
    }

    #[test]
    fn an_unknown_container_names_the_bytes_it_found() {
        // The property that turned a mystery into a five-second diagnosis when
        // VSZ first appeared in a real install.
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
        // A limit below what the chunk decodes to. The fixture is 61 bytes, so
        // the limit has to be smaller than that rather than a round number
        // chosen when the fixture was a megabyte.
        assert!(matches!(
            decode_with_limit(VSZ_CHUNK, 8),
            Err(ChunkError::TooLarge { .. })
        ));
    }
}
