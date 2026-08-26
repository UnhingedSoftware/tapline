//! The `VZ` chunk container.
//!
//! Every byte of depot content arrives wrapped in this. The layout is not
//! documented anywhere; it was read off real chunks fetched from
//! `cache8-iad1.steamcontent.com` on 2026-08-26 and confirmed by decoding them
//! back into the files the manifest named.
//!
//! ```text
//!  0    1    2      3          7           12                    -10        -6      -2
//! +----+----+----+------------+-----------+----------------------+----------+-------+----+
//! |'V' |'Z' |'a' |  crc32     | LZMA prop |    LZMA stream       |  crc32   | size  |'zv'|
//! +----+----+----+------------+-----------+----------------------+----------+-------+----+
//!                  u32 LE       5 bytes                            u32 LE    u32 LE
//! ```
//!
//! Two things are worth knowing about it.
//!
//! **The CRC-32 appears twice**, in the header and again in the footer, and both
//! cover the *decompressed* bytes. Checking them costs nothing next to the
//! network transfer that delivered the chunk, and it catches a corrupted stream
//! before the SHA-1 check would.
//!
//! **The LZMA stream is raw**, with properties but no length prefix — the length
//! is in the footer. `lzma-rs` wants the `.lzma` "alone" framing, so the size is
//! supplied out of band rather than by splicing a synthetic header onto the
//! stream.
//!
//! # What about zstd?
//!
//! Steam's schema implies a `VSZ` variant. Probed on 2026-08-26 across Team
//! Fortress 2 DS (232250), Counter-Strike 2 DS (740) and Valheim DS (896660):
//! every chunk came back `VZ`. Rather than write a decoder for a container
//! nothing was observed to serve, an unrecognised magic is reported with the
//! bytes that were found — so if `VSZ` does turn up, the error names it instead
//! of failing somewhere less obvious.

use std::fmt;

/// The header magic: `VZ`.
const MAGIC: [u8; 2] = *b"VZ";
/// The only version observed.
const VERSION: u8 = b'a';
/// The footer magic: `zv`.
const FOOTER_MAGIC: [u8; 2] = *b"zv";

/// Header bytes before the LZMA properties.
const HEADER_LEN: usize = 7;
/// LZMA property bytes.
const PROPS_LEN: usize = 5;
/// Footer bytes: crc, size, magic.
const FOOTER_LEN: usize = 10;

/// The largest chunk we will decompress.
///
/// Steam's chunks are at most 1 MiB uncompressed — measured across three apps,
/// every chunk reported exactly 1,048,576 or less. The cap is set well above
/// that so a legitimate change in Valve's chunker does not break downloads,
/// and far below anything that would trouble a small node.
pub const MAX_CHUNK: usize = 16 * 1024 * 1024;

/// What went wrong decoding a chunk container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VzError {
    /// Too short to be a container.
    Truncated,
    /// The header magic was not `VZ`.
    ///
    /// Carries what was found, so a `VSZ` chunk — which nothing was observed to
    /// serve — would be identifiable rather than a mystery.
    BadMagic([u8; 2]),
    /// A container version this build does not know.
    UnsupportedVersion(u8),
    /// The footer magic was not `zv`.
    BadFooter([u8; 2]),
    /// The LZMA stream did not decode.
    Lzma(String),
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
    ///
    /// They are two copies of one value; disagreement means the container is
    /// damaged in a way that a single check might not have noticed.
    InconsistentChecksum {
        /// The header's copy.
        header: u32,
        /// The footer's copy.
        footer: u32,
    },
    /// The claimed size exceeds [`MAX_CHUNK`].
    TooLarge {
        /// What was claimed.
        claimed: u32,
    },
}

impl fmt::Display for VzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => f.write_str("chunk container is too short"),
            Self::BadMagic(found) => write!(
                f,
                "expected a VZ container, found magic {:02x?} ({:?})",
                found,
                String::from_utf8_lossy(found)
            ),
            Self::UnsupportedVersion(v) => {
                write!(f, "unsupported container version {:?}", char::from(*v))
            }
            Self::BadFooter(found) => write!(f, "bad container footer {found:02x?}"),
            Self::Lzma(message) => write!(f, "LZMA decode failed: {message}"),
            Self::SizeMismatch { expected, actual } => {
                write!(f, "container claims {expected} bytes, decoded {actual}")
            }
            Self::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "CRC-32 mismatch: expected {expected:#010x}, got {actual:#010x}"
                )
            }
            Self::InconsistentChecksum { header, footer } => write!(
                f,
                "header CRC {header:#010x} disagrees with footer CRC {footer:#010x}"
            ),
            Self::TooLarge { claimed } => write!(f, "chunk claims {claimed} bytes"),
        }
    }
}

impl std::error::Error for VzError {}

/// Decodes one `VZ` chunk into its plaintext.
///
/// The input is the *decrypted* container: chunks arrive AES-encrypted and are
/// decrypted with the depot key before reaching here.
pub fn decode(input: &[u8]) -> Result<Vec<u8>, VzError> {
    if input.len() < HEADER_LEN + PROPS_LEN + FOOTER_LEN {
        return Err(VzError::Truncated);
    }

    let magic: [u8; 2] = input
        .get(..2)
        .and_then(|s| s.try_into().ok())
        .ok_or(VzError::Truncated)?;
    if magic != MAGIC {
        return Err(VzError::BadMagic(magic));
    }

    let version = *input.get(2).ok_or(VzError::Truncated)?;
    if version != VERSION {
        return Err(VzError::UnsupportedVersion(version));
    }

    let header_crc = read_u32(input, 3).ok_or(VzError::Truncated)?;

    let footer_start = input.len() - FOOTER_LEN;
    let footer_crc = read_u32(input, footer_start).ok_or(VzError::Truncated)?;
    let claimed_size = read_u32(input, footer_start + 4).ok_or(VzError::Truncated)?;
    let footer_magic: [u8; 2] = input
        .get(footer_start + 8..)
        .and_then(|s| s.try_into().ok())
        .ok_or(VzError::Truncated)?;

    if footer_magic != FOOTER_MAGIC {
        return Err(VzError::BadFooter(footer_magic));
    }
    // Two copies of one number. If they differ, believing either would be a
    // guess.
    if header_crc != footer_crc {
        return Err(VzError::InconsistentChecksum {
            header: header_crc,
            footer: footer_crc,
        });
    }
    if claimed_size as usize > MAX_CHUNK {
        return Err(VzError::TooLarge {
            claimed: claimed_size,
        });
    }

    // Properties and the raw stream. `lzma-rs` is told the output size rather
    // than reading it from a header the stream does not have.
    let stream = input
        .get(HEADER_LEN..footer_start)
        .ok_or(VzError::Truncated)?;

    let mut reader = std::io::Cursor::new(stream);
    let mut out = Vec::with_capacity(claimed_size as usize);

    lzma_rs::lzma_decompress_with_options(
        &mut reader,
        &mut out,
        &lzma_rs::decompress::Options {
            unpacked_size: lzma_rs::decompress::UnpackedSize::UseProvided(Some(u64::from(
                claimed_size,
            ))),
            memlimit: Some(MAX_CHUNK),
            allow_incomplete: false,
        },
    )
    .map_err(|error| VzError::Lzma(error.to_string()))?;

    if out.len() != claimed_size as usize {
        return Err(VzError::SizeMismatch {
            expected: claimed_size,
            actual: out.len(),
        });
    }

    let actual_crc = crc32fast::hash(&out);
    if actual_crc != footer_crc {
        return Err(VzError::ChecksumMismatch {
            expected: footer_crc,
            actual: actual_crc,
        });
    }

    Ok(out)
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    Some(u32::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real chunks from depot 232257, fetched and decrypted on 2026-08-26.
    const WHITELIST: &[u8] =
        include_bytes!("../tests/fixtures/chunk_610f4c4e6d26a61f0a35ed66117a7e693cceb4b8.bin");
    const SCRIPT: &[u8] =
        include_bytes!("../tests/fixtures/chunk_f94a22cb75c93b5533844f6c2d06999d291e66b8.bin");

    #[test]
    fn a_real_chunk_decodes_into_the_file_it_came_from() {
        // 333 bytes of tf/cfg/pure_server_whitelist.txt.
        let out = decode(WHITELIST).expect("a real chunk must decode");
        assert_eq!(out.len(), 333);
        assert!(
            out.starts_with(b"whitelist"),
            "decoded into something that is not the file: {:?}",
            String::from_utf8_lossy(out.get(..40).unwrap_or_default())
        );
    }

    #[test]
    fn a_larger_real_chunk_decodes_too() {
        // 9,656 bytes of tf/cfg/unencrypted/print_instance_config.py.
        let out = decode(SCRIPT).expect("a real chunk must decode");
        assert_eq!(out.len(), 9_656);
        assert!(out.starts_with(b"#!/usr/bin/env python3"));
    }

    #[test]
    fn the_decoded_bytes_match_the_containers_own_checksum() {
        // Belt and braces: the decoder checks this itself, and so does this
        // test, because a decoder that skipped the check would still pass the
        // two above.
        for chunk in [WHITELIST, SCRIPT] {
            let footer_crc = read_u32(chunk, chunk.len() - FOOTER_LEN).expect("a footer");
            let out = decode(chunk).expect("must decode");
            assert_eq!(crc32fast::hash(&out), footer_crc);
        }
    }

    #[test]
    fn a_corrupted_stream_is_caught() {
        // A flipped bit in the compressed data either fails to decode or
        // decodes to something with the wrong CRC. Either way it must not be
        // returned as content.
        let mut damaged = SCRIPT.to_vec();
        let middle = damaged.len() / 2;
        if let Some(byte) = damaged.get_mut(middle) {
            *byte ^= 0xFF;
        }
        assert!(
            decode(&damaged).is_err(),
            "a corrupted chunk decoded without complaint"
        );
    }

    #[test]
    fn a_tampered_checksum_is_caught_as_an_inconsistency() {
        // The header and footer hold the same value; changing one is the
        // cheapest way to notice damage that a single copy would hide.
        let mut damaged = WHITELIST.to_vec();
        if let Some(byte) = damaged.get_mut(3) {
            *byte ^= 0x01;
        }
        assert!(matches!(
            decode(&damaged),
            Err(VzError::InconsistentChecksum { .. })
        ));
    }

    #[test]
    fn a_foreign_container_is_named_rather_than_guessed_at() {
        // If Valve ever serves the zstd variant, this is the error that says so.
        let mut vsz = WHITELIST.to_vec();
        if let Some(slot) = vsz.get_mut(..2) {
            slot.copy_from_slice(b"VS");
        }
        assert_eq!(decode(&vsz), Err(VzError::BadMagic(*b"VS")));
    }

    #[test]
    fn an_unknown_version_is_refused() {
        let mut future = WHITELIST.to_vec();
        if let Some(byte) = future.get_mut(2) {
            *byte = b'b';
        }
        assert_eq!(decode(&future), Err(VzError::UnsupportedVersion(b'b')));
    }

    #[test]
    fn a_damaged_footer_is_refused() {
        let mut damaged = WHITELIST.to_vec();
        let len = damaged.len();
        if let Some(slot) = damaged.get_mut(len - 2..) {
            slot.copy_from_slice(b"XX");
        }
        assert_eq!(decode(&damaged), Err(VzError::BadFooter(*b"XX")));
    }

    #[test]
    fn an_absurd_size_is_refused_before_allocating() {
        let mut huge = WHITELIST.to_vec();
        let len = huge.len();
        if let Some(slot) = huge.get_mut(len - 6..len - 2) {
            slot.copy_from_slice(&u32::MAX.to_le_bytes());
        }
        assert!(matches!(decode(&huge), Err(VzError::TooLarge { .. })));
    }

    #[test]
    fn truncation_at_every_length_is_an_error_not_a_panic() {
        for cut in 0..WHITELIST.len() {
            let prefix = WHITELIST.get(..cut).expect("in range");
            assert!(
                decode(prefix).is_err(),
                "a {cut}-byte prefix decoded as a whole chunk"
            );
        }
    }

    #[test]
    fn empty_and_garbage_input_are_refused() {
        assert_eq!(decode(&[]), Err(VzError::Truncated));
        assert_eq!(
            decode(b"not a chunk at all, really"),
            Err(VzError::BadMagic(*b"no"))
        );
    }
}
