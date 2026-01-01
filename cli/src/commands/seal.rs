//! Seal command - Seal an epoch (Merlin controller only)
//!
//! Settlement Math:
//! - SOLO: winner gets R * 0.75 (miners_pct)
//! - PPL: each miner gets R * 0.75 * (their_compute / total_compute)
//! - Hive: always gets R * 0.25 (hive_pct)
//! - Dust: remainder from rounding → hive ops

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::time::Duration;

use crate::crypto;
use crate::ipfs;
use crate::models::{
    EpochSnapshot, ExecutionMode, Settlements,
    MINERS_PCT, HIVE_PCT, to_microunits, from_microunits,
};

pub async fn execute(
    epoch_id: Option<String>,
    key: Option<String>,
    pool: &str,
) -> Result<()> {
    println!("{}", "Sealing Epoch".cyan().bold());
    println!("  {}", "(Merlin controller only)".bright_black());
    println!();

    // Get private key (must be Merlin's key)
    let private_key = key
        .or_else(|| std::env::var("SWARM_PRIVATE_KEY").ok())
        .context("Private key required. Use --key or set SWARM_PRIVATE_KEY")?;

    // Fetch current pool state
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Fetching pool state...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let pool_state = ipfs::fetch_pool_state(pool).await?;

    let target_epoch = epoch_id
        .or(pool_state.current_epoch.clone())
        .context("No active epoch to seal")?;

    pb.finish_with_message(format!("{} Pool state fetched", "✓".green()));

    println!("  {} {}", "Epoch:".bright_black(), target_epoch.cyan());
    println!("  {} {}", "Jobs:".bright_black(), pool_state.epoch_jobs);
    println!("  {} ${:.2}", "Volume:".bright_black(), pool_state.epoch_volume);
    println!();

    // Collect proofs for epoch
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Collecting epoch proofs...");
    pb.enable_steady_tick(Duration::from_millis(100));

    // TODO: Fetch actual proofs from IPFS /swarmpool/proofs/
    // For now: mock data representing epoch activity
    let mock_proofs = generate_mock_epoch_proofs();
    let proof_count = mock_proofs.len();

    pb.finish_with_message(format!("{} Collected {} proofs", "✓".green(), proof_count));

    // Calculate settlements
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Calculating settlements...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let settlements = calculate_settlements(&mock_proofs, pool_state.epoch_volume);

    pb.finish_with_message(format!("{} Settlements calculated", "✓".green()));

    // Print settlement summary
    println!();
    println!("{}", "Settlement Summary".cyan().bold());
    println!("  {} ${:.6}", "Total Volume:".bright_black(), settlements.total_volume);
    println!("  {} ${:.6}", "Miner Pool (75%):".bright_black(), settlements.miner_pool);
    println!("  {} ${:.6}", "Hive Ops (25%):".bright_black(), settlements.hive_ops);
    println!("  {} ${:.6}", "Dust → Hive:".bright_black(), settlements.dust_to_hive);
    println!();
    println!("  {}", "Provider Earnings:".bright_black());
    for (provider, amount) in &settlements.providers {
        println!("    {} ${:.6}", provider.green(), amount);
    }
    println!();

    // Build merkle root
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Building merkle tree...");
    pb.enable_steady_tick(Duration::from_millis(100));

    // TODO: Actual merkle tree construction from proof CIDs
    let merkle_root = crypto::keccak256_hash(
        format!("{}:{}:{}", target_epoch, proof_count, chrono::Utc::now().timestamp()).as_bytes()
    );

    pb.finish_with_message(format!("{} Merkle root: {}...", "✓".green(), &merkle_root[..18]));

    // Create sealed epoch snapshot
    let timestamp = chrono::Utc::now().timestamp();

    let mut epoch = EpochSnapshot {
        snapshot_type: "epoch-sealed".to_string(),
        version: "1.0.0".to_string(),
        epoch_id: target_epoch.clone(),
        name: generate_epoch_name(&target_epoch),
        status: "sealed".to_string(),
        started_at: timestamp - 3600, // Placeholder
        ended_at: Some(timestamp),
        jobs_count: proof_count as u64,
        total_volume_usdc: format!("{:.6}", settlements.total_volume),
        merkle_root: Some(merkle_root.clone()),
        settlements: Some(settlements.clone()),
        controller: "merlin.swarmos.eth".to_string(),
        timestamp,
        sig: None,
    };

    // Sign epoch
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Signing epoch seal...");
    pb.enable_steady_tick(Duration::from_millis(100));

    epoch.sig = Some(crypto::sign_snapshot(&epoch, &private_key).await?);
    pb.finish_with_message(format!("{} Epoch signed", "✓".green()));

    // Publish sealed epoch
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Publishing sealed epoch...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let epoch_cid = ipfs::write_epoch(&target_epoch, &epoch).await?;
    pb.finish_with_message(format!("{} Published: {}", "✓".green(), epoch_cid.cyan()));

    // Announce seal
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Announcing epoch seal...");
    pb.enable_steady_tick(Duration::from_millis(100));

    ipfs::pubsub_publish(
        &format!("/{}/epochs/sealed", pool),
        &serde_json::json!({
            "epoch_cid": epoch_cid,
            "epoch_id": target_epoch,
            "merkle_root": merkle_root,
            "jobs_count": proof_count,
            "total_volume": settlements.total_volume,
            "miner_pool": settlements.miner_pool,
            "hive_ops": settlements.hive_ops,
            "timestamp": timestamp
        }),
    )
    .await?;

    pb.finish_with_message(format!("{} Seal announced", "✓".green()));

    // Summary
    println!();
    println!("{}", "Epoch Sealed".green().bold());
    println!();
    println!("  {} {}", "Epoch:".bright_black(), target_epoch.cyan());
    println!("  {} {}", "CID:".bright_black(), epoch_cid);
    println!("  {} {}...", "Merkle Root:".bright_black(), &merkle_root[..18]);
    println!("  {} {}", "Jobs:".bright_black(), proof_count);
    println!("  {} ${:.2}", "Volume:".bright_black(), settlements.total_volume);
    println!("  {} ${:.2}", "Miner Pool:".bright_black(), settlements.miner_pool);
    println!("  {} ${:.2}", "Hive Ops:".bright_black(), settlements.hive_ops + settlements.dust_to_hive);
    println!();
    println!("  {}", "Provider balances now claimable via 'swarm withdraw'".yellow());
    println!();

    Ok(())
}

/// Mock proof data for epoch (in production: fetch from IPFS)
struct MockProof {
    job_id: String,
    provider: String,
    compute_seconds: f64,
    reward: f64,
    mode: ExecutionMode,
}

fn generate_mock_epoch_proofs() -> Vec<MockProof> {
    vec![
        // Job 1: SOLO - miner A wins
        MockProof {
            job_id: "job-001".to_string(),
            provider: "alpha.swarmbee.eth".to_string(),
            compute_seconds: 12.5,
            reward: 0.10,
            mode: ExecutionMode::Solo,
        },
        // Job 2: PPL - multiple miners contribute
        MockProof {
            job_id: "job-002".to_string(),
            provider: "alpha.swarmbee.eth".to_string(),
            compute_seconds: 40.0,
            reward: 0.10,
            mode: ExecutionMode::Ppl,
        },
        MockProof {
            job_id: "job-002".to_string(),
            provider: "beta.swarmbee.eth".to_string(),
            compute_seconds: 35.0,
            reward: 0.10,
            mode: ExecutionMode::Ppl,
        },
        MockProof {
            job_id: "job-002".to_string(),
            provider: "gamma.swarmbee.eth".to_string(),
            compute_seconds: 25.0,
            reward: 0.10,
            mode: ExecutionMode::Ppl,
        },
        // Job 3: SOLO - miner B wins
        MockProof {
            job_id: "job-003".to_string(),
            provider: "beta.swarmbee.eth".to_string(),
            compute_seconds: 8.2,
            reward: 0.10,
            mode: ExecutionMode::Solo,
        },
    ]
}

/// Calculate settlements for an epoch
///
/// Math:
/// - SOLO: winner gets R * 0.75
/// - PPL: each miner gets R * 0.75 * (their_compute / total_compute)
/// - Hive: always gets R * 0.25
/// - Dust: remainder → hive ops
fn calculate_settlements(proofs: &[MockProof], total_volume: f64) -> Settlements {
    let mut provider_earnings: HashMap<String, u64> = HashMap::new(); // microunits
    let mut total_hive_micro: u64 = 0;

    // Group proofs by job_id
    let mut jobs: HashMap<String, Vec<&MockProof>> = HashMap::new();
    for proof in proofs {
        jobs.entry(proof.job_id.clone()).or_default().push(proof);
    }

    // Process each job
    for (_job_id, job_proofs) in &jobs {
        if job_proofs.is_empty() {
            continue;
        }

        let first = job_proofs[0];
        let reward_micro = to_microunits(first.reward);
        let miner_pool_micro = (reward_micro as f64 * MINERS_PCT).floor() as u64;
        let hive_cut_micro = reward_micro - miner_pool_micro;

        total_hive_micro += hive_cut_micro;

        match first.mode {
            ExecutionMode::Solo => {
                // SOLO: First proof (winner) takes the miner pool
                // In production: validate this is the first valid proof
                let winner = &first.provider;
                *provider_earnings.entry(winner.clone()).or_insert(0) += miner_pool_micro;
            }
            ExecutionMode::Ppl => {
                // PPL: Proportional by compute_seconds
                let total_compute: f64 = job_proofs.iter().map(|p| p.compute_seconds).sum();

                if total_compute > 0.0 {
                    let mut distributed: u64 = 0;

                    for (i, proof) in job_proofs.iter().enumerate() {
                        let share = proof.compute_seconds / total_compute;
                        let payout_micro = if i == job_proofs.len() - 1 {
                            // Last miner gets remainder to avoid dust loss
                            miner_pool_micro - distributed
                        } else {
                            (miner_pool_micro as f64 * share).floor() as u64
                        };

                        *provider_earnings.entry(proof.provider.clone()).or_insert(0) += payout_micro;
                        distributed += payout_micro;
                    }
                }
            }
        }
    }

    // Convert back to USDC and calculate dust
    let providers: HashMap<String, f64> = provider_earnings
        .iter()
        .map(|(k, v)| (k.clone(), from_microunits(*v)))
        .collect();

    let miner_pool = providers.values().sum::<f64>();
    let hive_ops = from_microunits(total_hive_micro);

    // Dust = total_volume - miner_pool - hive_ops (should be ~0 or very small)
    let dust_to_hive = total_volume - miner_pool - hive_ops;

    Settlements {
        total_volume,
        miner_pool,
        hive_ops: hive_ops + dust_to_hive.max(0.0),
        providers,
        dust_to_hive: dust_to_hive.max(0.0),
    }
}

/// Generate NATO phonetic alphabet name for epoch
fn generate_epoch_name(epoch_id: &str) -> String {
    let nato = [
        "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf",
        "Hotel", "India", "Juliet", "Kilo", "Lima", "Mike", "November",
        "Oscar", "Papa", "Quebec", "Romeo", "Sierra", "Tango", "Uniform",
        "Victor", "Whiskey", "X-ray", "Yankee", "Zulu",
    ];

    // Extract number from epoch_id (e.g., "epoch-048" -> 48)
    let num: usize = epoch_id
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0);

    nato[num % nato.len()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solo_payout() {
        // SOLO: $0.10 job, winner takes $0.075
        let proofs = vec![
            MockProof {
                job_id: "job-001".to_string(),
                provider: "miner.eth".to_string(),
                compute_seconds: 10.0,
                reward: 0.10,
                mode: ExecutionMode::Solo,
            },
        ];

        let settlements = calculate_settlements(&proofs, 0.10);

        assert!((settlements.miner_pool - 0.075).abs() < 0.0001);
        assert!((settlements.hive_ops - 0.025).abs() < 0.001);
        assert_eq!(settlements.providers.get("miner.eth"), Some(&0.075));
    }

    #[test]
    fn test_ppl_payout() {
        // PPL: $0.10 job, split by compute_seconds
        // A: 40s, B: 35s, C: 25s (total 100s)
        let proofs = vec![
            MockProof {
                job_id: "job-001".to_string(),
                provider: "a.eth".to_string(),
                compute_seconds: 40.0,
                reward: 0.10,
                mode: ExecutionMode::Ppl,
            },
            MockProof {
                job_id: "job-001".to_string(),
                provider: "b.eth".to_string(),
                compute_seconds: 35.0,
                reward: 0.10,
                mode: ExecutionMode::Ppl,
            },
            MockProof {
                job_id: "job-001".to_string(),
                provider: "c.eth".to_string(),
                compute_seconds: 25.0,
                reward: 0.10,
                mode: ExecutionMode::Ppl,
            },
        ];

        let settlements = calculate_settlements(&proofs, 0.10);

        // A: 0.075 * 0.40 = 0.030
        // B: 0.075 * 0.35 = 0.02625
        // C: 0.075 * 0.25 = 0.01875
        let a = *settlements.providers.get("a.eth").unwrap_or(&0.0);
        let b = *settlements.providers.get("b.eth").unwrap_or(&0.0);
        let c = *settlements.providers.get("c.eth").unwrap_or(&0.0);

        assert!((a - 0.030).abs() < 0.001);
        assert!((b - 0.02625).abs() < 0.001);
        assert!((c - 0.01875).abs() < 0.001);
    }
}
