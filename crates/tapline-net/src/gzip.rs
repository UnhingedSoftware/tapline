//! Just enough gzip (RFC 1952) to unwrap a `CMsgMulti` payload.

use std::fmt;

const MAGIC: [u8; 2] = [0x1F, 0x8B];
const METHOD_DEFLATE: u8 = 8;
const HEADER_LEN: usize = 10;
// Trailer: CRC-32 then ISIZE.
const TRAILER_LEN: usize = 8;

const FLG_FHCRC: u8 = 1 << 1;
const FLG_FEXTRA: u8 = 1 << 2;
const FLG_FNAME: u8 = 1 << 3;
const FLG_FCOMMENT: u8 = 1 << 4;

/// What went wrong unwrapping a gzip stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GzipError {
    /// The stream did not start with the gzip magic, or ended too early.
    Malformed,
    /// A compression method other than deflate.
    UnsupportedMethod(u8),
    /// Inflation failed.
    Inflate(String),
    /// The output would have exceeded the caller's limit.
    TooLarge,
    /// The trailer's CRC-32 did not match what we decompressed.
    ChecksumMismatch,
    /// The trailer's length did not match what we decompressed.
    LengthMismatch {
        /// What the trailer claimed.
        expected: u32,
        /// What came out.
        actual: usize,
    },
}

impl fmt::Display for GzipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed => f.write_str("not a well-formed gzip stream"),
            Self::UnsupportedMethod(m) => write!(f, "unsupported compression method {m}"),
            Self::Inflate(e) => write!(f, "inflate failed: {e}"),
            Self::TooLarge => f.write_str("decompressed output exceeded the limit"),
            Self::ChecksumMismatch => f.write_str("gzip CRC-32 mismatch"),
            Self::LengthMismatch { expected, actual } => {
                write!(f, "gzip trailer claims {expected} bytes, got {actual}")
            }
        }
    }
}

impl std::error::Error for GzipError {}

/// Decompresses a gzip stream, refusing to produce more than `limit` bytes.
pub fn decompress(data: &[u8], limit: usize) -> Result<Vec<u8>, GzipError> {
    let magic = data.get(..2).ok_or(GzipError::Malformed)?;
    if magic != MAGIC {
        return Err(GzipError::Malformed);
    }

    let method = *data.get(2).ok_or(GzipError::Malformed)?;
    if method != METHOD_DEFLATE {
        return Err(GzipError::UnsupportedMethod(method));
    }

    let flags = *data.get(3).ok_or(GzipError::Malformed)?;
    let mut cursor = HEADER_LEN;

    if flags & FLG_FEXTRA != 0 {
        let len_bytes: [u8; 2] = data
            .get(cursor..cursor + 2)
            .and_then(|s| s.try_into().ok())
            .ok_or(GzipError::Malformed)?;
        cursor = cursor
            .checked_add(2 + usize::from(u16::from_le_bytes(len_bytes)))
            .ok_or(GzipError::Malformed)?;
    }
    if flags & FLG_FNAME != 0 {
        cursor = skip_nul_terminated(data, cursor)?;
    }
    if flags & FLG_FCOMMENT != 0 {
        cursor = skip_nul_terminated(data, cursor)?;
    }
    if flags & FLG_FHCRC != 0 {
        cursor = cursor.checked_add(2).ok_or(GzipError::Malformed)?;
    }

    let body_end = data
        .len()
        .checked_sub(TRAILER_LEN)
        .ok_or(GzipError::Malformed)?;
    let body = data.get(cursor..body_end).ok_or(GzipError::Malformed)?;
    let trailer = data.get(body_end..).ok_or(GzipError::Malformed)?;

    let out = miniz_oxide::inflate::decompress_to_vec_with_limit(body, limit).map_err(|e| {
        if matches!(e.status, miniz_oxide::inflate::TINFLStatus::HasMoreOutput) {
            GzipError::TooLarge
        } else {
            GzipError::Inflate(format!("{:?}", e.status))
        }
    })?;

    let expected_crc = u32::from_le_bytes(
        trailer
            .get(..4)
            .and_then(|s| s.try_into().ok())
            .ok_or(GzipError::Malformed)?,
    );
    let expected_len = u32::from_le_bytes(
        trailer
            .get(4..8)
            .and_then(|s| s.try_into().ok())
            .ok_or(GzipError::Malformed)?,
    );

    // ISIZE is the length modulo 2^32, which is the only thing gzip promises.
    if expected_len != (out.len() as u64 % (1_u64 << 32)) as u32 {
        return Err(GzipError::LengthMismatch {
            expected: expected_len,
            actual: out.len(),
        });
    }
    if crc32fast::hash(&out) != expected_crc {
        return Err(GzipError::ChecksumMismatch);
    }

    Ok(out)
}

fn skip_nul_terminated(data: &[u8], from: usize) -> Result<usize, GzipError> {
    let tail = data.get(from..).ok_or(GzipError::Malformed)?;
    let nul = tail
        .iter()
        .position(|b| *b == 0)
        .ok_or(GzipError::Malformed)?;
    from.checked_add(nul + 1).ok_or(GzipError::Malformed)
}

/// Builds test fixtures; nothing in the protocol asks tapline to produce gzip.
#[cfg(test)]
pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC);
    out.push(METHOD_DEFLATE);
    out.push(0); // no optional fields
    out.extend_from_slice(&0_u32.to_le_bytes()); // MTIME
    out.push(0); // XFL
    out.push(255); // OS: unknown
    out.extend_from_slice(&miniz_oxide::deflate::compress_to_vec(data, 6));
    out.extend_from_slice(&crc32fast::hash(data).to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_round_trip_survives() {
        let original = b"a batch of steam messages, repeated: aaaaaaaaaaaaaaaaaaaaaaaa";
        let compressed = compress(original);
        assert_eq!(
            decompress(&compressed, 1024).expect("must decompress"),
            original
        );
    }

    #[test]
    fn optional_header_fields_are_skipped() {
        let payload = b"payload";
        let deflated = miniz_oxide::deflate::compress_to_vec(payload, 6);

        let mut stream = Vec::new();
        stream.extend_from_slice(&MAGIC);
        stream.push(METHOD_DEFLATE);
        stream.push(FLG_FNAME | FLG_FCOMMENT);
        stream.extend_from_slice(&0_u32.to_le_bytes());
        stream.push(0);
        stream.push(255);
        stream.extend_from_slice(b"name.txt\0");
        stream.extend_from_slice(b"a comment\0");
        stream.extend_from_slice(&deflated);
        stream.extend_from_slice(&crc32fast::hash(payload).to_le_bytes());
        stream.extend_from_slice(&(payload.len() as u32).to_le_bytes());

        assert_eq!(decompress(&stream, 1024).expect("must decompress"), payload);
    }

    #[test]
    fn a_corrupted_payload_is_caught_by_the_checksum() {
        let mut compressed = compress(b"important message");
        let last = compressed.len() - TRAILER_LEN - 1;
        if let Some(byte) = compressed.get_mut(last) {
            *byte ^= 0xFF;
        }
        assert!(matches!(
            decompress(&compressed, 1024),
            Err(GzipError::ChecksumMismatch | GzipError::Inflate(_))
        ));
    }

    #[test]
    fn a_lying_length_is_caught() {
        let mut compressed = compress(b"twelve bytes");
        let len = compressed.len();
        if let Some(slot) = compressed.get_mut(len - 4..) {
            slot.copy_from_slice(&9999_u32.to_le_bytes());
        }
        assert!(matches!(
            decompress(&compressed, 1024),
            Err(GzipError::LengthMismatch { expected: 9999, .. })
        ));
    }

    #[test]
    fn the_limit_is_enforced_during_inflation() {
        let big = vec![b'x'; 200_000];
        let compressed = compress(&big);
        assert_eq!(decompress(&compressed, 1024), Err(GzipError::TooLarge));
    }

    #[test]
    fn non_gzip_input_is_rejected() {
        assert_eq!(decompress(b"", 1024), Err(GzipError::Malformed));
        assert_eq!(
            decompress(b"not gzip at all", 1024),
            Err(GzipError::Malformed)
        );
        // Right magic, wrong method.
        let mut wrong = vec![0x1F, 0x8B, 99, 0];
        wrong.extend_from_slice(&[0; 14]);
        assert_eq!(
            decompress(&wrong, 1024),
            Err(GzipError::UnsupportedMethod(99))
        );
    }

    #[test]
    fn truncation_at_every_length_is_an_error_not_a_panic() {
        let compressed = compress(b"some payload worth truncating");
        for cut in 0..compressed.len() {
            let prefix = compressed.get(..cut).expect("in range");
            assert!(
                decompress(prefix, 4096).is_err(),
                "a {cut}-byte prefix decompressed"
            );
        }
    }
}
