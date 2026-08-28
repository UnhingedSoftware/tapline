//! The ZIP chunk container: one deflated entry named `z`, no Steam footer.

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

/// Decodes a ZIP-wrapped chunk from its local header alone; one entry, no directory.
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

    const REAL: &[u8] = include_bytes!(
        "../tests/fixtures/smallest_zip_ba8ab5a0280b953aa97435ff8946cbcbb2755a27.bin"
    );

    /// The chunk id: the SHA-1 of the plaintext.
    const REAL_ID: [u8; 20] = [
        0xba, 0x8a, 0xb5, 0xa0, 0x28, 0x0b, 0x95, 0x3a, 0xa9, 0x74, 0x35, 0xff, 0x89, 0x46, 0xcb,
        0xcb, 0xb2, 0x75, 0x5a, 0x27,
    ];

    #[test]
    fn the_decoded_bytes_hash_to_the_chunk_id() {
        // The chunk id is the SHA-1 of the plaintext.
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
        // Damage the payload specifically; the CRC protects nothing else.
        const LOCAL_HEADER: usize = 30;
        let name_len = usize::from(read_u16(REAL, 26).expect("a name length"));
        let payload = LOCAL_HEADER + name_len;

        let mut damaged = REAL.to_vec();
        let byte = damaged.get_mut(payload).expect("the fixture has a payload");
        *byte ^= 0xFF;

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
        // Stepped: inflating every prefix is not free.
        for cut in (0..REAL.len()).step_by(4099) {
            let prefix = REAL.get(..cut).expect("in range");
            assert!(
                decode(prefix, MAX_CHUNK).is_err(),
                "a {cut}-byte prefix decoded as a whole chunk"
            );
        }
    }
}
