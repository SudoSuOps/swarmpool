//! Cryptographic utilities for EIP-191 signing and verification

use anyhow::{Context, Result};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::Signature;
use serde::Serialize;

/// Sign a snapshot/struct with EIP-191 personal sign (async)
/// Uses keccak256 for hashing (Ethereum standard)
pub async fn sign_snapshot<T: Serialize>(data: &T, private_key: &str) -> Result<String> {
    // Serialize to canonical JSON (sorted keys)
    let json = serde_json::to_string(data)?;

    // Hash with keccak256 (Ethereum standard)
    let hash = ethers::utils::keccak256(json.as_bytes());

    // Parse private key and sign
    let wallet: LocalWallet = private_key
        .trim_start_matches("0x")
        .parse()
        .context("Invalid private key format")?;

    // EIP-191 personal sign: "\x19Ethereum Signed Message:\n" + len + message
    let message = format!(
        "\x19Ethereum Signed Message:\n{}{}",
        hash.len(),
        hex::encode(hash)
    );
    let message_hash = ethers::utils::keccak256(message.as_bytes());

    // Sign asynchronously
    let signature: Signature = wallet
        .sign_message(&message_hash[..])
        .await
        .context("Failed to sign message")?;

    Ok(format!("0x{}", hex::encode(signature.to_vec())))
}

/// Sign raw JSON value, returns JSON with sig field added
pub async fn sign_json(data: &serde_json::Value, private_key: &str) -> Result<serde_json::Value> {
    let sig = sign_snapshot(data, private_key).await?;

    let mut signed = data.clone();
    if let Some(obj) = signed.as_object_mut() {
        obj.insert("sig".to_string(), serde_json::Value::String(sig));
    }

    Ok(signed)
}

/// Generate random hex string
pub fn random_hex(bytes: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..bytes).map(|_| rng.gen()).collect();
    hex::encode(random_bytes)
}

/// Verify an EIP-191 signature
pub fn verify_signature(
    data: &str,
    signature: &str,
    expected_address: &str,
) -> Result<bool> {
    // Hash the data with keccak256
    let hash = ethers::utils::keccak256(data.as_bytes());

    // Create EIP-191 message hash
    let message = format!(
        "\x19Ethereum Signed Message:\n{}{}",
        hash.len(),
        hex::encode(hash)
    );
    let message_hash = ethers::utils::keccak256(message.as_bytes());

    // Parse signature
    let sig_bytes = hex::decode(signature.trim_start_matches("0x"))
        .context("Invalid signature format")?;
    let signature: Signature = sig_bytes
        .as_slice()
        .try_into()
        .context("Invalid signature length")?;

    // Recover address
    let recovered = signature
        .recover(&message_hash[..])
        .context("Failed to recover address from signature")?;

    let recovered_addr = format!("{:?}", recovered);

    Ok(recovered_addr.to_lowercase() == expected_address.to_lowercase())
}

/// Hash data with keccak256 (Ethereum standard)
pub fn keccak256_hash(data: &[u8]) -> String {
    let result = ethers::utils::keccak256(data);
    format!("0x{}", hex::encode(result))
}

/// Legacy alias - use keccak256_hash instead
#[deprecated(note = "Use keccak256_hash for Ethereum compatibility")]
pub fn sha256_hash(data: &[u8]) -> String {
    keccak256_hash(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_hex() {
        let hex1 = random_hex(16);
        let hex2 = random_hex(16);

        assert_eq!(hex1.len(), 32); // 16 bytes = 32 hex chars
        assert_ne!(hex1, hex2);
    }

    #[test]
    fn test_keccak256_hash() {
        let hash = keccak256_hash(b"hello");
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 66); // 0x + 64 hex chars
        // Known keccak256("hello")
        assert_eq!(
            hash,
            "0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
        );
    }
}
