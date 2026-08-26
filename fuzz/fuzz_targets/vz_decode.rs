//! The chunk container decoder must survive arbitrary bytes.
//!
//! Chunks arrive from a CDN, and hosting fleets deliberately put a caching
//! proxy in that path — so "the bytes came from Steam" is not something the
//! decoder gets to assume. Anything it returns must also match the container's
//! own checksum, which the decoder is responsible for enforcing.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    match tapline_lzma::decode(data) {
        Err(_) => {}
        Ok(plaintext) => {
            // If it decoded, the container's CRC must have matched — that is
            // the decoder's contract, and returning unverified bytes would let
            // a poisoned cache through.
            let footer = data.len().saturating_sub(10);
            if let Some(slice) = data.get(footer..footer + 4) {
                if let Ok(bytes) = <[u8; 4]>::try_from(slice) {
                    assert_eq!(
                        crc32fast::hash(&plaintext),
                        u32::from_le_bytes(bytes),
                        "decode returned bytes that fail the container's own CRC"
                    );
                }
            }
            assert!(
                plaintext.len() <= tapline_lzma::MAX_CHUNK,
                "decode exceeded its own size cap"
            );
        }
    }
});
