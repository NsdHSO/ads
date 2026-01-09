//! Application-level E2EE scaffold.
//! - Symmetric encryption via AES-GCM.
//! - Hook points for rustls-based session key derivation (feature = "rustls").

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("encryption failed")]
    Encrypt,
    #[error("decryption failed")]
    Decrypt,
}

/// Opaque session for encrypt/decrypt of payloads.
#[derive(Clone)]
pub struct Session {
    key: aes_gcm::Key<aes_gcm::aes::Aes256>,
}

impl Session {
    /// Construct from a 32-byte key.
    pub fn from_key(key: [u8; 32]) -> Self {
        Self { key: key.into() }
    }

    /// Encrypt a payload with a random nonce (12 bytes) prepended to the ciphertext.
    pub fn seal(&self, aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        let cipher = Aes256Gcm::new(&self.key);
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let mut out = Vec::with_capacity(12 + plaintext.len() + 16);
        out.extend_from_slice(&nonce_bytes);
        let ct = cipher
            .encrypt(
                nonce,
                aes_gcm::aead::Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| Error::Encrypt)?;
        out.extend_from_slice(&ct);
        Ok(out)
    }

    /// Decrypt a payload produced by `seal`.
    pub fn open(&self, aad: &[u8], framed: &[u8]) -> Result<Vec<u8>, Error> {
        if framed.len() < 12 {
            return Err(Error::Decrypt);
        }
        let (nonce_bytes, ct) = framed.split_at(12);
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(nonce, aes_gcm::aead::Payload { msg: ct, aad })
            .map_err(|_| Error::Decrypt)
    }
}

/// Derive a session from a pre-shared key (placeholder for initial prototypes).
pub fn session_from_psk(psk: &[u8]) -> Session {
    use blake3::hash as blake3_hash;
    let mut key = [0u8; 32];
    key.copy_from_slice(blake3_hash(psk).as_bytes());
    Session::from_key(key)
}

#[cfg(feature = "rustls")]
pub mod tls {
    //! Hook points to derive an application-level session key via a rustls TLS 1.3 handshake.
    //! Integrate by exporting keying material (EKM) after handshake and feeding it to `Session::from_key`.
    use super::Session;
    use rustls::Connection;

    pub fn session_from_ekm(conn: &Connection) -> Option<Session> {
        // Export 32 bytes of keying material following RFC 5705-like interface (rustls API provides EKM).
        let mut out = [0u8; 32];
        let label = b"ads-e2ee-2026";
        let context: &[u8] = &[];
        conn.export_keying_material(&mut out, label, Some(context))
            .ok()?;
        Some(Session::from_key(out))
    }
}
