use crate::ChunkError;
use std::io::Read as _;

const MAGIC: [u8; 3] = *b"VSZ";
const VERSION: u8 = b'a';
const FOOTER_MAGIC: [u8; 3] = *b"zsv";

const HEADER_LEN: usize = 8;
const FOOTER_LEN: usize = 15;

#[must_use]
pub fn matches(input: &[u8]) -> bool {
    input.get(..3) == Some(&MAGIC)
}

#[cfg(test)]
pub fn decode(input: &[u8], max_output: usize) -> Result<Vec<u8>, ChunkError> {
    let mut out = Vec::new();
    decode_into(input, max_output, &mut out)?;
    Ok(out)
}

pub fn decode_into(input: &[u8], max_output: usize, out: &mut Vec<u8>) -> Result<(), ChunkError> {
    if input.len() < HEADER_LEN + FOOTER_LEN {
        return Err(ChunkError::Truncated);
    }

    let version = *input.get(3).ok_or(ChunkError::Truncated)?;
    if version != VERSION {
        return Err(ChunkError::UnsupportedVersion(version));
    }

    let header_crc = crate::read_u32(input, 4).ok_or(ChunkError::Truncated)?;

    let footer_start = input.len() - FOOTER_LEN;
    let footer_crc = crate::read_u32(input, footer_start).ok_or(ChunkError::Truncated)?;
    let claimed_size = crate::read_u32(input, footer_start + 4).ok_or(ChunkError::Truncated)?;
    let footer_magic: [u8; 3] = input
        .get(footer_start + 12..)
        .and_then(|s| s.try_into().ok())
        .ok_or(ChunkError::Truncated)?;

    if footer_magic != FOOTER_MAGIC {
        return Err(ChunkError::BadFooter(footer_magic.to_vec()));
    }
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

    let frame = input
        .get(HEADER_LEN..footer_start)
        .ok_or(ChunkError::Truncated)?;

    let mut reader = ruzstd::decoding::StreamingDecoder::new(frame)
        .map_err(|e| ChunkError::Decompress(e.to_string()))?;

    out.clear();
    out.reserve(claimed_size as usize);
    (&mut reader)
        .take(u64::from(claimed_size) + 1)
        .read_to_end(out)
        .map_err(|e| ChunkError::Decompress(e.to_string()))?;

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

    const REAL: &[u8] = include_bytes!(
        "../tests/fixtures/smallest_vsz_7395dfeef25971f3be265de414de08c61ec65563.bin"
    );

    #[test]
    fn a_real_vsz_chunk_decodes() {
        let out = decode(REAL, MAX_CHUNK).expect("a real chunk must decode");
        assert!(!out.is_empty());
    }

    #[test]
    fn the_output_matches_the_containers_own_checksum() {
        let footer_crc = crate::read_u32(REAL, REAL.len() - FOOTER_LEN).expect("a footer");
        let out = decode(REAL, MAX_CHUNK).expect("must decode");
        assert_eq!(crc32fast::hash(&out), footer_crc);
    }

    #[test]
    fn it_is_recognised_by_magic() {
        assert!(matches(REAL));
        assert!(!matches(b"VZa"));
        assert!(!matches(b""));
    }

    #[test]
    fn a_corrupted_frame_is_caught() {
        let mut damaged = REAL.to_vec();
        let middle = damaged.len() / 2;
        if let Some(byte) = damaged.get_mut(middle) {
            *byte ^= 0xFF;
        }
        assert!(decode(&damaged, MAX_CHUNK).is_err());
    }

    #[test]
    fn a_tampered_checksum_shows_up_as_an_inconsistency() {
        let mut damaged = REAL.to_vec();
        if let Some(byte) = damaged.get_mut(4) {
            *byte ^= 0x01;
        }
        assert!(matches!(
            decode(&damaged, MAX_CHUNK),
            Err(ChunkError::InconsistentChecksum { .. })
        ));
    }

    #[test]
    fn an_absurd_size_is_refused_before_decompressing() {
        let mut huge = REAL.to_vec();
        let len = huge.len();
        if let Some(slot) = huge.get_mut(len - 11..len - 7) {
            slot.copy_from_slice(&u32::MAX.to_le_bytes());
        }
        assert!(matches!(
            decode(&huge, MAX_CHUNK),
            Err(ChunkError::TooLarge { .. })
        ));
    }

    #[test]
    fn truncation_is_an_error_not_a_panic() {
        for cut in (0..REAL.len()).step_by(997) {
            let prefix = REAL.get(..cut).expect("in range");
            assert!(
                decode(prefix, MAX_CHUNK).is_err(),
                "a {cut}-byte prefix decoded as a whole chunk"
            );
        }
    }
}
