// Simple API Key encryption
//
// Uses XOR cipher with a derived key + hex encoding.
// The key is derived from the app data directory path to make it
// machine-specific. This is not military-grade security but prevents
// casual plain-text leakage of API keys in the database.
//
// NOTE: For production use, replace with platform-native secure storage
// (Windows Credential Manager, macOS Keychain, Linux libsecret).

use std::path::PathBuf;

/// Derive an encryption key from the app data directory.
/// This makes the encrypted key machine-specific.
fn derive_key() -> Vec<u8> {
    let seed = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("womenhenku")
        .join(".crypto_key")
        .to_string_lossy()
        .to_string();

    // Simple key derivation: repeat the seed bytes to reach 32 bytes
    let seed_bytes = seed.as_bytes();
    let mut key = Vec::with_capacity(32);
    for i in 0..32 {
        key.push(seed_bytes[i % seed_bytes.len()].wrapping_add((i * 17) as u8));
    }
    key
}

/// Encrypt plain text API key to hex-encoded cipher text.
pub fn encrypt(plain_text: &str) -> String {
    let key = derive_key();
    let bytes: Vec<u8> = plain_text
        .as_bytes()
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect();
    hex_encode(&bytes)
}

/// Decrypt hex-encoded cipher text back to plain text API key.
pub fn decrypt(hex_text: &str) -> Result<String, String> {
    let key = derive_key();
    let bytes = hex_decode(hex_text).map_err(|e| format!("解密失败: {}", e))?;
    let plain: Vec<u8> = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect();
    String::from_utf8(plain).map_err(|e| format!("解密结果无效: {}", e))
}

/// Encode bytes to hex string.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Decode hex string to bytes.
fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return Err("hex 字符串长度必须为偶数".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| format!("无效的 hex 字符: {}", &hex[i..i + 2]))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "sk-ant-this-is-a-test-api-key";
        let encrypted = encrypt(original);
        assert_ne!(encrypted, original);
        assert!(!encrypted.is_empty());

        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_encrypt_empty_string() {
        let original = "";
        let encrypted = encrypt(original);
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_decrypt_invalid_hex() {
        let result = decrypt("not-hex-string!");
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_produces_different_output_for_different_inputs() {
        let e1 = encrypt("key1");
        let e2 = encrypt("key2");
        assert_ne!(e1, e2);
    }
}
