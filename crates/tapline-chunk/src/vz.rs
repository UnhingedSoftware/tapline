//! The `VZ` chunk container: LZMA.
//!
//! Read off real chunks fetched from `cache8-iad1.steamcontent.com` on
//! 2026-08-26 and confirmed by decoding them back into the files the manifest
//! named.
//!
//! ```text
//!  0    1    2      3          7           12                    -10        -6      -2
//! +----+----+----+------------+-----------+----------------------+----------+-------+----+
//! |'V' |'Z' |'a' |  crc32     | LZMA prop |    LZMA stream       |  crc32   | size  |'zv'|
//! +----+----+----+------------+-----------+----------------------+----------+-------+----+
//!                  u32 LE       5 bytes                            u32 LE    u32 LE
//! ```
//!
//! The CRC-32 appears in both the header and the footer and both cover the
//! decompressed bytes. The LZMA stream is raw — properties but no length prefix,
//! since the length lives in the footer — so the size is handed to `lzma-rs` out
//! of band rather than by splicing a synthetic header onto the stream.

use crate::ChunkError;

/// The header magic.
const MAGIC: [u8; 2] = *b"VZ";
/// The only version observed.
const VERSION: u8 = b'a';
/// The footer magic.
const FOOTER_MAGIC: [u8; 2] = *b"zv";

/// Header bytes before the LZMA properties.
const HEADER_LEN: usize = 7;
/// LZMA property bytes.
const PROPS_LEN: usize = 5;
/// Footer bytes: crc, size, magic.
const FOOTER_LEN: usize = 10;

/// Whether these bytes look like a `VZ` container.
#[must_use]
pub fn matches(input: &[u8]) -> bool {
    input.get(..2) == Some(&MAGIC)
}

/// Decodes a `VZ` chunk.
#[cfg(test)]
pub fn decode(input: &[u8], max_output: usize) -> Result<Vec<u8>, ChunkError> {
    let mut out = Vec::new();
    decode_into(input, max_output, &mut out)?;
    Ok(out)
}

/// Decodes into a buffer the caller owns.
///
/// What a download uses. Allocating a megabyte per chunk and freeing it again
/// is what makes an allocator raise its mmap threshold and keep the heap; a
/// reused buffer never gives it the chance.
pub fn decode_into(input: &[u8], max_output: usize, out: &mut Vec<u8>) -> Result<(), ChunkError> {
    if input.len() < HEADER_LEN + PROPS_LEN + FOOTER_LEN {
        return Err(ChunkError::Truncated);
    }

    let version = *input.get(2).ok_or(ChunkError::Truncated)?;
    if version != VERSION {
        return Err(ChunkError::UnsupportedVersion(version));
    }

    let header_crc = crate::read_u32(input, 3).ok_or(ChunkError::Truncated)?;

    let footer_start = input.len() - FOOTER_LEN;
    let footer_crc = crate::read_u32(input, footer_start).ok_or(ChunkError::Truncated)?;
    let claimed_size = crate::read_u32(input, footer_start + 4).ok_or(ChunkError::Truncated)?;
    let footer_magic: [u8; 2] = input
        .get(footer_start + 8..)
        .and_then(|s| s.try_into().ok())
        .ok_or(ChunkError::Truncated)?;

    if footer_magic != FOOTER_MAGIC {
        return Err(ChunkError::BadFooter(footer_magic.to_vec()));
    }
    // Two copies of one number. If they differ, believing either would be a
    // guess.
    if header_crc != footer_crc {
        return Err(ChunkError::InconsistentChecksum {
            header: header_crc,
            footer: footer_crc,
        });
    }
    if claimed_size as usize > max_output {
        return Err(ChunkError::TooLarge {
            claimed: claimed_size,
        });
    }

    let stream = input
        .get(HEADER_LEN..footer_start)
        .ok_or(ChunkError::Truncated)?;

    let mut reader = std::io::Cursor::new(stream);
    out.clear();
    out.reserve(claimed_size as usize);

    lzma_rs::lzma_decompress_with_options(
        &mut reader,
        out,
        &lzma_rs::decompress::Options {
            unpacked_size: lzma_rs::decompress::UnpackedSize::UseProvided(Some(u64::from(
                claimed_size,
            ))),
            memlimit: Some(max_output),
            allow_incomplete: false,
        },
    )
    .map_err(|error| ChunkError::Decompress(error.to_string()))?;

    if out.len() != claimed_size as usize {
        return Err(ChunkError::SizeMismatch {
            expected: claimed_size,
            actual: out.len(),
        });
    }

    let actual_crc = crc32fast::hash(out);
    if actual_crc != footer_crc {
        return Err(ChunkError::ChecksumMismatch {
            expected: footer_crc,
            actual: actual_crc,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAX_CHUNK;

    /// Real chunks from depot 232257, fetched and decrypted on 2026-08-26.
    const WHITELIST: &[u8] =
        include_bytes!("../tests/fixtures/chunk_610f4c4e6d26a61f0a35ed66117a7e693cceb4b8.bin");
    const SCRIPT: &[u8] =
        include_bytes!("../tests/fixtures/chunk_f94a22cb75c93b5533844f6c2d06999d291e66b8.bin");

    #[test]
    fn a_real_chunk_decodes_into_the_file_it_came_from() {
        // 333 bytes of tf/cfg/pure_server_whitelist.txt.
        let out = decode(WHITELIST, MAX_CHUNK).expect("a real chunk must decode");
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
        let out = decode(SCRIPT, MAX_CHUNK).expect("a real chunk must decode");
        assert_eq!(out.len(), 9_656);
        assert!(out.starts_with(b"#!/usr/bin/env python3"));
    }

    #[test]
    fn the_decoded_bytes_match_the_containers_own_checksum() {
        // Belt and braces: the decoder checks this itself, and so does this
        // test, because a decoder that skipped the check would still pass the
        // two above.
        for chunk in [WHITELIST, SCRIPT] {
            let footer_crc = crate::read_u32(chunk, chunk.len() - FOOTER_LEN).expect("a footer");
            let out = decode(chunk, MAX_CHUNK).expect("must decode");
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
            decode(&damaged, MAX_CHUNK).is_err(),
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
            decode(&damaged, MAX_CHUNK),
            Err(ChunkError::InconsistentChecksum { .. })
        ));
    }

    #[test]
    fn an_unknown_version_is_refused() {
        let mut future = WHITELIST.to_vec();
        if let Some(byte) = future.get_mut(2) {
            *byte = b'b';
        }
        assert_eq!(
            decode(&future, MAX_CHUNK),
            Err(ChunkError::UnsupportedVersion(b'b'))
        );
    }

    #[test]
    fn a_damaged_footer_is_refused() {
        let mut damaged = WHITELIST.to_vec();
        let len = damaged.len();
        if let Some(slot) = damaged.get_mut(len - 2..) {
            slot.copy_from_slice(b"XX");
        }
        assert_eq!(
            decode(&damaged, MAX_CHUNK),
            Err(ChunkError::BadFooter(b"XX".to_vec()))
        );
    }

    #[test]
    fn an_absurd_size_is_refused_before_allocating() {
        let mut huge = WHITELIST.to_vec();
        let len = huge.len();
        if let Some(slot) = huge.get_mut(len - 6..len - 2) {
            slot.copy_from_slice(&u32::MAX.to_le_bytes());
        }
        assert!(matches!(
            decode(&huge, MAX_CHUNK),
            Err(ChunkError::TooLarge { .. })
        ));
    }

    #[test]
    fn truncation_at_every_length_is_an_error_not_a_panic() {
        for cut in 0..WHITELIST.len() {
            let prefix = WHITELIST.get(..cut).expect("in range");
            assert!(
                decode(prefix, MAX_CHUNK).is_err(),
                "a {cut}-byte prefix decoded as a whole chunk"
            );
        }
    }

    #[test]
    fn empty_and_garbage_input_are_refused() {
        assert_eq!(decode(&[], MAX_CHUNK), Err(ChunkError::Truncated));
        assert!(decode(b"not a chunk at all, really", MAX_CHUNK).is_err());
    }
}
