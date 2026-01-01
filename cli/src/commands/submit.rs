//! Submit job command

use anyhow::{bail, Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::crypto;
use crate::ipfs;
use crate::models::{JobParams, JobSnapshot, Payment};

pub async fn execute(
    file: Option<String>,
    model: String,
    input: String,
    client: String,
    key: Option<String>,
    pool: &str,
) -> Result<()> {
    println!("{}", "Submitting job to SwarmPool".cyan().bold());
    println!();

    // Get private key
    let private_key = key
        .or_else(|| std::env::var("SWARM_PRIVATE_KEY").ok())
        .context("Private key required. Use --key or set SWARM_PRIVATE_KEY")?;

    // Show job details
    println!("  {} {}", "Model:".bright_black(), model.green());
    println!("  {} {}", "Input:".bright_black(), input);
    println!("  {} {}", "Client:".bright_black(), client);
    println!("  {} {}", "Pool:".bright_black(), pool);
    println!();

    // Upload input to IPFS if it's a file path
    let input_cid = if input.starts_with("bafy") || input.starts_with("Qm") {
        input.clone()
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message("Uploading input to IPFS...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let cid = ipfs::upload_file(&input).await?;
        pb.finish_with_message(format!("Uploaded: {}", cid.green()));
        cid
    };

    // Create job snapshot
    let job_id = format!(
        "job-{}-{}",
        chrono::Utc::now().format("%Y%m%d%H%M%S"),
        &crypto::random_hex(4)
    );

    let timestamp = chrono::Utc::now().timestamp();
    let nonce = crypto::random_hex(16);

    let mut job = JobSnapshot {
        snapshot_type: "job".to_string(),
        version: "1.0.0".to_string(),
        job_id: job_id.clone(),
        job_type: format!("{}-inference", model),
        model: model.clone(),
        input_cid,
        params: JobParams {
            confidence_threshold: 0.6,
            output_format: "pdf".to_string(),
        },
        payment: Payment {
            amount: "0.10".to_string(),
            token: "USDC".to_string(),
        },
        client: client.clone(),
        timestamp,
        nonce,
        sig: None,
    };

    // Sign job
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Signing job...");
    pb.enable_steady_tick(Duration::from_millis(100));

    job.sig = Some(crypto::sign_snapshot(&job, &private_key)?);
    pb.finish_with_message(format!("{} Job signed", "✓".green()));

    // Write job to canonical IPFS path: /swarmpool/jobs/{job_id}.json
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Publishing to IPFS mempool...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let job_cid = ipfs::write_job(&job_id, &job).await?;
    pb.finish_with_message(format!("{} Published: {}", "✓".green(), job_cid.cyan()));

    // Announce to pool (via IPFS pubsub or Redis signal)
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Announcing to pool...");
    pb.enable_steady_tick(Duration::from_millis(100));

    ipfs::pubsub_publish(
        &format!("/{}/jobs", pool),
        &serde_json::json!({
            "cid": job_cid,
            "client": client,
            "model": model,
            "timestamp": timestamp
        }),
    )
    .await?;

    pb.finish_with_message(format!("{} Announced to {}", "✓".green(), pool));

    // Summary
    println!();
    println!("{}", "Job Submitted Successfully".green().bold());
    println!();
    println!("  {} {}", "Job ID:".bright_black(), job_id.cyan());
    println!("  {} {}", "CID:".bright_black(), job_cid);
    println!();
    println!(
        "  {}",
        "Waiting for a compute provider to process...".bright_black()
    );
    println!(
        "  {}",
        format!("Check status: swarm status --job {}", job_cid).bright_black()
    );

    Ok(())
}
