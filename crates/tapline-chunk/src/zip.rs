//! The ZIP chunk container.
//!
//! The third one, and the one that made Garry's Mod fail to install at all.
//! Read off a real chunk from GMod Dedicated Server's depot 4021 on 2026-08-26:
//!
//! ```text
//! +----------+----+----+--------+--------+--------+----+----+------+---------+
//! |PK\x03\x04|ver |flg |method  | time   | crc32  |csize|usize|n,e  | 'z'|data|
//! +----------+----+----+--------+--------+--------+----+----+------+---------+
//!   0..4      4..6 6..8 8..10    10..14   14..18   18..22 22..26 26..30
//! ```
//!
//! Structurally identical to the wrapper around a manifest: one deflated entry
//! named `z`. Unlike `VZ` and `VSZ` there is no Steam footer, so the integrity
//! check is the ZIP header's own CRC-32 over the decompressed bytes — plus the
//! SHA-1 the caller checks against the chunk id, which is the real guarantee
//! either way.
//!
//! # It is not rare
//!
//! A census of GMod's depots on the same day:
//!
//! | depot | ZIP | VSZ | VZ  |
//! |-------|-----|-----|-----|
//! | 1006  |   0 |  32 |   8 |
//! | 4021  |  14 |  36 |  30 |
//! | 4023  |  18 |  53 |  49 |
//!
//! All three coexist inside a single depot, so the container is a per-chunk
//! property and dispatch cannot be hoisted out of the loop. The earlier probe
//! sampled only depot 1006 — which has no ZIP chunks at all — and concluded
//! there were two containers. That is the second time a sample of one depot
//! gave a confident wrong answer about the whole app.

use crate::ChunkError;

/// The ZIP local file header magic.
const MAGIC: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
/// Fixed header length before the name and extra fields.
const HEADER_LEN: usize = 30;
/// Stored, no compression.
const METHOD_STORE: u16 = 0;
/// Deflate, which is what Steam uses.
const METHOD_DEFLATE: u16 = 8;

/// Whether these bytes look like a ZIP-wrapped chunk.
#[must_use]
pub fn matches(input: &[u8]) -> bool {
    input.get(..4) == Some(&MAGIC)
}

/// Decodes a ZIP-wrapped chunk.
///
/// Deliberately not a general ZIP reader: a chunk holds exactly one entry, so
/// the local file header is enough and the central directory is never consulted
/// — which also sidesteps every ZIP-parsing trick that turns on the two
/// disagreeing.
pub fn decode(input: &[u8], max_output: usize) -> Result<Vec<u8>, ChunkError> {
    if input.len() < HEADER_LEN {
        return Err(ChunkError::Truncated);
    }

    let method = read_u16(input, 8).ok_or(ChunkError::Truncated)?;
    let expected_crc = crate::read_u32(input, 14).ok_or(ChunkError::Truncated)?;
    let compressed_size = crate::read_u32(input, 18).ok_or(ChunkError::Truncated)? as usize;
    let uncompressed_size = crate::read_u32(input, 22).ok_or(ChunkError::Truncated)?;
    let name_len = read_u16(input, 26).ok_or(ChunkError::Truncated)? as usize;
    let extra_len = read_u16(input, 28).ok_or(ChunkError::Truncated)? as usize;

    if uncompressed_size as usize > max_output {
        return Err(ChunkError::TooLarge {
            claimed: uncompressed_size,
        });
    }

    let data_start = HEADER_LEN
        .checked_add(name_len)
        .and_then(|value| value.checked_add(extra_len))
        .ok_or(ChunkError::Truncated)?;
    let data_end = data_start
        .checked_add(compressed_size)
        .ok_or(ChunkError::Truncated)?;
    let body = input
        .get(data_start..data_end)
        .ok_or(ChunkError::Truncated)?;

    let out = match method {
        METHOD_STORE => body.to_vec(),
        METHOD_DEFLATE => miniz_oxide::inflate::decompress_to_vec_with_limit(body, max_output)
            .map_err(|error| match error.status {
                miniz_oxide::inflate::TINFLStatus::HasMoreOutput => ChunkError::TooLarge {
                    claimed: uncompressed_size,
                },
                status => ChunkError::Decompress(format!("{status:?}")),
            })?,
        other => return Err(ChunkError::UnsupportedZipMethod(other)),
    };

    if out.len() != uncompressed_size as usize {
        return Err(ChunkError::SizeMismatch {
            expected: uncompressed_size,
            actual: out.len(),
        });
    }

    // The ZIP header carries a CRC-32 and checking it costs nothing next to the
    // transfer that delivered these bytes.
    let actual_crc = crc32fast::hash(&out);
    if actual_crc != expected_crc {
        return Err(ChunkError::ChecksumMismatch {
            expected: expected_crc,
            actual: actual_crc,
        });
    }

    Ok(out)
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let end = offset.checked_add(2)?;
    Some(u16::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAX_CHUNK;

    /// A real ZIP-wrapped chunk from GMod Dedicated Server's depot 4021,
    /// captured 2026-08-26. Its SHA-1 is its filename, which is what makes the
    /// decode verifiable here without any network.
    const REAL: &[u8] =
        include_bytes!("../tests/fixtures/zip_36b7e46fbf001df8a67217d7313c8cea2648cdec.bin");

    /// The chunk id: the SHA-1 of the plaintext.
    const REAL_ID: [u8; 20] = [
        0x36, 0xb7, 0xe4, 0x6f, 0xbf, 0x00, 0x1d, 0xf8, 0xa6, 0x72, 0x17, 0xd7, 0x31, 0x3c, 0x8c,
        0xea, 0x26, 0x48, 0xcd, 0xec,
    ];

    #[test]
    fn a_real_zip_chunk_decodes_to_steams_chunk_size() {
        let out = decode(REAL, MAX_CHUNK).expect("a real chunk must decode");
        assert_eq!(out.len(), 1_048_576);
    }

    #[test]
    fn the_decoded_bytes_hash_to_the_chunk_id() {
        // The check that matters: the id *is* the SHA-1 of the plaintext, so
        // this proves the decode produced the content Steam named rather than
        // merely something of the right length.
        let out = decode(REAL, MAX_CHUNK).expect("must decode");
        let mut hasher = <sha1::Sha1 as sha1::Digest>::new();
        sha1::Digest::update(&mut hasher, &out);
        let digest: [u8; 20] = sha1::Digest::finalize(hasher).into();
        assert_eq!(digest, REAL_ID);
    }

    #[test]
    fn it_is_recognised_by_magic() {
        assert!(matches(REAL));
        assert!(!matches(b"VZa\0"));
        assert!(!matches(b"VSZa"));
        assert!(!matches(b""));
    }

    #[test]
    fn a_corrupted_payload_is_caught() {
        let mut damaged = REAL.to_vec();
        let middle = damaged.len() / 2;
        if let Some(byte) = damaged.get_mut(middle) {
            *byte ^= 0xFF;
        }
        assert!(
            decode(&damaged, MAX_CHUNK).is_err(),
            "a corrupted ZIP chunk decoded without complaint"
        );
    }

    #[test]
    fn a_tampered_checksum_is_caught() {
        let mut damaged = REAL.to_vec();
        if let Some(byte) = damaged.get_mut(14) {
            *byte ^= 0x01;
        }
        assert!(matches!(
            decode(&damaged, MAX_CHUNK),
            Err(ChunkError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn an_absurd_size_is_refused_before_decompressing() {
        let mut huge = REAL.to_vec();
        if let Some(slot) = huge.get_mut(22..26) {
            slot.copy_from_slice(&u32::MAX.to_le_bytes());
        }
        assert!(matches!(
            decode(&huge, MAX_CHUNK),
            Err(ChunkError::TooLarge { .. })
        ));
    }

    #[test]
    fn an_unsupported_compression_method_names_itself() {
        // bzip2 and lzma are legal ZIP methods Steam does not use. If one ever
        // appears, the error should say which rather than "malformed".
        let mut odd = REAL.to_vec();
        if let Some(slot) = odd.get_mut(8..10) {
            slot.copy_from_slice(&12_u16.to_le_bytes());
        }
        assert_eq!(
            decode(&odd, MAX_CHUNK),
            Err(ChunkError::UnsupportedZipMethod(12))
        );
    }

    #[test]
    fn truncation_is_an_error_not_a_panic() {
        // Stepped: the fixture is 462 KB and inflating prefixes is not free.
        for cut in (0..REAL.len()).step_by(4099) {
            let prefix = REAL.get(..cut).expect("in range");
            assert!(
                decode(prefix, MAX_CHUNK).is_err(),
                "a {cut}-byte prefix decoded as a whole chunk"
            );
        }
    }
}
