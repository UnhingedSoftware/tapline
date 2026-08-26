//! Encrypting a password for `BeginAuthSessionViaCredentials`.
//!
//! Steam hands out a **per-account RSA public key** for each login attempt, as a
//! hex modulus and hex exponent, with a timestamp that must be echoed back. The
//! password is encrypted under it with PKCS#1 v1.5 and base64-encoded.
//!
//! Two things worth stating plainly.
//!
//! **There is no hardcoded Valve key here, and there is none anywhere in this
//! workspace.** The key arrives from Steam for this login and is used once.
//!
//! **The plaintext is zeroed as soon as it has been encrypted.** That is not a
//! guarantee against a determined attacker with memory access, and it is not
//! meant to be — it shortens the window in which a password sits in a heap
//! allocation that might be swapped, cored, or read by a later bug.

use rsa::{BigUint, Pkcs1v15Encrypt, RsaPublicKey};
use std::fmt;
use zeroize::Zeroize;

/// The RSA key Steam issued for one login.
#[derive(Clone, PartialEq, Eq)]
pub struct PublicKey {
    key: RsaPublicKey,
    /// Steam's timestamp, which must be echoed in the login request.
    ///
    /// A login that sends a password encrypted under one key and the timestamp
    /// of another is rejected, which is the point: it binds the ciphertext to
    /// the key exchange.
    pub timestamp: u64,
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // A public key is not secret, but printing a 2048-bit modulus in a log
        // line helps nobody.
        f.debug_struct("PublicKey")
            .field("timestamp", &self.timestamp)
            .finish_non_exhaustive()
    }
}

/// What went wrong encrypting a password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordError {
    /// The modulus or exponent was not valid hex.
    MalformedKey(String),
    /// The key Steam sent was too small to be a real RSA key.
    ///
    /// Refused rather than used: a login that encrypts a password under a
    /// 64-bit modulus has not protected it.
    KeyTooSmall {
        /// The modulus size in bits.
        bits: usize,
    },
    /// RSA rejected the input, almost always because the password is longer
    /// than the key can hold.
    Encryption(String),
}

impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedKey(what) => write!(f, "Steam sent a malformed RSA key: {what}"),
            Self::KeyTooSmall { bits } => {
                write!(
                    f,
                    "Steam sent a {bits}-bit RSA key, which is too small to trust"
                )
            }
            Self::Encryption(message) => write!(f, "could not encrypt the password: {message}"),
        }
    }
}

impl std::error::Error for PasswordError {}

/// The smallest modulus we will encrypt under.
///
/// Steam uses 2048. Anything markedly smaller means either a broken response or
/// something interposing on the connection, and encrypting a password under it
/// would be worse than failing to log in.
const MIN_KEY_BITS: usize = 1024;

impl PublicKey {
    /// Parses the hex modulus and exponent Steam returns.
    pub fn from_hex(modulus: &str, exponent: &str, timestamp: u64) -> Result<Self, PasswordError> {
        let modulus_bytes = decode_hex(modulus)
            .ok_or_else(|| PasswordError::MalformedKey("modulus is not hex".to_owned()))?;
        let exponent_bytes = decode_hex(exponent)
            .ok_or_else(|| PasswordError::MalformedKey("exponent is not hex".to_owned()))?;

        let n = BigUint::from_bytes_be(&modulus_bytes);
        let e = BigUint::from_bytes_be(&exponent_bytes);

        let bits = n.bits();
        if bits < MIN_KEY_BITS {
            return Err(PasswordError::KeyTooSmall { bits });
        }

        let key = RsaPublicKey::new(n, e)
            .map_err(|error| PasswordError::MalformedKey(error.to_string()))?;

        Ok(Self { key, timestamp })
    }

    /// The modulus size, for a log line that says what was used.
    #[must_use]
    pub fn bits(&self) -> usize {
        rsa::traits::PublicKeyParts::n(&self.key).bits()
    }
}

/// Encrypts a password under Steam's key, returning the base64 Steam expects.
///
/// Takes the password by value and zeroes it before returning: the caller
/// cannot forget to, and there is no borrowed copy left behind.
pub fn encrypt_password(mut password: String, key: &PublicKey) -> Result<String, PasswordError> {
    let mut rng = rsa::rand_core::OsRng;
    let result = key
        .key
        .encrypt(&mut rng, Pkcs1v15Encrypt, password.as_bytes())
        .map_err(|error| PasswordError::Encryption(error.to_string()));

    // Before the `?`, so the plaintext is gone on the failure path too — which
    // is the path where something has already gone wrong and nobody is looking.
    password.zeroize();

    Ok(base64_encode(&result?))
}

/// Decodes a hex string.
fn decode_hex(input: &str) -> Option<Vec<u8>> {
    let trimmed = input.trim();
    // An odd-length hex string is malformed, not something to pad silently.
    if trimmed.is_empty() || trimmed.len() % 2 != 0 {
        return None;
    }

    let bytes = trimmed.as_bytes();
    let mut out = Vec::with_capacity(trimmed.len() / 2);
    for pair in bytes.chunks(2) {
        let high = hex_value(*pair.first()?)?;
        let low = hex_value(*pair.get(1)?)?;
        out.push((high << 4) | low);
    }
    Some(out)
}

const fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Standard base64, which is how Steam wants the ciphertext.
fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk.first().copied().unwrap_or(0);
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let triple = (u32::from(b0) << 16) | (u32::from(b1) << 8) | u32::from(b2);

        for (position, shift) in [18_u32, 12, 6, 0].into_iter().enumerate() {
            let pad_from = match chunk.len() {
                1 => 2,
                2 => 3,
                _ => 4,
            };
            if position >= pad_from {
                out.push('=');
            } else {
                let index = ((triple >> shift) & 0x3F) as usize;
                out.push(char::from(ALPHABET.get(index).copied().unwrap_or(b'A')));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2048-bit key generated once for these tests, in the hex shape Steam
    /// sends. Not Valve's, and not secret — it exists so the encryption path can
    /// be exercised without a login.
    fn test_key() -> PublicKey {
        use rsa::traits::PublicKeyParts;

        let mut rng = rsa::rand_core::OsRng;
        let private = rsa::RsaPrivateKey::new(&mut rng, 2048).expect("keygen");
        let public = private.to_public_key();

        let modulus = hex_of(&public.n().to_bytes_be());
        let exponent = hex_of(&public.e().to_bytes_be());
        PublicKey::from_hex(&modulus, &exponent, 1_700_000_000).expect("must parse")
    }

    fn hex_of(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn a_steam_shaped_key_parses() {
        let key = test_key();
        assert_eq!(key.bits(), 2048);
        assert_eq!(key.timestamp, 1_700_000_000);
    }

    #[test]
    fn a_password_encrypts_to_base64_that_is_not_the_password() {
        // The bar is low and worth asserting anyway: the ciphertext must not
        // contain the plaintext.
        let key = test_key();
        let encrypted = encrypt_password("hunter2".to_owned(), &key).expect("must encrypt");

        assert!(!encrypted.is_empty());
        assert!(!encrypted.contains("hunter2"));
        // 2048-bit RSA produces 256 bytes, which is 344 base64 characters.
        assert_eq!(encrypted.len(), 344);
        assert!(encrypted.ends_with('='), "expected base64 padding");
    }

    #[test]
    fn the_same_password_encrypts_differently_each_time() {
        // PKCS#1 v1.5 randomises its padding. Identical ciphertexts would mean
        // an observer could tell that two logins used the same password.
        let key = test_key();
        let first = encrypt_password("hunter2".to_owned(), &key).expect("encrypt");
        let second = encrypt_password("hunter2".to_owned(), &key).expect("encrypt");
        assert_ne!(first, second);
    }

    #[test]
    fn the_ciphertext_decrypts_back_to_the_password() {
        // The real check: a key we hold both halves of, so the round trip can be
        // verified rather than assumed.
        use rsa::traits::PublicKeyParts;

        let mut rng = rsa::rand_core::OsRng;
        let private = rsa::RsaPrivateKey::new(&mut rng, 2048).expect("keygen");
        let public = private.to_public_key();

        let key = PublicKey::from_hex(
            &hex_of(&public.n().to_bytes_be()),
            &hex_of(&public.e().to_bytes_be()),
            0,
        )
        .expect("parse");

        let encrypted =
            encrypt_password("correct horse battery".to_owned(), &key).expect("encrypt");
        let raw = base64_decode(&encrypted).expect("our own base64 must decode");
        let decrypted = private.decrypt(Pkcs1v15Encrypt, &raw).expect("decrypt");

        assert_eq!(decrypted, b"correct horse battery");
    }

    #[test]
    fn a_key_too_small_to_trust_is_refused() {
        // Encrypting a password under a 64-bit modulus has not protected it,
        // and a login that fails is better than one that pretends.
        let error = PublicKey::from_hex("ffffffffffffffff", "010001", 0)
            .expect_err("a tiny key must be refused");
        assert!(matches!(error, PasswordError::KeyTooSmall { .. }));
    }

    #[test]
    fn a_malformed_key_is_refused_rather_than_guessed_at() {
        assert!(PublicKey::from_hex("not hex", "010001", 0).is_err());
        assert!(PublicKey::from_hex("", "010001", 0).is_err());
        // Odd length is malformed, not something to pad.
        assert!(PublicKey::from_hex("abc", "010001", 0).is_err());
    }

    #[test]
    fn hex_decoding_accepts_both_cases_and_rejects_the_rest() {
        assert_eq!(decode_hex("00ff"), Some(vec![0x00, 0xFF]));
        assert_eq!(decode_hex("00FF"), Some(vec![0x00, 0xFF]));
        assert_eq!(decode_hex("0x00"), None);
        assert_eq!(decode_hex("zz"), None);
    }

    #[test]
    fn base64_matches_rfc_4648() {
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b""), "");
    }

    /// Decodes base64, for the round-trip test above.
    fn base64_decode(input: &str) -> Option<Vec<u8>> {
        fn value(byte: u8) -> Option<u8> {
            match byte {
                b'A'..=b'Z' => Some(byte - b'A'),
                b'a'..=b'z' => Some(byte - b'a' + 26),
                b'0'..=b'9' => Some(byte - b'0' + 52),
                b'+' => Some(62),
                b'/' => Some(63),
                _ => None,
            }
        }

        let mut out = Vec::new();
        let mut accumulator = 0_u32;
        let mut bits = 0_u32;
        for byte in input.bytes().take_while(|b| *b != b'=') {
            accumulator = (accumulator << 6) | u32::from(value(byte)?);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push(((accumulator >> bits) & 0xFF) as u8);
            }
        }
        Some(out)
    }
}
