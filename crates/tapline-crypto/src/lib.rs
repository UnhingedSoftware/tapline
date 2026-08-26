//! The cryptography Steam's protocol needs.
//!
//! Two kinds of thing live here. The primitives — AES, RSA, SHA-1, HMAC — are
//! thin wrappers over RustCrypto, present so the rest of the workspace has one
//! place to look and one place to change if a primitive is ever swapped. The
//! *compositions* are ours to get right: Steam layers ECB and CBC in a specific
//! order, and derives its CBC IV from an HMAC in a specific way, and neither is
//! something a general-purpose crypto crate offers.
//!
//! # What is deliberately not here
//!
//! Valve's universe public key. It is a constant, and a constant that must be
//! verified against a real handshake rather than typed from memory, so it lives
//! next to the handshake that uses it in `tapline-net`.

mod symmetric;

pub use symmetric::{CryptoError, SessionKey, decrypt_message, encrypt_message};

use hmac::{Hmac, Mac};
use sha1::{Digest, Sha1};
use sha2::Sha256;

/// SHA-1 of `data`.
///
/// SHA-1 is not a security boundary here and is not being used as one: Steam
/// identifies every content chunk by the SHA-1 of its plaintext, so this is
/// content addressing. The integrity guarantee comes from the id having been
/// named in a manifest that arrived over an authenticated channel.
#[must_use]
pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// SHA-256 of `data`.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// HMAC-SHA1 of `data` under `key`.
#[must_use]
pub fn hmac_sha1(key: &[u8], data: &[u8]) -> [u8; 20] {
    // `new_from_slice` only fails for key lengths HMAC cannot accommodate, and
    // HMAC accommodates every length by construction, so this cannot fail.
    let mut mac = <Hmac<Sha1> as Mac>::new_from_slice(key)
        .unwrap_or_else(|_| unreachable!("HMAC accepts keys of any length"));
    mac.update(data);
    mac.finalize().into_bytes().into()
}

/// Fills an array from the operating system's RNG.
///
/// Every random value in the workspace comes from here so there is one place to
/// look when asking "is this actually random". Used for session keys and for
/// WebSocket frame masks — the latter looks cosmetic but is not, since a
/// predictable mask defeats the cache-poisoning defence the mask exists for.
#[must_use]
pub fn random_bytes<const N: usize>() -> [u8; N] {
    use rand_core::{OsRng, RngCore};
    let mut bytes = [0_u8; N];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

/// Compares two byte strings in time independent of their contents.
///
/// Used for MAC comparison. A short-circuiting `==` would leak how many leading
/// bytes an attacker guessed correctly, which is enough to forge a MAC one byte
/// at a time.
#[must_use]
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    // The length is public — it is a protocol constant — so returning early on
    // a mismatch there leaks nothing. The contents are not.
    let mut diff = 0_u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_matches_the_known_answer() {
        // FIPS 180-1 test vector.
        let digest = sha1(b"abc");
        assert_eq!(
            digest,
            [
                0xA9, 0x99, 0x3E, 0x36, 0x47, 0x06, 0x81, 0x6A, 0xBA, 0x3E, 0x25, 0x71, 0x78, 0x50,
                0xC2, 0x6C, 0x9C, 0xD0, 0xD8, 0x9D
            ]
        );
    }

    #[test]
    fn sha256_matches_the_known_answer() {
        let digest = sha256(b"abc");
        assert_eq!(
            digest,
            [
                0xBA, 0x78, 0x16, 0xBF, 0x8F, 0x01, 0xCF, 0xEA, 0x41, 0x41, 0x40, 0xDE, 0x5D, 0xAE,
                0x22, 0x23, 0xB0, 0x03, 0x61, 0xA3, 0x96, 0x17, 0x7A, 0x9C, 0xB4, 0x10, 0xFF, 0x61,
                0xF2, 0x00, 0x15, 0xAD
            ]
        );
    }

    #[test]
    fn hmac_sha1_matches_rfc_2202() {
        // RFC 2202 test case 2.
        let mac = hmac_sha1(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(
            mac,
            [
                0xEF, 0xFC, 0xDF, 0x6A, 0xE5, 0xEB, 0x2F, 0xA2, 0xD2, 0x74, 0x16, 0xD5, 0xF1, 0x84,
                0xDF, 0x9C, 0x25, 0x9A, 0x7C, 0x79
            ]
        );
    }

    #[test]
    fn constant_time_eq_agrees_with_equality() {
        assert!(constant_time_eq(b"", b""));
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
        // The first byte differing must be no different from the last byte
        // differing, as far as the caller can tell.
        assert!(!constant_time_eq(b"xbc", b"abc"));
    }
}
