//! Data models for SwarmPool CLI

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// PAYOUT CONSTANTS
// ============================================================================

/// Miner pool percentage (75%)
pub const MINERS_PCT: f64 = 0.75;

/// Hive operations percentage (25%)
pub const HIVE_PCT: f64 = 0.25;

/// USDC decimals (6) - track in microunits for precision
pub const USDC_DECIMALS: u32 = 6;

/// Convert USDC amount to microunits (e.g., $0.10 -> 100_000)
pub fn to_microunits(amount: f64) -> u64 {
    (amount * 10_f64.powi(USDC_DECIMALS as i32)).round() as u64
}

/// Convert microunits back to USDC (e.g., 100_000 -> $0.10)
pub fn from_microunits(micro: u64) -> f64 {
    micro as f64 / 10_f64.powi(USDC_DECIMALS as i32)
}

/// Execution mode for job claims
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ExecutionMode {
    /// Winner takes full job reward (first valid proof wins)
    Solo,
    /// Pay-Per-Load: proportional payout based on compute_seconds
    Ppl,
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionMode::Solo => write!(f, "SOLO"),
            ExecutionMode::Ppl => write!(f, "PPL"),
        }
    }
}

impl std::str::FromStr for ExecutionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SOLO" => Ok(ExecutionMode::Solo),
            "PPL" => Ok(ExecutionMode::Ppl),
            _ => Err(format!("Invalid mode: {}. Use SOLO or PPL", s)),
        }
    }
}

/// Claim snapshot - miner intent to execute a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimSnapshot {
    #[serde(rename = "type")]
    pub snapshot_type: String,
    pub version: String,
    pub claim_id: String,
    pub job_id: String,
    pub job_cid: String,
    pub provider: String,
    pub mode: ExecutionMode,
    pub timestamp: i64,
    pub nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

/// Job submission snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSnapshot {
    #[serde(rename = "type")]
    pub snapshot_type: String,
    pub version: String,
    pub job_id: String,
    pub job_type: String,
    pub model: String,
    pub input_cid: String,
    pub params: JobParams,
    pub payment: Payment,
    pub client: String,
    pub timestamp: i64,
    pub nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobParams {
    pub confidence_threshold: f64,
    pub output_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub amount: String,
    pub token: String,
}

/// Result/Proof snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSnapshot {
    #[serde(rename = "type")]
    pub snapshot_type: String,
    pub version: String,
    pub proof_id: String,
    pub job_id: String,
    pub job_cid: String,
    pub status: String,
    pub output_cid: String,
    pub report_cid: Option<String>,
    pub metrics: ProofMetrics,
    pub provider: String,
    pub timestamp: i64,
    pub proof_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetrics {
    pub inference_seconds: f64,
    pub compute_seconds: f64,  // Total compute time (for PPL proportional payout)
    pub confidence: f64,
    pub model_version: String,
}

/// Epoch snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSnapshot {
    #[serde(rename = "type")]
    pub snapshot_type: String,
    pub version: String,
    pub epoch_id: String,
    pub name: String,
    pub status: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub jobs_count: u64,
    pub total_volume_usdc: String,
    pub merkle_root: Option<String>,
    pub settlements: Option<Settlements>,
    pub controller: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

/// Settlement calculation for an epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlements {
    /// Total epoch volume in USDC
    pub total_volume: f64,
    /// Total miner pool (75%)
    pub miner_pool: f64,
    /// Hive operations cut (25%)
    pub hive_ops: f64,
    /// Per-provider earnings (ENS -> USDC amount)
    pub providers: HashMap<String, f64>,
    /// Dust assigned to hive (rounding remainder)
    pub dust_to_hive: f64,
}

/// Individual job settlement (computed at seal time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSettlement {
    pub job_id: String,
    pub job_cid: String,
    pub reward: f64,
    pub mode: ExecutionMode,
    pub miner_pool: f64,
    pub hive_cut: f64,
    /// For SOLO: single winner. For PPL: proportional split
    pub payouts: HashMap<String, f64>,
}

/// Proof with compute contribution (for PPL calculation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofContribution {
    pub proof_cid: String,
    pub provider: String,
    pub compute_seconds: f64,
    pub mode: ExecutionMode,
}

/// Provider registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRegistration {
    #[serde(rename = "type")]
    pub snapshot_type: String,
    pub provider: String,
    pub wallet: String,
    pub gpus: Vec<String>,
    pub models: Vec<String>,
    pub timestamp: i64,
    pub nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

/// Provider info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub ens: String,
    pub wallet: String,
    pub status: String,
    pub registered_at: i64,
    pub last_heartbeat: i64,
    pub gpus: Vec<String>,
    pub models: Vec<String>,
    pub jobs_completed: u64,
    pub total_earnings: f64,
    pub available_balance: f64,
}

/// Pool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    pub pool_id: String,
    pub version: String,
    pub total_jobs: u64,
    pub total_proofs: u64,
    pub total_volume_usdc: f64,
    pub current_epoch: Option<String>,
    pub epoch_jobs: u64,
    pub epoch_volume: f64,
    pub pending_jobs: Vec<String>,
    pub active_providers: std::collections::HashMap<String, ProviderInfo>,
    pub last_updated: i64,
}

/// Network stats for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_jobs: u64,
    pub total_volume: f64,
    pub active_providers: u64,
    pub online_providers: u64,
    pub pending_jobs: u64,
    pub current_epoch: String,
    pub epoch_jobs: u64,
    pub epoch_time_remaining: Option<String>,
}
