//! Steam's symmetric message encryption: `ECB(key, iv) || CBC(key, iv, plaintext)`.

use crate::{constant_time_eq, hmac_sha1};
use aes::Aes256;
use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{
    BlockDecrypt, BlockDecryptMut, BlockEncrypt, BlockEncryptMut, BlockSizeUser, KeyInit,
    KeyIvInit, generic_array::GenericArray,
};
use rand_core::{OsRng, RngCore};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;

const BLOCK: usize = 16;
const TAG_LEN: usize = 13;
const NONCE_LEN: usize = BLOCK - TAG_LEN;
const HMAC_KEY_LEN: usize = 16;

/// What went wrong encrypting or decrypting a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    /// The ciphertext was shorter than an IV block, or not block-aligned.
    MalformedCiphertext,
    /// Bad PKCS#7 padding; not distinguished further, to avoid a padding oracle.
    DecryptionFailed,
    /// The message decrypted but its HMAC tag did not match.
    AuthenticationFailed,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedCiphertext => f.write_str("ciphertext is not a whole number of blocks"),
            Self::DecryptionFailed => f.write_str("decryption failed"),
            Self::AuthenticationFailed => f.write_str("message authentication failed"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// The 32-byte AES session key; zeroed on drop, never printed.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SessionKey([u8; 32]);

impl SessionKey {
    /// Wraps 32 key bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Generates a fresh key from the operating system's RNG.
    #[must_use]
    pub fn generate() -> Self {
        let mut bytes = [0_u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// The raw key bytes, for handing to a cipher.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Steam keys the HMAC with the first half of the session key.
    fn hmac_key(&self) -> &[u8] {
        self.0.get(..HMAC_KEY_LEN).unwrap_or(&self.0)
    }
}

impl fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionKey(<redacted>)")
    }
}

/// ECB is safe here: the input is exactly one block.
fn ecb_encrypt_block(key: &SessionKey, block: [u8; BLOCK]) -> [u8; BLOCK] {
    let cipher = Aes256::new(GenericArray::from_slice(key.as_bytes()));
    let mut block = GenericArray::from(block);
    cipher.encrypt_block(&mut block);
    block.into()
}

fn ecb_decrypt_block(key: &SessionKey, block: [u8; BLOCK]) -> [u8; BLOCK] {
    let cipher = Aes256::new(GenericArray::from_slice(key.as_bytes()));
    let mut block = GenericArray::from(block);
    cipher.decrypt_block(&mut block);
    block.into()
}

/// Steam's IV doubles as a tag: `HMAC-SHA1(key[..16], nonce || plaintext)[..13] || nonce`.
fn derive_iv(key: &SessionKey, nonce: [u8; NONCE_LEN], plaintext: &[u8]) -> [u8; BLOCK] {
    let mut hmac_input = Vec::with_capacity(NONCE_LEN + plaintext.len());
    hmac_input.extend_from_slice(&nonce);
    hmac_input.extend_from_slice(plaintext);

    let tag = hmac_sha1(key.hmac_key(), &hmac_input);

    let mut iv = [0_u8; BLOCK];
    // Slices are in-bounds constants; the fallbacks keep the no-panic rule.
    if let (Some(tag_part), Some(tag_src)) = (iv.get_mut(..TAG_LEN), tag.get(..TAG_LEN)) {
        tag_part.copy_from_slice(tag_src);
    }
    if let Some(nonce_part) = iv.get_mut(TAG_LEN..) {
        nonce_part.copy_from_slice(&nonce);
    }
    iv
}

/// Encrypts a message for the CM channel; the derived IV authenticates it.
pub fn encrypt_message(key: &SessionKey, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut nonce = [0_u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);

    let iv = derive_iv(key, nonce, plaintext);
    let encrypted_iv = ecb_encrypt_block(key, iv);

    let cipher = Aes256CbcEnc::new(
        GenericArray::from_slice(key.as_bytes()),
        GenericArray::from_slice(&iv),
    );

    let padded_len = plaintext.len() + BLOCK - (plaintext.len() % BLOCK);
    let mut out = vec![0_u8; BLOCK + padded_len];

    let iv_slot = out.get_mut(..BLOCK).ok_or(CryptoError::DecryptionFailed)?;
    iv_slot.copy_from_slice(&encrypted_iv);

    let body = out.get_mut(BLOCK..).ok_or(CryptoError::DecryptionFailed)?;
    cipher
        .encrypt_padded_b2b_mut::<Pkcs7>(plaintext, body)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    Ok(out)
}

/// Decrypts and authenticates a message from the CM channel.
pub fn decrypt_message(key: &SessionKey, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let encrypted_iv: [u8; BLOCK] = ciphertext
        .get(..BLOCK)
        .and_then(|s| s.try_into().ok())
        .ok_or(CryptoError::MalformedCiphertext)?;
    let body = ciphertext
        .get(BLOCK..)
        .ok_or(CryptoError::MalformedCiphertext)?;
    if body.is_empty() || body.len() % Aes256::block_size() != 0 {
        return Err(CryptoError::MalformedCiphertext);
    }

    let iv = ecb_decrypt_block(key, encrypted_iv);

    let cipher = Aes256CbcDec::new(
        GenericArray::from_slice(key.as_bytes()),
        GenericArray::from_slice(&iv),
    );
    let mut buf = body.to_vec();
    let plaintext = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|_| CryptoError::DecryptionFailed)?
        .to_vec();

    let nonce: [u8; NONCE_LEN] = iv
        .get(TAG_LEN..)
        .and_then(|s| s.try_into().ok())
        .ok_or(CryptoError::MalformedCiphertext)?;
    let expected = derive_iv(key, nonce, &plaintext);

    if !constant_time_eq(&expected, &iv) {
        return Err(CryptoError::AuthenticationFailed);
    }
    Ok(plaintext)
}

/// Decrypts depot content, whose IV is random rather than HMAC-derived.
pub fn decrypt_content(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let wrapped = SessionKey::from_bytes(*key);

    let encrypted_iv: [u8; BLOCK] = ciphertext
        .get(..BLOCK)
        .and_then(|s| s.try_into().ok())
        .ok_or(CryptoError::MalformedCiphertext)?;
    let body = ciphertext
        .get(BLOCK..)
        .ok_or(CryptoError::MalformedCiphertext)?;
    if body.is_empty() || body.len() % BLOCK != 0 {
        return Err(CryptoError::MalformedCiphertext);
    }

    let iv = ecb_decrypt_block(&wrapped, encrypted_iv);

    let cipher = Aes256CbcDec::new(GenericArray::from_slice(key), GenericArray::from_slice(&iv));
    let mut buf = body.to_vec();
    let len = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|_| CryptoError::DecryptionFailed)?
        .len();
    buf.truncate(len);
    Ok(buf)
}

/// Like [`decrypt_content`], but decrypts in place and allocates nothing.
pub fn decrypt_content_owned(
    key: &[u8; 32],
    mut ciphertext: Vec<u8>,
) -> Result<Vec<u8>, CryptoError> {
    let wrapped = SessionKey::from_bytes(*key);

    let encrypted_iv: [u8; BLOCK] = ciphertext
        .get(..BLOCK)
        .and_then(|s| s.try_into().ok())
        .ok_or(CryptoError::MalformedCiphertext)?;
    let body_len = ciphertext.len().saturating_sub(BLOCK);
    if body_len == 0 || body_len % BLOCK != 0 {
        return Err(CryptoError::MalformedCiphertext);
    }

    let iv = ecb_decrypt_block(&wrapped, encrypted_iv);
    let cipher = Aes256CbcDec::new(GenericArray::from_slice(key), GenericArray::from_slice(&iv));

    let body = ciphertext
        .get_mut(BLOCK..)
        .ok_or(CryptoError::MalformedCiphertext)?;
    let len = cipher
        .decrypt_padded_mut::<Pkcs7>(body)
        .map_err(|_| CryptoError::DecryptionFailed)?
        .len();

    ciphertext.copy_within(BLOCK..BLOCK + len, 0);
    ciphertext.truncate(len);
    Ok(ciphertext)
}

/// Encrypts content with a plain IV; only used to build test fixtures.
pub fn encrypt_content(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let wrapped = SessionKey::from_bytes(*key);
    let iv = crate::random_bytes::<BLOCK>();
    let encrypted_iv = ecb_encrypt_block(&wrapped, iv);

    let cipher = Aes256CbcEnc::new(GenericArray::from_slice(key), GenericArray::from_slice(&iv));

    let padded_len = plaintext.len() + BLOCK - (plaintext.len() % BLOCK);
    let mut out = vec![0_u8; BLOCK + padded_len];

    let iv_slot = out.get_mut(..BLOCK).ok_or(CryptoError::DecryptionFailed)?;
    iv_slot.copy_from_slice(&encrypted_iv);

    let body = out.get_mut(BLOCK..).ok_or(CryptoError::DecryptionFailed)?;
    cipher
        .encrypt_padded_b2b_mut::<Pkcs7>(plaintext, body)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> SessionKey {
        SessionKey::from_bytes([0x42; 32])
    }

    #[test]
    fn content_encryption_round_trips_with_a_plain_iv() {
        let key = [0x37; 32];
        for plaintext in [b"".as_slice(), b"models/player/heavy.mdl", &[0xAB; 1024]] {
            let ciphertext = encrypt_content(&key, plaintext).expect("must encrypt");
            assert_eq!(
                decrypt_content(&key, &ciphertext).expect("must decrypt"),
                plaintext
            );
        }
    }

    #[test]
    fn content_decryption_does_not_expect_an_hmac_iv() {
        let key = [0x37; 32];
        let ciphertext = encrypt_content(&key, b"models/player/heavy.mdl").expect("encrypt");
        assert_eq!(
            decrypt_message(&SessionKey::from_bytes(key), &ciphertext),
            Err(CryptoError::AuthenticationFailed)
        );
    }

    #[test]
    fn the_wrong_depot_key_does_not_yield_a_filename() {
        let ciphertext = encrypt_content(&[0x37; 32], b"bin/srcds_linux").expect("encrypt");
        // Bad padding usually errors; when it does not, the plaintext must still differ.
        match decrypt_content(&[0x38; 32], &ciphertext) {
            Err(_) => {}
            Ok(plaintext) => assert_ne!(plaintext, b"bin/srcds_linux"),
        }
    }

    #[test]
    fn messages_round_trip() {
        let key = test_key();
        for len in [0_usize, 1, 15, 16, 17, 1024] {
            let plaintext = vec![0x5A; len];
            let ciphertext = encrypt_message(&key, &plaintext).expect("must encrypt");
            assert_eq!(
                decrypt_message(&key, &ciphertext),
                Ok(plaintext),
                "length {len}"
            );
        }
    }

    #[test]
    fn ciphertext_carries_the_encrypted_iv_in_front() {
        let key = test_key();
        let ciphertext = encrypt_message(&key, b"hello").expect("must encrypt");
        // One IV block plus one padded block.
        assert_eq!(ciphertext.len(), BLOCK * 2);
    }

    #[test]
    fn the_same_plaintext_encrypts_differently_each_time() {
        let key = test_key();
        let a = encrypt_message(&key, b"same message").expect("must encrypt");
        let b = encrypt_message(&key, b"same message").expect("must encrypt");
        assert_ne!(a, b);
    }

    #[test]
    fn a_tampered_body_fails_authentication() {
        let key = test_key();
        let mut ciphertext = encrypt_message(&key, b"transfer 10 credits").expect("must encrypt");

        // CBC still "decrypts" a flipped bit; the HMAC check is the gate.
        if let Some(byte) = ciphertext.get_mut(BLOCK + 3) {
            *byte ^= 0x01;
        }

        let result = decrypt_message(&key, &ciphertext);
        assert!(
            matches!(
                result,
                Err(CryptoError::AuthenticationFailed) | Err(CryptoError::DecryptionFailed)
            ),
            "tampered ciphertext was accepted: {result:?}"
        );
    }

    #[test]
    fn a_tampered_iv_fails_authentication() {
        let key = test_key();
        let mut ciphertext = encrypt_message(&key, b"hello world").expect("must encrypt");
        if let Some(byte) = ciphertext.get_mut(0) {
            *byte ^= 0x80;
        }
        assert!(matches!(
            decrypt_message(&key, &ciphertext),
            Err(CryptoError::AuthenticationFailed) | Err(CryptoError::DecryptionFailed)
        ));
    }

    #[test]
    fn the_wrong_key_never_yields_a_plaintext() {
        let ciphertext = encrypt_message(&test_key(), b"secret").expect("must encrypt");
        let wrong = SessionKey::from_bytes([0x43; 32]);
        assert!(decrypt_message(&wrong, &ciphertext).is_err());
    }

    #[test]
    fn truncated_and_ragged_ciphertexts_are_rejected() {
        let key = test_key();
        assert_eq!(
            decrypt_message(&key, &[]),
            Err(CryptoError::MalformedCiphertext)
        );
        assert_eq!(
            decrypt_message(&key, &[0; BLOCK]),
            Err(CryptoError::MalformedCiphertext)
        );
        // A body that is not a whole number of blocks.
        assert_eq!(
            decrypt_message(&key, &[0; BLOCK + 5]),
            Err(CryptoError::MalformedCiphertext)
        );
    }

    #[test]
    fn session_keys_do_not_print_themselves() {
        let key = SessionKey::generate();
        assert_eq!(format!("{key:?}"), "SessionKey(<redacted>)");
    }

    #[test]
    fn generated_keys_differ() {
        assert_ne!(
            SessionKey::generate().as_bytes(),
            SessionKey::generate().as_bytes()
        );
    }

    #[test]
    fn decrypting_in_place_matches_decrypting_by_reference() {
        let key = [7_u8; 32];
        for len in [1_usize, 15, 16, 17, 255, 1024] {
            let plaintext: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
            let encrypted = encrypt_content(&key, &plaintext).expect("encrypt");

            let by_reference = decrypt_content(&key, &encrypted).expect("by reference");
            let owned = decrypt_content_owned(&key, encrypted.clone()).expect("owned");

            assert_eq!(by_reference, plaintext, "reference form wrong at {len}");
            assert_eq!(owned, plaintext, "owned form wrong at {len}");
        }
    }

    #[test]
    fn the_owned_form_refuses_the_same_malformed_input() {
        let key = [1_u8; 32];
        assert!(decrypt_content_owned(&key, Vec::new()).is_err());
        assert!(decrypt_content_owned(&key, vec![0; BLOCK]).is_err());
        // Not a whole number of blocks after the IV.
        assert!(decrypt_content_owned(&key, vec![0; BLOCK + 3]).is_err());
    }
}
