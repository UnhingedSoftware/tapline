mod pool;

pub use pool::{Host, HostPool, PoolError, usable_over_tls};

use std::fmt;
use tapline_ids::DepotId;
use tapline_io::{Fetch, Request};
use tapline_manifest::{Chunk, Manifest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CdnError {
    Fetch(String),
    Status {
        code: u16,
        host: String,
    },
    Decrypt,
    Container(String),
    IntegrityFailure {
        expected: String,
        actual: String,
        host: String,
    },
    WrongLength {
        expected: u32,
        actual: usize,
    },
    NoHostsLeft,
}

impl fmt::Display for CdnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fetch(message) => write!(f, "fetch failed: {message}"),
            Self::Status { code, host } => write!(f, "{host} answered {code}"),
            Self::Decrypt => f.write_str("the chunk did not decrypt with the depot key"),
            Self::Container(message) => write!(f, "chunk container: {message}"),
            Self::IntegrityFailure {
                expected,
                actual,
                host,
            } => write!(
                f,
                "{host} served a chunk hashing to {actual}, but the manifest named {expected}"
            ),
            Self::WrongLength { expected, actual } => {
                write!(f, "expected a {expected}-byte chunk, received {actual}")
            }
            Self::NoHostsLeft => f.write_str("no CDN host is still usable"),
        }
    }
}

impl std::error::Error for CdnError {}

pub async fn fetch_chunk<F: Fetch>(
    fetcher: &F,
    host: &str,
    depot: DepotId,
    chunk: &Chunk,
    depot_key: &[u8; 32],
) -> Result<Vec<u8>, CdnError> {
    let stored = fetch_chunk_bytes(fetcher, host, depot, chunk).await?;
    decode_chunk(&stored, chunk, depot_key, host)
}

pub async fn fetch_chunk_bytes<F: Fetch>(
    fetcher: &F,
    host: &str,
    depot: DepotId,
    chunk: &Chunk,
) -> Result<Vec<u8>, CdnError> {
    let id = chunk.id_hex();
    let url = format!("https://{host}/depot/{depot}/chunk/{id}");

    let limit = u64::from(chunk.compressed_size).max(1) + 4096;
    let response = fetcher
        .get(Request::get(url), limit)
        .await
        .map_err(|error| CdnError::Fetch(error.to_string()))?;

    if !response.is_success() {
        return Err(CdnError::Status {
            code: response.status,
            host: host.to_owned(),
        });
    }
    if response.body.len() != chunk.compressed_size as usize {
        return Err(CdnError::WrongLength {
            expected: chunk.compressed_size,
            actual: response.body.len(),
        });
    }

    Ok(response.body)
}

pub fn decode_chunk(
    stored: &[u8],
    chunk: &Chunk,
    depot_key: &[u8; 32],
    host: &str,
) -> Result<Vec<u8>, CdnError> {
    decode_chunk_owned(stored.to_vec(), chunk, depot_key, host)
}

pub fn decode_chunk_owned(
    stored: Vec<u8>,
    chunk: &Chunk,
    depot_key: &[u8; 32],
    host: &str,
) -> Result<Vec<u8>, CdnError> {
    let mut out = Vec::new();
    decode_chunk_into(stored, chunk, depot_key, host, &mut out)?;
    Ok(out)
}

pub fn decode_chunk_into(
    stored: Vec<u8>,
    chunk: &Chunk,
    depot_key: &[u8; 32],
    host: &str,
    plaintext: &mut Vec<u8>,
) -> Result<(), CdnError> {
    let container =
        tapline_crypto::decrypt_content_owned(depot_key, stored).map_err(|_| CdnError::Decrypt)?;

    tapline_chunk::decode_into(&container, tapline_chunk::MAX_CHUNK, plaintext)
        .map_err(|e| CdnError::Container(e.to_string()))?;
    let plaintext = &*plaintext;

    let digest = tapline_crypto::sha1(plaintext);
    if digest != chunk.id {
        return Err(CdnError::IntegrityFailure {
            expected: chunk.id_hex(),
            actual: digest.iter().map(|b| format!("{b:02x}")).collect(),
            host: host.to_owned(),
        });
    }

    if plaintext.len() != chunk.uncompressed_size as usize {
        return Err(CdnError::WrongLength {
            expected: chunk.uncompressed_size,
            actual: plaintext.len(),
        });
    }

    Ok(())
}

pub async fn fetch_manifest<F: Fetch>(
    fetcher: &F,
    host: &str,
    depot: DepotId,
    manifest_id: u64,
    request_code: u64,
    depot_key: Option<&[u8; 32]>,
) -> Result<Manifest, CdnError> {
    let url = format!("https://{host}/depot/{depot}/manifest/{manifest_id}/5/{request_code}");

    let response = fetcher
        .get(Request::get(url), tapline_manifest::MAX_MANIFEST as u64)
        .await
        .map_err(|error| CdnError::Fetch(error.to_string()))?;

    if !response.is_success() {
        return Err(CdnError::Status {
            code: response.status,
            host: host.to_owned(),
        });
    }

    let manifest = Manifest::parse(&response.body, depot_key)
        .map_err(|e| CdnError::Container(e.to_string()))?;

    if manifest.id.get() != manifest_id {
        return Err(CdnError::Container(format!(
            "asked for manifest {manifest_id}, received {}",
            manifest.id
        )));
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTAINER: &[u8] =
        include_bytes!("../tests/fixtures/chunk_610f4c4e6d26a61f0a35ed66117a7e693cceb4b8.bin");

    const REAL_ID: [u8; 20] = [
        0x61, 0x0f, 0x4c, 0x4e, 0x6d, 0x26, 0xa6, 0x1f, 0x0a, 0x35, 0xed, 0x66, 0x11, 0x7a, 0x7e,
        0x69, 0x3c, 0xce, 0xb4, 0xb8,
    ];

    const TEST_KEY: [u8; 32] = [0x5A; 32];

    fn stored() -> Vec<u8> {
        tapline_crypto::encrypt_content(&TEST_KEY, CONTAINER).expect("must encrypt")
    }

    fn real_chunk() -> Chunk {
        Chunk {
            id: REAL_ID,
            crc: 0x9bea_5cdc,
            offset: 0,
            uncompressed_size: 333,
            compressed_size: 0,
        }
    }

    #[test]
    fn the_whole_pipeline_produces_the_file_it_should() {
        let plaintext = decode_chunk(&stored(), &real_chunk(), &TEST_KEY, "test")
            .expect("a real chunk must decode");

        assert_eq!(plaintext.len(), 333);
        assert!(plaintext.starts_with(b"whitelist"));
        assert_eq!(tapline_crypto::sha1(&plaintext), REAL_ID);
    }

    #[test]
    fn a_chunk_that_hashes_wrong_is_refused_even_though_it_decoded() {
        let mut wrong = real_chunk();
        wrong.id = [0xAA; 20];

        let error = decode_chunk(&stored(), &wrong, &TEST_KEY, "cache1.invalid")
            .expect_err("a mismatched hash must not be accepted");

        match error {
            CdnError::IntegrityFailure { host, actual, .. } => {
                assert_eq!(host, "cache1.invalid", "the failing host must be named");
                assert_eq!(actual, "610f4c4e6d26a61f0a35ed66117a7e693cceb4b8");
            }
            other => panic!("expected an integrity failure, got {other}"),
        }
    }

    #[test]
    fn a_tampered_chunk_never_reaches_the_caller() {
        let encrypted = stored();
        for position in [0_usize, 16, 40, encrypted.len() - 1] {
            let mut damaged = stored();
            if let Some(byte) = damaged.get_mut(position) {
                *byte ^= 0x01;
            }
            assert!(
                decode_chunk(&damaged, &real_chunk(), &TEST_KEY, "test").is_err(),
                "a chunk damaged at byte {position} was accepted"
            );
        }
    }

    #[test]
    fn the_wrong_depot_key_does_not_yield_content() {
        assert!(
            decode_chunk(&stored(), &real_chunk(), &[0x00; 32], "test").is_err(),
            "the wrong key produced content"
        );
    }

    #[test]
    fn a_chunk_of_the_wrong_length_is_refused() {
        let mut wrong = real_chunk();
        wrong.uncompressed_size = 999;

        assert!(matches!(
            decode_chunk(&stored(), &wrong, &TEST_KEY, "test"),
            Err(CdnError::WrongLength { .. })
        ));
    }

    #[test]
    fn chunk_ids_render_as_the_cdn_spells_them() {
        assert_eq!(
            real_chunk().id_hex(),
            "610f4c4e6d26a61f0a35ed66117a7e693cceb4b8"
        );
    }
}
