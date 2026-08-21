use argon2::{Argon2, Params};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use std::fmt;

const MAGIC_HEADER: &[u8] = b"NOTEDOG_ENC_V1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

#[derive(Debug)]
pub enum CryptoError {
    InvalidHeader,
    PayloadTooShort,
    KeyDerivationError,
    DecryptionFailed,
    EncryptionFailed,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::InvalidHeader => write!(f, "Invalid file format header (not an encrypted Notedog note)"),
            CryptoError::PayloadTooShort => write!(f, "Encrypted file payload is too short"),
            CryptoError::KeyDerivationError => write!(f, "Failed to derive encryption key"),
            CryptoError::DecryptionFailed => write!(f, "Decryption failed (incorrect password or corrupted file)"),
            CryptoError::EncryptionFailed => write!(f, "Encryption failed"),
        }
    }
}

impl std::error::Error for CryptoError {}

pub fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], CryptoError> {
    let params = Params::new(19456, 2, 1, Some(32))
        .map_err(|_| CryptoError::KeyDerivationError)?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|_| CryptoError::KeyDerivationError)?;

    Ok(key)
}

pub fn encrypt_note(plaintext: &str, passphrase: &str) -> Result<Vec<u8>, CryptoError> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key = derive_key(passphrase, &salt)?;
    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|_| CryptoError::EncryptionFailed)?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let mut result = Vec::with_capacity(MAGIC_HEADER.len() + SALT_LEN + NONCE_LEN + ciphertext.len());
    result.extend_from_slice(MAGIC_HEADER);
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt_note(data: &[u8], passphrase: &str) -> Result<String, CryptoError> {
    let header_len = MAGIC_HEADER.len();
    if data.len() < header_len + SALT_LEN + NONCE_LEN {
        return Err(CryptoError::PayloadTooShort);
    }

    if &data[..header_len] != MAGIC_HEADER {
        return Err(CryptoError::InvalidHeader);
    }

    let salt = &data[header_len..header_len + SALT_LEN];
    let nonce_bytes = &data[header_len + SALT_LEN..header_len + SALT_LEN + NONCE_LEN];
    let ciphertext = &data[header_len + SALT_LEN + NONCE_LEN..];

    let key = derive_key(passphrase, salt)?;
    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|_| CryptoError::DecryptionFailed)?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    String::from_utf8(plaintext_bytes).map_err(|_| CryptoError::DecryptionFailed)
}

pub fn is_encrypted_data(data: &[u8]) -> bool {
    data.len() >= MAGIC_HEADER.len() && &data[..MAGIC_HEADER.len()] == MAGIC_HEADER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption_cycle() {
        let text = "# Top Secret Note\n\nConfidential content.";
        let pass = "SuperSecret123!";

        let encrypted = encrypt_note(text, pass).expect("Encryption failed");
        assert!(is_encrypted_data(&encrypted));

        let decrypted = decrypt_note(&encrypted, pass).expect("Decryption failed");
        assert_eq!(decrypted, text);
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let text = "Data";
        let pass = "CorrectPassword";
        let wrong_pass = "WrongPassword";

        let encrypted = encrypt_note(text, pass).unwrap();
        assert!(decrypt_note(&encrypted, wrong_pass).is_err());
    }
}

