//! Steam's symmetric message encryption.
//!
//! Steam does not simply CBC-encrypt a message under the session key. It picks a
//! per-message IV, encrypts *the IV itself* with the same key in ECB mode, and
//! sends that in front of the CBC ciphertext:
//!
//! ```text
//!   ECB(key, iv) || CBC(key, iv, plaintext)
//!   \___________/    \____________________/
//!      16 bytes          PKCS#7 padded
//! ```
//!
//! Since Steam's own 2015-era change the IV is not random but derived, so that
//! it doubles as a message authentication tag:
//!
//! ```text
//!   random  = 3 random bytes
//!   tag     = HMAC-SHA1(key[0..16], random || plaintext)[0..13]
//!   iv      = tag || random
//! ```
//!
//! On receipt the tag is recomputed over the decrypted plaintext and compared in
//! constant time. That is what makes the channel authenticated rather than
//! merely encrypted, and it is why [`decrypt_message`] refuses to hand back a
//! plaintext whose tag does not match, even though the CBC decryption itself
//! succeeded.

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

/// The AES block size, and therefore the IV length.
const BLOCK: usize = 16;
/// How many bytes of the IV carry the HMAC tag.
const TAG_LEN: usize = 13;
/// How many bytes of the IV are random.
const NONCE_LEN: usize = BLOCK - TAG_LEN;
/// The HMAC is keyed with the first half of the session key.
const HMAC_KEY_LEN: usize = 16;

/// What went wrong encrypting or decrypting a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    /// The ciphertext was shorter than an IV block, or not a multiple of the
    /// block size.
    MalformedCiphertext,
    /// The PKCS#7 padding was not well formed.
    ///
    /// Reported without saying *how* it was malformed: distinguishing a bad pad
    /// from a bad MAC is the padding-oracle bug, and the caller has no use for
    /// the difference anyway.
    DecryptionFailed,
    /// The message decrypted but its HMAC tag did not match.
    ///
    /// The plaintext is discarded rather than returned. A caller that acted on
    /// unauthenticated plaintext would be trusting whoever is on the wire.
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

/// The 32-byte AES key negotiated during the CM handshake.
///
/// Zeroed on drop, and refuses to print itself. It is the secret that protects
/// the logon that follows, including the account password.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SessionKey([u8; 32]);

impl SessionKey {
    /// Wraps 32 key bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Generates a fresh key from the operating system's RNG.
    ///
    /// This is the key the client sends to Steam encrypted under Valve's public
    /// key, so its quality is the whole security of the channel.
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

    /// The HMAC key: the first half of the session key.
    fn hmac_key(&self) -> &[u8] {
        self.0.get(..HMAC_KEY_LEN).unwrap_or(&self.0)
    }
}

impl fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionKey(<redacted>)")
    }
}

/// Encrypts one IV block in ECB mode.
///
/// ECB is correct here and only here: the input is exactly one block, and the
/// weakness of ECB is that it repeats across blocks. There are no other blocks.
fn ecb_encrypt_block(key: &SessionKey, block: [u8; BLOCK]) -> [u8; BLOCK] {
    let cipher = Aes256::new(GenericArray::from_slice(key.as_bytes()));
    let mut block = GenericArray::from(block);
    cipher.encrypt_block(&mut block);
    block.into()
}

/// Decrypts one IV block in ECB mode.
fn ecb_decrypt_block(key: &SessionKey, block: [u8; BLOCK]) -> [u8; BLOCK] {
    let cipher = Aes256::new(GenericArray::from_slice(key.as_bytes()));
    let mut block = GenericArray::from(block);
    cipher.decrypt_block(&mut block);
    block.into()
}

/// Derives the authenticating IV for a plaintext.
fn derive_iv(key: &SessionKey, nonce: [u8; NONCE_LEN], plaintext: &[u8]) -> [u8; BLOCK] {
    let mut hmac_input = Vec::with_capacity(NONCE_LEN + plaintext.len());
    hmac_input.extend_from_slice(&nonce);
    hmac_input.extend_from_slice(plaintext);

    let tag = hmac_sha1(key.hmac_key(), &hmac_input);

    let mut iv = [0_u8; BLOCK];
    // Both slices are compile-time constants within a 16-byte array, so neither
    // `get_mut` can fail; the fallbacks keep the workspace's no-panic rule
    // without an `expect`.
    if let (Some(tag_part), Some(tag_src)) = (iv.get_mut(..TAG_LEN), tag.get(..TAG_LEN)) {
        tag_part.copy_from_slice(tag_src);
    }
    if let Some(nonce_part) = iv.get_mut(TAG_LEN..) {
        nonce_part.copy_from_slice(&nonce);
    }
    iv
}

/// Encrypts a message for the CM channel.
///
/// The output is `ECB(iv) || CBC(iv, plaintext)`, with the IV derived from the
/// plaintext so it authenticates it.
///
/// The error case is unreachable — the buffer is sized by the same arithmetic
/// the padder uses — but it is returned rather than swallowed. Silently emitting
/// an empty ciphertext would surface as an unexplained disconnect from Steam.
pub fn encrypt_message(key: &SessionKey, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut nonce = [0_u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);

    let iv = derive_iv(key, nonce, plaintext);
    let encrypted_iv = ecb_encrypt_block(key, iv);

    let cipher = Aes256CbcEnc::new(
        GenericArray::from_slice(key.as_bytes()),
        GenericArray::from_slice(&iv),
    );

    // PKCS#7 always adds at least one byte, so a plaintext that is already a
    // whole number of blocks grows by a full block.
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
///
/// Returns [`CryptoError::AuthenticationFailed`] without the plaintext when the
/// tag does not match, even though the bytes are sitting right there.
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

    // Recompute the tag over what we just decrypted and compare it with the one
    // carried in the IV.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> SessionKey {
        SessionKey::from_bytes([0x42; 32])
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
        // The IV carries three random bytes, so a repeated message must not
        // produce a repeated ciphertext — otherwise an observer learns when the
        // client sends the same thing twice.
        let key = test_key();
        let a = encrypt_message(&key, b"same message").expect("must encrypt");
        let b = encrypt_message(&key, b"same message").expect("must encrypt");
        assert_ne!(a, b);
    }

    #[test]
    fn a_tampered_body_fails_authentication() {
        let key = test_key();
        let mut ciphertext = encrypt_message(&key, b"transfer 10 credits").expect("must encrypt");

        // Flip a bit in the payload. CBC will still "decrypt" it into something,
        // which is exactly why the HMAC check has to be the gate.
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
        // A constant "random" key would silently destroy the channel's security
        // while every other test still passed.
        assert_ne!(
            SessionKey::generate().as_bytes(),
            SessionKey::generate().as_bytes()
        );
    }
}
