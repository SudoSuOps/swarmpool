//! Claim command - Claim a job from the mempool
//!
//! Miners choose execution mode:
//! - SOLO: Winner takes full job reward (first valid proof wins)
//! - PPL: Pay-Per-Load, proportional payout based on compute_seconds

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::config;
use crate::crypto;
use crate::ipfs;
use crate::models::{ClaimSnapshot, ExecutionMode, JobSnapshot};

pub async fn execute(
    job_cid: String,
    mode: String,
    provider_override: Option<String>,
    key: Option<String>,
    pool: &str,
) -> Result<()> {
    // Load config
    let config = config::load_config()?;

    let provider_ens = provider_override
        .or(config.provider_ens)
        .context("Provider ENS required. Run 'swarm init' first or use --provider")?;

    let private_key = key
        .or_else(|| std::env::var("SWARM_PRIVATE_KEY").ok())
        .context("Private key required. Use --key or set SWARM_PRIVATE_KEY")?;

    // Parse execution mode
    let exec_mode: ExecutionMode = mode.parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    println!("{}", "Claiming Job".cyan().bold());
    println!();
    println!("  {} {}", "Job CID:".bright_black(), job_cid.cyan());
    println!("  {} {}", "Provider:".bright_black(), provider_ens.green());
    println!("  {} {}", "Mode:".bright_black(), format_mode(&exec_mode));
    println!();

    // Fetch job details
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Fetching job from IPFS...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let job: JobSnapshot = ipfs::fetch_json(&job_cid).await?;
    pb.finish_with_message(format!("{} Job fetched", "✓".green()));

    // Show job details
    println!("  {} {}", "Model:".bright_black(), job.model.green());
    println!("  {} {}", "Client:".bright_black(), job.client);
    println!("  {} {}", "Payment:".bright_black(), format!("{} {}", job.payment.amount, job.payment.token).yellow());
    println!();

    // Create claim snapshot
    let timestamp = chrono::Utc::now().timestamp();
    let nonce = crypto::random_hex(16);
    let claim_id = format!(
        "claim-{}-{}",
        chrono::Utc::now().format("%Y%m%d%H%M%S"),
        &crypto::random_hex(4)
    );

    let mut claim = ClaimSnapshot {
        snapshot_type: "claim".to_string(),
        version: "1.0.0".to_string(),
        claim_id: claim_id.clone(),
        job_id: job.job_id.clone(),
        job_cid: job_cid.clone(),
        provider: provider_ens.clone(),
        mode: exec_mode,
        timestamp,
        nonce,
        sig: None,
    };

    // Sign claim
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Signing claim...");
    pb.enable_steady_tick(Duration::from_millis(100));

    claim.sig = Some(crypto::sign_snapshot(&claim, &private_key).await?);
    pb.finish_with_message(format!("{} Claim signed", "✓".green()));

    // Publish claim to IPFS
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Publishing claim...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let claim_cid = ipfs::write_claim(&claim_id, &claim).await?;
    pb.finish_with_message(format!("{} Published: {}", "✓".green(), claim_cid.cyan()));

    // Announce claim to pool
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Announcing claim to pool...");
    pb.enable_steady_tick(Duration::from_millis(100));

    ipfs::pubsub_publish(
        &format!("/{}/claims", pool),
        &serde_json::json!({
            "claim_cid": claim_cid,
            "claim_id": claim_id,
            "job_cid": job_cid,
            "job_id": job.job_id,
            "provider": provider_ens,
            "mode": exec_mode.to_string(),
            "timestamp": timestamp
        }),
    )
    .await?;

    pb.finish_with_message(format!("{} Claim announced", "✓".green()));

    // Summary
    println!();
    println!("{}", "Job Claimed".green().bold());
    println!();
    println!("  {} {}", "Claim ID:".bright_black(), claim_id.cyan());
    println!("  {} {}", "Claim CID:".bright_black(), claim_cid);
    println!("  {} {}", "Job ID:".bright_black(), job.job_id);
    println!("  {} {}", "Mode:".bright_black(), format_mode(&exec_mode));
    println!();

    // Mode-specific messaging
    match exec_mode {
        ExecutionMode::Solo => {
            println!("  {}", "SOLO: First valid proof wins the full reward".yellow());
        }
        ExecutionMode::Ppl => {
            println!("  {}", "PPL: Reward proportional to compute_seconds contributed".yellow());
        }
    }

    println!();
    println!("  {}", "Next:".bright_black());
    println!(
        "    {}",
        format!("swarm prove --job {} --claim {}", job_cid, claim_cid).cyan()
    );
    println!();

    Ok(())
}

fn format_mode(mode: &ExecutionMode) -> colored::ColoredString {
    match mode {
        ExecutionMode::Solo => "SOLO".yellow().bold(),
        ExecutionMode::Ppl => "PPL".blue().bold(),
    }
}
