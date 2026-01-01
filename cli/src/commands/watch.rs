//! Watch command - Subscribe to job feed and monitor for claimable jobs

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tokio::signal;

use crate::config;
use crate::ipfs;

pub async fn execute(
    models: Option<String>,
    provider_override: Option<String>,
    pool: &str,
) -> Result<()> {
    // Load config
    let config = config::load_config()?;

    let provider_ens = provider_override
        .or(config.provider_ens)
        .context("Provider ENS required. Run 'swarm init' first or use --provider")?;

    let model_list: Vec<String> = models
        .map(|m| m.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_else(|| config.models.clone());

    // Print startup banner
    println!("{}", "SwarmPool Job Watcher".cyan().bold());
    println!();
    println!("  {} {}", "Provider:".bright_black(), provider_ens.green());
    println!("  {} {}", "Pool:".bright_black(), pool);
    println!("  {} {:?}", "Models:".bright_black(), model_list);
    println!();

    // Connect to IPFS
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    pb.set_message("Connecting to IPFS...");
    pb.enable_steady_tick(Duration::from_millis(100));
    ipfs::check_connection().await?;
    pb.finish_with_message(format!("{} Connected to IPFS", "✓".green()));

    // Subscribe to job feed
    pb.set_message("Subscribing to job feed...");
    pb.enable_steady_tick(Duration::from_millis(100));
    ipfs::pubsub_subscribe(&format!("/{}/jobs", pool)).await?;
    pb.finish_with_message(format!("{} Subscribed to /{}/jobs", "✓".green(), pool));

    // Print ready message
    println!();
    println!("{}", "━".repeat(60).bright_black());
    println!();
    println!(
        "  {} {}",
        "👁️ ".cyan(),
        "Watching for Jobs".cyan().bold()
    );
    println!(
        "  {}",
        "Press Ctrl+C to stop".bright_black()
    );
    println!();
    println!("{}", "━".repeat(60).bright_black());
    println!();

    // Watch loop
    let mut jobs_seen: u64 = 0;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!();
                println!("{}", "Stopping watcher...".yellow());
                break;
            }

            // Poll for new jobs (in production: SSE stream from IPFS pubsub)
            _ = tokio::time::sleep(Duration::from_secs(2)) => {
                // In production: check actual pubsub messages
                // For now: poll pending jobs from pool state
                match ipfs::fetch_pool_state(pool).await {
                    Ok(state) => {
                        for job_cid in &state.pending_jobs {
                            jobs_seen += 1;
                            println!(
                                "  {} Job available: {}",
                                "📋".yellow(),
                                job_cid.cyan()
                            );
                            println!(
                                "       {}",
                                format!("Claim with: swarm claim --job {}", job_cid).bright_black()
                            );
                        }
                    }
                    Err(e) => {
                        // Silent retry on error
                        tracing::debug!("Error fetching pool state: {}", e);
                    }
                }
            }

            // Send heartbeat every 30 seconds
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                let timestamp = chrono::Utc::now().timestamp();
                let heartbeat = serde_json::json!({
                    "provider": provider_ens,
                    "status": "watching",
                    "models": model_list,
                    "timestamp": timestamp
                });

                if let Err(e) = ipfs::pubsub_publish(
                    &format!("/{}/heartbeats", pool),
                    &heartbeat
                ).await {
                    eprintln!("  {} Heartbeat failed: {}", "⚠️".yellow(), e);
                }
            }
        }
    }

    // Summary
    println!();
    println!("{}", "Watch Session Summary".cyan().bold());
    println!("  {} {}", "Jobs Seen:".bright_black(), jobs_seen);
    println!();

    Ok(())
}
