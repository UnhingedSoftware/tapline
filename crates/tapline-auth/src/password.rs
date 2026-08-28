use rsa::{BigUint, Pkcs1v15Encrypt, RsaPublicKey};
use std::fmt;
use zeroize::Zeroize;

#[derive(Clone, PartialEq, Eq)]
pub struct PublicKey {
    key: RsaPublicKey,
    pub timestamp: u64,
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PublicKey")
            .field("timestamp", &self.timestamp)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordError {
    MalformedKey(String),
    KeyTooSmall { bits: usize },
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

const MIN_KEY_BITS: usize = 1024;

impl PublicKey {
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

    #[must_use]
    pub fn bits(&self) -> usize {
        rsa::traits::PublicKeyParts::n(&self.key).bits()
    }
}

pub fn encrypt_password(mut password: String, key: &PublicKey) -> Result<String, PasswordError> {
    let mut rng = rsa::rand_core::OsRng;
    let result = key
        .key
        .encrypt(&mut rng, Pkcs1v15Encrypt, password.as_bytes())
        .map_err(|error| PasswordError::Encryption(error.to_string()));

    password.zeroize();

    Ok(base64_encode(&result?))
}

fn decode_hex(input: &str) -> Option<Vec<u8>> {
    let trimmed = input.trim();
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
        let key = test_key();
        let encrypted = encrypt_password("hunter2".to_owned(), &key).expect("must encrypt");

        assert!(!encrypted.is_empty());
        assert!(!encrypted.contains("hunter2"));
        assert_eq!(encrypted.len(), 344);
        assert!(encrypted.ends_with('='), "expected base64 padding");
    }

    #[test]
    fn the_same_password_encrypts_differently_each_time() {
        let key = test_key();
        let first = encrypt_password("hunter2".to_owned(), &key).expect("encrypt");
        let second = encrypt_password("hunter2".to_owned(), &key).expect("encrypt");
        assert_ne!(first, second);
    }

    #[test]
    fn the_ciphertext_decrypts_back_to_the_password() {
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
        let error = PublicKey::from_hex("ffffffffffffffff", "010001", 0)
            .expect_err("a tiny key must be refused");
        assert!(matches!(error, PasswordError::KeyTooSmall { .. }));
    }

    #[test]
    fn a_malformed_key_is_refused_rather_than_guessed_at() {
        assert!(PublicKey::from_hex("not hex", "010001", 0).is_err());
        assert!(PublicKey::from_hex("", "010001", 0).is_err());
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
