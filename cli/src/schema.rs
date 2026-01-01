//! JSON Schema validation for SwarmPool snapshots
//!
//! Schema-first publishing: Invalid snapshots never leave the box.
//! All snapshots are validated against their schema before IPFS publish.

use anyhow::{Context, Result};
use serde_json::Value;

/// Snapshot schema definitions
pub mod schemas {
    pub const GENESIS: &str = r#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Genesis Snapshot",
        "description": "Provider registration snapshot",
        "type": "object",
        "required": ["type", "version", "provider", "wallet", "gpus", "timestamp", "nonce", "sig"],
        "properties": {
            "type": { "const": "genesis" },
            "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
            "provider": { "type": "string", "pattern": "^[a-z0-9.-]+\\.eth$" },
            "wallet": { "type": "string", "pattern": "^0x[a-fA-F0-9]{40}$" },
            "gpus": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
            "models": { "type": "array", "items": { "type": "string" } },
            "timestamp": { "type": "integer", "minimum": 0 },
            "nonce": { "type": "string", "minLength": 16 },
            "sig": { "type": "string", "pattern": "^0x[a-fA-F0-9]{130}$" }
        },
        "additionalProperties": false
    }"#;

    pub const JOB: &str = r#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Job Snapshot",
        "description": "Client job submission",
        "type": "object",
        "required": ["type", "version", "job_id", "model", "input_cid", "payment", "client", "timestamp", "nonce", "sig"],
        "properties": {
            "type": { "const": "job" },
            "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
            "job_id": { "type": "string", "minLength": 10 },
            "model": { "type": "string", "minLength": 1 },
            "input_cid": { "type": "string", "pattern": "^(bafy|Qm)[a-zA-Z0-9]+" },
            "params": { "type": "object" },
            "payment": {
                "type": "object",
                "required": ["amount", "token"],
                "properties": {
                    "amount": { "type": "string", "pattern": "^\\d+\\.?\\d*$" },
                    "token": { "type": "string" }
                }
            },
            "client": { "type": "string", "pattern": "^[a-z0-9.-]+\\.eth$" },
            "timestamp": { "type": "integer", "minimum": 0 },
            "nonce": { "type": "string", "minLength": 16 },
            "sig": { "type": "string", "pattern": "^0x[a-fA-F0-9]{130}$" }
        },
        "additionalProperties": false
    }"#;

    pub const CLAIM: &str = r#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Claim Snapshot",
        "description": "Miner job claim intent",
        "type": "object",
        "required": ["type", "version", "claim_id", "job_cid", "provider", "mode", "timestamp", "nonce", "sig"],
        "properties": {
            "type": { "const": "claim" },
            "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
            "claim_id": { "type": "string", "minLength": 10 },
            "job_cid": { "type": "string", "pattern": "^(bafy|Qm)[a-zA-Z0-9]+" },
            "provider": { "type": "string", "pattern": "^[a-z0-9.-]+\\.eth$" },
            "mode": { "enum": ["SOLO", "PPL"] },
            "timestamp": { "type": "integer", "minimum": 0 },
            "nonce": { "type": "string", "minLength": 16 },
            "sig": { "type": "string", "pattern": "^0x[a-fA-F0-9]{130}$" }
        },
        "additionalProperties": false
    }"#;

    pub const PROOF: &str = r#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Proof Snapshot",
        "description": "Completed work proof",
        "type": "object",
        "required": ["type", "version", "proof_id", "job_cid", "output_cid", "metrics", "provider", "timestamp", "proof_hash", "sig"],
        "properties": {
            "type": { "const": "proof" },
            "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
            "proof_id": { "type": "string", "minLength": 10 },
            "job_cid": { "type": "string", "pattern": "^(bafy|Qm)[a-zA-Z0-9]+" },
            "claim_cid": { "type": "string", "pattern": "^(bafy|Qm)[a-zA-Z0-9]+" },
            "output_cid": { "type": "string", "pattern": "^(bafy|Qm)[a-zA-Z0-9]+" },
            "report_cid": { "type": "string", "pattern": "^(bafy|Qm)[a-zA-Z0-9]+" },
            "metrics": {
                "type": "object",
                "required": ["inference_seconds", "confidence"],
                "properties": {
                    "inference_seconds": { "type": "number", "minimum": 0 },
                    "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
                    "model_version": { "type": "string" }
                }
            },
            "provider": { "type": "string", "pattern": "^[a-z0-9.-]+\\.eth$" },
            "timestamp": { "type": "integer", "minimum": 0 },
            "proof_hash": { "type": "string", "pattern": "^0x[a-fA-F0-9]{64}$" },
            "sig": { "type": "string", "pattern": "^0x[a-fA-F0-9]{130}$" }
        },
        "additionalProperties": false
    }"#;

    pub const EPOCH: &str = r#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Epoch Snapshot",
        "description": "Sealed epoch with settlements",
        "type": "object",
        "required": ["type", "version", "epoch_id", "name", "status", "started_at", "jobs_count", "total_volume_usdc", "controller", "timestamp", "sig"],
        "properties": {
            "type": { "const": "epoch" },
            "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
            "epoch_id": { "type": "string", "minLength": 5 },
            "name": { "type": "string" },
            "status": { "enum": ["active", "sealed"] },
            "started_at": { "type": "integer", "minimum": 0 },
            "ended_at": { "type": "integer", "minimum": 0 },
            "jobs_count": { "type": "integer", "minimum": 0 },
            "proofs_count": { "type": "integer", "minimum": 0 },
            "total_volume_usdc": { "type": "string", "pattern": "^\\d+\\.?\\d*$" },
            "proofs": { "type": "array" },
            "settlements": { "type": "object" },
            "merkle_root": { "type": "string", "pattern": "^0x[a-fA-F0-9]{64}$" },
            "controller": { "type": "string", "pattern": "^[a-z0-9.-]+\\.eth$" },
            "timestamp": { "type": "integer", "minimum": 0 },
            "sig": { "type": "string", "pattern": "^0x[a-fA-F0-9]{130}$" }
        },
        "additionalProperties": false
    }"#;
}

/// Schema type enum
#[derive(Debug, Clone, Copy)]
pub enum SchemaType {
    Genesis,
    Job,
    Claim,
    Proof,
    Epoch,
}

impl SchemaType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "genesis" => Some(SchemaType::Genesis),
            "job" => Some(SchemaType::Job),
            "claim" => Some(SchemaType::Claim),
            "proof" => Some(SchemaType::Proof),
            "epoch" => Some(SchemaType::Epoch),
            _ => None,
        }
    }

    pub fn schema(&self) -> &'static str {
        match self {
            SchemaType::Genesis => schemas::GENESIS,
            SchemaType::Job => schemas::JOB,
            SchemaType::Claim => schemas::CLAIM,
            SchemaType::Proof => schemas::PROOF,
            SchemaType::Epoch => schemas::EPOCH,
        }
    }
}

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![msg.into()],
        }
    }

    pub fn errors(errors: Vec<String>) -> Self {
        Self {
            valid: errors.is_empty(),
            errors,
        }
    }
}

/// Validate snapshot against schema
///
/// Returns ValidationResult with any errors found.
/// This MUST pass before any IPFS publish.
pub fn validate_snapshot(data: &Value, schema_type: SchemaType) -> ValidationResult {
    let schema_str = schema_type.schema();

    // Parse schema
    let schema: Value = match serde_json::from_str(schema_str) {
        Ok(s) => s,
        Err(e) => return ValidationResult::error(format!("Invalid schema: {}", e)),
    };

    // Validate
    let mut errors = Vec::new();

    // Check type field
    if let Some(expected_type) = schema["properties"]["type"]["const"].as_str() {
        if data["type"].as_str() != Some(expected_type) {
            errors.push(format!(
                "Invalid type: expected '{}', got {:?}",
                expected_type,
                data["type"]
            ));
        }
    }

    // Check required fields
    if let Some(required) = schema["required"].as_array() {
        for field in required {
            if let Some(field_name) = field.as_str() {
                if data.get(field_name).is_none() {
                    errors.push(format!("Missing required field: {}", field_name));
                }
            }
        }
    }

    // Check property types and patterns
    if let Some(properties) = schema["properties"].as_object() {
        for (key, prop_schema) in properties {
            if let Some(value) = data.get(key) {
                // Check pattern if specified
                if let Some(pattern) = prop_schema["pattern"].as_str() {
                    if let Some(str_val) = value.as_str() {
                        let re = regex::Regex::new(pattern).unwrap();
                        if !re.is_match(str_val) {
                            errors.push(format!(
                                "Field '{}' does not match pattern '{}': {}",
                                key, pattern, str_val
                            ));
                        }
                    }
                }

                // Check type
                if let Some(expected_type) = prop_schema["type"].as_str() {
                    let valid = match expected_type {
                        "string" => value.is_string(),
                        "integer" => value.is_i64() || value.is_u64(),
                        "number" => value.is_number(),
                        "boolean" => value.is_boolean(),
                        "array" => value.is_array(),
                        "object" => value.is_object(),
                        _ => true,
                    };
                    if !valid {
                        errors.push(format!(
                            "Field '{}' has wrong type: expected {}, got {:?}",
                            key, expected_type, value
                        ));
                    }
                }

                // Check enum if specified
                if let Some(enum_values) = prop_schema["enum"].as_array() {
                    let valid = enum_values.iter().any(|v| v == value);
                    if !valid {
                        errors.push(format!(
                            "Field '{}' must be one of {:?}, got {:?}",
                            key, enum_values, value
                        ));
                    }
                }

                // Check minimum for numbers
                if let Some(min) = prop_schema["minimum"].as_f64() {
                    if let Some(num) = value.as_f64() {
                        if num < min {
                            errors.push(format!(
                                "Field '{}' must be >= {}, got {}",
                                key, min, num
                            ));
                        }
                    }
                }

                // Check maximum for numbers
                if let Some(max) = prop_schema["maximum"].as_f64() {
                    if let Some(num) = value.as_f64() {
                        if num > max {
                            errors.push(format!(
                                "Field '{}' must be <= {}, got {}",
                                key, max, num
                            ));
                        }
                    }
                }

                // Check minLength for strings
                if let Some(min_len) = prop_schema["minLength"].as_u64() {
                    if let Some(str_val) = value.as_str() {
                        if (str_val.len() as u64) < min_len {
                            errors.push(format!(
                                "Field '{}' must be at least {} characters",
                                key, min_len
                            ));
                        }
                    }
                }

                // Check minItems for arrays
                if let Some(min_items) = prop_schema["minItems"].as_u64() {
                    if let Some(arr) = value.as_array() {
                        if (arr.len() as u64) < min_items {
                            errors.push(format!(
                                "Field '{}' must have at least {} items",
                                key, min_items
                            ));
                        }
                    }
                }
            }
        }
    }

    // Check for additional properties
    if schema["additionalProperties"] == Value::Bool(false) {
        if let (Some(data_obj), Some(props)) = (data.as_object(), schema["properties"].as_object())
        {
            for key in data_obj.keys() {
                if !props.contains_key(key) {
                    errors.push(format!("Unknown field: {}", key));
                }
            }
        }
    }

    ValidationResult::errors(errors)
}

/// Validate and return Result
pub fn validate(data: &Value, schema_type: SchemaType) -> Result<()> {
    let result = validate_snapshot(data, schema_type);

    if result.valid {
        Ok(())
    } else {
        anyhow::bail!("Schema validation failed:\n  - {}", result.errors.join("\n  - "))
    }
}

/// Validate file contents
pub fn validate_file(path: &str, schema_type: SchemaType) -> Result<ValidationResult> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read file")?;

    let data: Value = serde_json::from_str(&content)
        .context("Failed to parse JSON")?;

    Ok(validate_snapshot(&data, schema_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_genesis() {
        let data = serde_json::json!({
            "type": "genesis",
            "version": "1.0.0",
            "provider": "miner.alice.eth",
            "wallet": "0x1234567890123456789012345678901234567890",
            "gpus": ["RTX 5090"],
            "models": ["queenbee-spine"],
            "timestamp": 1704067200,
            "nonce": "abcdef1234567890",
            "sig": "0x" .to_string() + &"a".repeat(130)
        });

        let result = validate_snapshot(&data, SchemaType::Genesis);
        assert!(result.valid, "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_invalid_genesis_missing_field() {
        let data = serde_json::json!({
            "type": "genesis",
            "version": "1.0.0",
            // missing provider
            "wallet": "0x1234567890123456789012345678901234567890",
            "gpus": ["RTX 5090"],
            "timestamp": 1704067200,
            "nonce": "abcdef1234567890",
            "sig": "0x" .to_string() + &"a".repeat(130)
        });

        let result = validate_snapshot(&data, SchemaType::Genesis);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("provider")));
    }

    #[test]
    fn test_invalid_ens_pattern() {
        let data = serde_json::json!({
            "type": "genesis",
            "version": "1.0.0",
            "provider": "invalid-ens",  // Missing .eth
            "wallet": "0x1234567890123456789012345678901234567890",
            "gpus": ["RTX 5090"],
            "timestamp": 1704067200,
            "nonce": "abcdef1234567890",
            "sig": "0x" .to_string() + &"a".repeat(130)
        });

        let result = validate_snapshot(&data, SchemaType::Genesis);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("pattern")));
    }
}
