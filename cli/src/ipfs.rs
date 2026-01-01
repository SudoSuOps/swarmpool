//! IPFS client utilities with canonical directory layout
//!
//! Directory Structure:
//! /swarmpool/
//! ├── genesis/          # Provider registrations
//! │   └── {provider}.json
//! ├── epochs/           # Epoch snapshots
//! │   └── {epoch_id}.json
//! ├── jobs/             # Job submissions
//! │   └── {job_id}.json
//! ├── claims/           # Job claims
//! │   └── {claim_id}.json
//! ├── proofs/           # Completed proofs
//! │   └── {proof_id}.json
//! └── index/            # Indexes and state
//!     ├── state.json
//!     └── providers.json

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

use crate::models::{EpochSnapshot, PoolState};

const IPFS_API: &str = "http://localhost:5001/api/v0";
const IPFS_GATEWAY: &str = "https://ipfs.io/ipfs";

/// Canonical IPFS directory paths
pub mod paths {
    pub const ROOT: &str = "/swarmpool";
    pub const GENESIS: &str = "/swarmpool/genesis";
    pub const EPOCHS: &str = "/swarmpool/epochs";
    pub const JOBS: &str = "/swarmpool/jobs";
    pub const CLAIMS: &str = "/swarmpool/claims";
    pub const PROOFS: &str = "/swarmpool/proofs";
    pub const INDEX: &str = "/swarmpool/index";
}

/// Check IPFS connection
pub async fn check_connection() -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/id", IPFS_API))
        .send()
        .await
        .context("Failed to connect to IPFS daemon")?;

    if !response.status().is_success() {
        anyhow::bail!("IPFS daemon returned error: {}", response.status());
    }

    Ok(())
}

/// Initialize canonical directory structure
pub async fn init_directories() -> Result<()> {
    let client = reqwest::Client::new();

    for dir in [
        paths::ROOT,
        paths::GENESIS,
        paths::EPOCHS,
        paths::JOBS,
        paths::CLAIMS,
        paths::PROOFS,
        paths::INDEX,
    ] {
        client
            .post(&format!("{}/files/mkdir?arg={}&parents=true", IPFS_API, dir))
            .send()
            .await
            .context(format!("Failed to create directory: {}", dir))?;
    }

    Ok(())
}

/// Upload file to IPFS (returns CID)
pub async fn upload_file(path: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let file_path = Path::new(path);

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let file_bytes = tokio::fs::read(path)
        .await
        .context("Failed to read file")?;

    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(file_bytes).file_name(file_name.to_string()),
    );

    let response = client
        .post(&format!("{}/add", IPFS_API))
        .multipart(form)
        .send()
        .await
        .context("Failed to upload to IPFS")?;

    let result: serde_json::Value = response.json().await?;

    result["Hash"]
        .as_str()
        .map(|s| s.to_string())
        .context("Invalid response from IPFS")
}

/// Upload JSON to IPFS (returns CID)
pub async fn upload_json<T: Serialize>(data: &T) -> Result<String> {
    let client = reqwest::Client::new();
    let json_str = serde_json::to_string_pretty(data)?;

    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(json_str.into_bytes()).file_name("data.json"),
    );

    let response = client
        .post(&format!("{}/add", IPFS_API))
        .multipart(form)
        .send()
        .await
        .context("Failed to upload to IPFS")?;

    let result: serde_json::Value = response.json().await?;

    result["Hash"]
        .as_str()
        .map(|s| s.to_string())
        .context("Invalid response from IPFS")
}

/// Write JSON to canonical MFS path
pub async fn write_to_path<T: Serialize>(mfs_path: &str, data: &T) -> Result<String> {
    let client = reqwest::Client::new();
    let json_str = serde_json::to_string_pretty(data)?;

    // First add to IPFS to get CID
    let cid = upload_json(data).await?;

    // Then copy to MFS path
    client
        .post(&format!(
            "{}/files/cp?arg=/ipfs/{}&arg={}",
            IPFS_API, cid, mfs_path
        ))
        .send()
        .await
        .context("Failed to write to MFS path")?;

    Ok(cid)
}

/// Write job to canonical path: /swarmpool/jobs/{job_id}.json
pub async fn write_job<T: Serialize>(job_id: &str, data: &T) -> Result<String> {
    let path = format!("{}/{}.json", paths::JOBS, job_id);
    write_to_path(&path, data).await
}

/// Write claim to canonical path: /swarmpool/claims/{claim_id}.json
pub async fn write_claim<T: Serialize>(claim_id: &str, data: &T) -> Result<String> {
    let path = format!("{}/{}.json", paths::CLAIMS, claim_id);
    write_to_path(&path, data).await
}

/// Write proof to canonical path: /swarmpool/proofs/{proof_id}.json
pub async fn write_proof<T: Serialize>(proof_id: &str, data: &T) -> Result<String> {
    let path = format!("{}/{}.json", paths::PROOFS, proof_id);
    write_to_path(&path, data).await
}

/// Write epoch to canonical path: /swarmpool/epochs/{epoch_id}.json
pub async fn write_epoch<T: Serialize>(epoch_id: &str, data: &T) -> Result<String> {
    let path = format!("{}/{}.json", paths::EPOCHS, epoch_id);
    write_to_path(&path, data).await
}

/// Write genesis (provider init) to canonical path: /swarmpool/genesis/{provider}.json
pub async fn write_genesis<T: Serialize>(provider: &str, data: &T) -> Result<String> {
    // Sanitize provider name for path
    let safe_name = provider.replace('.', "_");
    let path = format!("{}/{}.json", paths::GENESIS, safe_name);
    write_to_path(&path, data).await
}

/// Fetch JSON from IPFS by CID (tries local API first, then gateway)
pub async fn fetch_json<T: DeserializeOwned>(cid: &str) -> Result<T> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Try local IPFS API first
    let local_url = format!("{}/cat?arg={}", IPFS_API, cid);
    if let Ok(response) = client.post(&local_url).send().await {
        if response.status().is_success() {
            if let Ok(data) = response.json().await {
                return Ok(data);
            }
        }
    }

    // Fall back to public gateway
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let url = format!("{}/{}", IPFS_GATEWAY, cid);

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch from IPFS")?;

    if !response.status().is_success() {
        anyhow::bail!("IPFS fetch failed: {}", response.status());
    }

    let data: T = response.json().await.context("Failed to parse JSON")?;
    Ok(data)
}

/// Read JSON from MFS path
pub async fn read_from_path<T: DeserializeOwned>(mfs_path: &str) -> Result<T> {
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/files/read?arg={}", IPFS_API, mfs_path))
        .send()
        .await
        .context("Failed to read from MFS")?;

    if !response.status().is_success() {
        anyhow::bail!("MFS read failed: {}", response.status());
    }

    let data: T = response.json().await.context("Failed to parse JSON")?;
    Ok(data)
}

/// Pin CID to local IPFS node
pub async fn pin(cid: &str) -> Result<()> {
    let client = reqwest::Client::new();

    client
        .post(&format!("{}/pin/add?arg={}", IPFS_API, cid))
        .send()
        .await
        .context("Failed to pin CID")?;

    Ok(())
}

/// Subscribe to IPFS pubsub topic
pub async fn pubsub_subscribe(topic: &str) -> Result<()> {
    let client = reqwest::Client::new();

    client
        .post(&format!(
            "{}/pubsub/sub?arg={}",
            IPFS_API,
            urlencoding::encode(topic)
        ))
        .send()
        .await
        .context("Failed to subscribe to topic")?;

    Ok(())
}

/// Publish to IPFS pubsub topic
pub async fn pubsub_publish<T: Serialize>(topic: &str, data: &T) -> Result<()> {
    let client = reqwest::Client::new();
    let json_str = serde_json::to_string(data)?;

    client
        .post(&format!(
            "{}/pubsub/pub?arg={}&arg={}",
            IPFS_API,
            urlencoding::encode(topic),
            urlencoding::encode(&json_str)
        ))
        .send()
        .await
        .context("Failed to publish to topic")?;

    Ok(())
}

/// Fetch pool state from index
pub async fn fetch_pool_state(pool: &str) -> Result<PoolState> {
    // Try to read from MFS first
    match read_from_path::<PoolState>(&format!("{}/state.json", paths::INDEX)).await {
        Ok(state) => return Ok(state),
        Err(_) => {
            // Fall back to mock data for development
            Ok(PoolState {
                pool_id: pool.to_string(),
                version: "1.0.0".to_string(),
                total_jobs: 12847,
                total_proofs: 12800,
                total_volume_usdc: 1284.70,
                current_epoch: Some("epoch-048".to_string()),
                epoch_jobs: 156,
                epoch_volume: 15.60,
                pending_jobs: vec![],
                active_providers: std::collections::HashMap::new(),
                last_updated: chrono::Utc::now().timestamp(),
            })
        }
    }
}

/// Fetch epochs
pub async fn fetch_epochs(pool: &str, limit: u32) -> Result<Vec<EpochSnapshot>> {
    // In production: list /swarmpool/epochs/ and fetch each
    // Mock data for now
    Ok(vec![
        EpochSnapshot {
            snapshot_type: "epoch".to_string(),
            version: "1.0.0".to_string(),
            epoch_id: "epoch-048".to_string(),
            name: "Golf".to_string(),
            status: "active".to_string(),
            started_at: chrono::Utc::now().timestamp() - 3600,
            ended_at: None,
            jobs_count: 156,
            total_volume_usdc: "15.60".to_string(),
            merkle_root: None,
            settlements: None,
            controller: "merlin.swarmos.eth".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            sig: None,
        },
        EpochSnapshot {
            snapshot_type: "epoch".to_string(),
            version: "1.0.0".to_string(),
            epoch_id: "epoch-047".to_string(),
            name: "Foxtrot".to_string(),
            status: "sealed".to_string(),
            started_at: chrono::Utc::now().timestamp() - 7200,
            ended_at: Some(chrono::Utc::now().timestamp() - 3600),
            jobs_count: 312,
            total_volume_usdc: "31.20".to_string(),
            merkle_root: Some("0xabc123...".to_string()),
            settlements: None,
            controller: "merlin.swarmos.eth".to_string(),
            timestamp: chrono::Utc::now().timestamp() - 3600,
            sig: Some("0x...".to_string()),
        },
    ])
}

/// Fetch single epoch
pub async fn fetch_epoch(pool: &str, epoch_id: &str) -> Result<EpochSnapshot> {
    let epochs = fetch_epochs(pool, 100).await?;
    epochs
        .into_iter()
        .find(|e| e.epoch_id == epoch_id)
        .context("Epoch not found")
}

/// List files in MFS directory
pub async fn list_directory(mfs_path: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/files/ls?arg={}&long=true", IPFS_API, mfs_path))
        .send()
        .await
        .context("Failed to list directory")?;

    let result: serde_json::Value = response.json().await?;

    let entries = result["Entries"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e["Name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(entries)
}
