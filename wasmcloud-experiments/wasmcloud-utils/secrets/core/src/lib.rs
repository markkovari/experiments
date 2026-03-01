use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum SecretError {
    NotInitialized,
    NotFound,
    AlreadyExists,
    InvalidConfig,
    EncryptionError,
    StorageError,
    PermissionDenied,
}

impl std::fmt::Display for SecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretError::NotInitialized => write!(f, "not initialized"),
            SecretError::NotFound => write!(f, "not found"),
            SecretError::AlreadyExists => write!(f, "already exists"),
            SecretError::InvalidConfig => write!(f, "invalid config"),
            SecretError::EncryptionError => write!(f, "encryption error"),
            SecretError::StorageError => write!(f, "storage error"),
            SecretError::PermissionDenied => write!(f, "permission denied"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretValue {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub name: String,
    pub version: u32,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone)]
pub struct SecretsConfig {
    pub backend: String,
    pub namespace: Option<String>,
    pub encryption_key: Option<String>,
}

/// Encrypt plaintext bytes using AES-256-GCM.
/// Returns base64(nonce || ciphertext).
pub fn encrypt(key_b64: &str, plaintext: &[u8]) -> Result<String, SecretError> {
    let key_bytes = B64.decode(key_b64).map_err(|_| SecretError::InvalidConfig)?;
    if key_bytes.len() != 32 {
        return Err(SecretError::InvalidConfig);
    }
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| SecretError::EncryptionError)?;
    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(B64.encode(combined))
}

/// Decrypt base64(nonce || ciphertext) using AES-256-GCM.
pub fn decrypt(key_b64: &str, encoded: &str) -> Result<Vec<u8>, SecretError> {
    let key_bytes = B64.decode(key_b64).map_err(|_| SecretError::InvalidConfig)?;
    if key_bytes.len() != 32 {
        return Err(SecretError::InvalidConfig);
    }
    let combined = B64.decode(encoded).map_err(|_| SecretError::EncryptionError)?;
    if combined.len() < 12 {
        return Err(SecretError::EncryptionError);
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| SecretError::EncryptionError)
}

/// KV key for the secret data blob.
pub fn kv_data_key(namespace: &str, name: &str) -> String {
    format!("{namespace}:{name}")
}

/// KV key for the secret metadata.
pub fn kv_meta_key(namespace: &str, name: &str) -> String {
    format!("{namespace}:{name}:meta")
}

/// Returns true if a KV key is a metadata key (ends with `:meta`).
pub fn is_meta_key(key: &str) -> bool {
    key.ends_with(":meta")
}

/// Extract the secret name from a data KV key given the namespace prefix.
pub fn name_from_data_key<'a>(namespace: &str, key: &'a str) -> Option<&'a str> {
    let prefix = format!("{namespace}:");
    key.strip_prefix(&prefix)
}

/// In-memory KV store used for unit testing and the `secrets-core` self-contained tests.
/// Production uses wasi:keyvalue via the WIT component layer.
#[cfg(test)]
pub mod mem_store {
    use std::collections::HashMap;
    use std::cell::RefCell;

    thread_local! {
        static STORE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    }

    pub fn set(key: &str, value: &str) {
        STORE.with(|s| s.borrow_mut().insert(key.to_string(), value.to_string()));
    }

    pub fn get(key: &str) -> Option<String> {
        STORE.with(|s| s.borrow().get(key).cloned())
    }

    pub fn delete(key: &str) {
        STORE.with(|s| s.borrow_mut().remove(key));
    }

    pub fn keys() -> Vec<String> {
        STORE.with(|s| s.borrow().keys().cloned().collect())
    }

    pub fn clear() {
        STORE.with(|s| s.borrow_mut().clear());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mem_store as kv;

    fn test_key() -> String {
        // 32 bytes of zeros, base64-encoded — for tests only
        B64.encode([0u8; 32])
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"super secret value";
        let enc = encrypt(&key, plaintext).unwrap();
        let dec = decrypt(&key, &enc).unwrap();
        assert_eq!(dec, plaintext);
    }

    #[test]
    fn different_nonces_each_encrypt() {
        let key = test_key();
        let enc1 = encrypt(&key, b"value").unwrap();
        let enc2 = encrypt(&key, b"value").unwrap();
        // Nonces are random — ciphertexts should differ
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn wrong_key_fails_decrypt() {
        let key1 = test_key();
        let key2 = B64.encode([1u8; 32]);
        let enc = encrypt(&key1, b"value").unwrap();
        assert_eq!(decrypt(&key2, &enc), Err(SecretError::EncryptionError));
    }

    #[test]
    fn kv_keys_are_correct() {
        assert_eq!(kv_data_key("ns", "db-pass"), "ns:db-pass");
        assert_eq!(kv_meta_key("ns", "db-pass"), "ns:db-pass:meta");
        assert!(is_meta_key("ns:db-pass:meta"));
        assert!(!is_meta_key("ns:db-pass"));
    }

    #[test]
    fn rotate_bumps_version() {
        kv::clear();
        let ns = "test";
        let name = "my-secret";
        let key = test_key();

        // Simulate initial set: write data + meta
        let enc = encrypt(&key, b"initial").unwrap();
        kv::set(&kv_data_key(ns, name), &enc);
        let meta = SecretMetadata {
            name: name.to_string(),
            version: 1,
            created_at_ms: 1000,
            updated_at_ms: 1000,
        };
        kv::set(&kv_meta_key(ns, name), &serde_json::to_string(&meta).unwrap());

        // Simulate rotate: read meta, bump version, write new data + meta
        let raw_meta = kv::get(&kv_meta_key(ns, name)).unwrap();
        let mut m: SecretMetadata = serde_json::from_str(&raw_meta).unwrap();
        let new_enc = encrypt(&key, b"rotated").unwrap();
        m.version += 1;
        m.updated_at_ms = 2000;
        kv::set(&kv_data_key(ns, name), &new_enc);
        kv::set(&kv_meta_key(ns, name), &serde_json::to_string(&m).unwrap());

        // Verify
        let final_meta: SecretMetadata =
            serde_json::from_str(&kv::get(&kv_meta_key(ns, name)).unwrap()).unwrap();
        assert_eq!(final_meta.version, 2);
        let dec = decrypt(&key, &kv::get(&kv_data_key(ns, name)).unwrap()).unwrap();
        assert_eq!(dec, b"rotated");
    }

    #[test]
    fn list_names_excludes_meta_keys() {
        kv::clear();
        let ns = "app";
        kv::set(&kv_data_key(ns, "alpha"), "enc-alpha");
        kv::set(&kv_meta_key(ns, "alpha"), "{}");
        kv::set(&kv_data_key(ns, "beta"), "enc-beta");
        kv::set(&kv_meta_key(ns, "beta"), "{}");

        let prefix = format!("{ns}:");
        let mut names: Vec<String> = kv::keys()
            .into_iter()
            .filter(|k| k.starts_with(&prefix) && !is_meta_key(k))
            .filter_map(|k| name_from_data_key(ns, &k).map(|n| n.to_string()))
            .collect();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta"]);
    }
}
