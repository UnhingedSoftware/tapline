//! The SteamPipe CDN.
//!
//! Two jobs: keep a pool of hosts worth talking to, and turn a chunk id into
//! verified plaintext.
//!
//! # The chunk pipeline
//!
//! ```text
//! GET /depot/{id}/chunk/{sha1}   →  AES-256 decrypt  →  VZ decode  →  SHA-1 check
//!                                   (depot key)         (LZMA)        (== chunk id)
//! ```
//!
//! That last step is the one that matters, and it is why
//! [`fetch_chunk`] returns bytes only when it passes. A chunk's id *is* the
//! SHA-1 of its plaintext, so a chunk that hashes to something else is not the
//! chunk the manifest named — whatever served it, and however well-formed the
//! container was. This is not a defence against an exotic attack: hosting
//! fleets deliberately put a caching proxy in this path, and a proxy that
//! returns the wrong object is an ordinary operational failure.
//!
//! # Rate limits
//!
//! Steam rate-limits per host, and a download that hammers one is a download
//! that gets an account or an IP throttled. The pool exists to spread requests
//! and to stop asking a host that has started refusing — being fast must never
//! look like abuse, since a locked account is worse than a slow download.

mod pool;

pub use pool::{Host, HostPool, PoolError};

use std::fmt;
use tapline_ids::DepotId;
use tapline_io::{Fetch, Request};
use tapline_manifest::{Chunk, Manifest};

/// What went wrong fetching content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CdnError {
    /// The fetch itself failed.
    Fetch(String),
    /// The host answered with a status that is not 2xx.
    Status {
        /// The code.
        code: u16,
        /// The host that sent it.
        host: String,
    },
    /// The chunk did not decrypt with the depot key.
    Decrypt,
    /// The container did not decode.
    Container(String),
    /// The plaintext did not hash to the id the manifest named.
    ///
    /// The single most important error in this crate. It means the bytes on the
    /// wire were not the bytes that were asked for, and the only safe response
    /// is to discard them and try a different host.
    IntegrityFailure {
        /// The id the manifest named.
        expected: String,
        /// What the bytes actually hash to.
        actual: String,
        /// Which host served them.
        host: String,
    },
    /// The response length disagreed with the manifest.
    ///
    /// Caught before decryption, so a host that streams an unbounded body is cut
    /// off rather than followed.
    WrongLength {
        /// What the manifest said.
        expected: u32,
        /// What arrived.
        actual: usize,
    },
    /// Every host in the pool has been exhausted.
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

/// Fetches one chunk and returns its verified plaintext.
///
/// The `Result` is the contract: bytes come back only when they hash to the id
/// the manifest named.
///
/// Convenient, and does the decode inline. A downloader running many of these
/// at once should use [`fetch_chunk_bytes`] and schedule [`decode_chunk`] as
/// blocking work instead — decrypting, decompressing and hashing a megabyte is
/// CPU, and doing it on an async worker stalls every task sharing that thread.
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

/// Fetches a chunk's stored bytes, without decoding them.
///
/// The IO half. Still checks the length against the manifest, so an over-long
/// response is cut off rather than followed.
pub async fn fetch_chunk_bytes<F: Fetch>(
    fetcher: &F,
    host: &str,
    depot: DepotId,
    chunk: &Chunk,
) -> Result<Vec<u8>, CdnError> {
    let id = chunk.id_hex();
    let url = format!("https://{host}/depot/{depot}/chunk/{id}");

    // The manifest says how large the stored chunk is, so an over-long response
    // is refused as it arrives rather than after it has all been read.
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

/// Decrypts, decompresses and verifies a chunk's stored bytes.
///
/// Split out from the fetch so it can be tested against captured chunks with no
/// network at all.
pub fn decode_chunk(
    stored: &[u8],
    chunk: &Chunk,
    depot_key: &[u8; 32],
    host: &str,
) -> Result<Vec<u8>, CdnError> {
    let container =
        tapline_crypto::decrypt_content(depot_key, stored).map_err(|_| CdnError::Decrypt)?;

    let plaintext =
        tapline_chunk::decode(&container).map_err(|e| CdnError::Container(e.to_string()))?;

    // The check the whole pipeline exists to reach.
    let digest = tapline_crypto::sha1(&plaintext);
    if digest != chunk.id {
        return Err(CdnError::IntegrityFailure {
            expected: chunk.id_hex(),
            actual: digest.iter().map(|b| format!("{b:02x}")).collect(),
            host: host.to_owned(),
        });
    }

    // The manifest's own size, checked too — a chunk that verifies but is the
    // wrong length would corrupt the file it lands in.
    if plaintext.len() != chunk.uncompressed_size as usize {
        return Err(CdnError::WrongLength {
            expected: chunk.uncompressed_size,
            actual: plaintext.len(),
        });
    }

    Ok(plaintext)
}

/// Fetches and parses a depot manifest.
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

    // A cache serving the wrong manifest would otherwise produce a confidently
    // wrong install: right depot, wrong build, every chunk id unfamiliar.
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

    /// A real `VZ` container from depot 232257, captured 2026-08-26 — the same
    /// bytes Steam served, after decryption.
    ///
    /// The *encrypted* form is rebuilt here under a test key rather than
    /// committed. Steam's own depot key would work and is granted to any
    /// anonymous session, but shipping one in a repository is precisely the
    /// `keys.txt` habit this project lists as a non-goal, and the pipeline is
    /// no less exercised for the key being ours: decrypt, decompress and
    /// hash-check all run over content Steam really produced, and the SHA-1 the
    /// check compares against is the real chunk id.
    const CONTAINER: &[u8] =
        include_bytes!("../tests/fixtures/chunk_610f4c4e6d26a61f0a35ed66117a7e693cceb4b8.bin");

    /// The chunk id: the SHA-1 of the plaintext inside `CONTAINER`.
    const REAL_ID: [u8; 20] = [
        0x61, 0x0f, 0x4c, 0x4e, 0x6d, 0x26, 0xa6, 0x1f, 0x0a, 0x35, 0xed, 0x66, 0x11, 0x7a, 0x7e,
        0x69, 0x3c, 0xce, 0xb4, 0xb8,
    ];

    /// A key of our own, standing in for the one Steam grants.
    const TEST_KEY: [u8; 32] = [0x5A; 32];

    /// The container encrypted the way the CDN stores it.
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
        // Encrypted bytes in, verified plaintext out — decrypt, decompress and
        // hash-check, against a chunk Steam actually served.
        let plaintext = decode_chunk(&stored(), &real_chunk(), &TEST_KEY, "test")
            .expect("a real chunk must decode");

        assert_eq!(plaintext.len(), 333);
        assert!(plaintext.starts_with(b"whitelist"));
        assert_eq!(tapline_crypto::sha1(&plaintext), REAL_ID);
    }

    #[test]
    fn a_chunk_that_hashes_wrong_is_refused_even_though_it_decoded() {
        // The container is intact and decrypts fine; only the id disagrees.
        // This is exactly the shape of a cache serving the wrong object, and
        // the bytes must not be returned.
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
        // Flip a bit anywhere in the stored bytes. Decryption, decompression or
        // the hash check must catch it — but never silence.
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
        // Verifies, but is not the length the manifest promised — it would
        // corrupt the file it lands in.
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
