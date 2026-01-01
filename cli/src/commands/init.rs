//! Init command - Initialize provider and register with pool

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::config::{self, Config};
use crate::crypto;
use crate::ipfs;
use crate::models::ProviderRegistration;
use crate::provider;

pub async fn execute(
    provider: String,
    wallet: String,
    gpus: Option<String>,
    key: Option<String>,
    pool: &str,
) -> Result<()> {
    println!("{}", "Initializing SwarmPool Provider".cyan().bold());
    println!();

    // Get private key
    let private_key = key
        .or_else(|| std::env::var("SWARM_PRIVATE_KEY").ok())
        .context("Private key required. Use --key or set SWARM_PRIVATE_KEY")?;

    // Detect or parse GPUs
    let gpu_list: Vec<String> = if let Some(gpus) = gpus {
        gpus.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        println!("  {} Detecting GPUs...", "⚡".yellow());
        let detected = provider::detect_gpus();
        detected.iter().map(|g| g.name.clone()).collect()
    };

    // Show init details
    println!("  {} {}", "Provider:".bright_black(), provider.green());
    println!("  {} {}", "Wallet:".bright_black(), wallet);
    println!("  {} {:?}", "GPUs:".bright_black(), gpu_list);
    println!("  {} {}", "Pool:".bright_black(), pool);
    println!();

    // Create registration snapshot
    let timestamp = chrono::Utc::now().timestamp();
    let nonce = crypto::random_hex(16);

    let mut registration = ProviderRegistration {
        snapshot_type: "provider-init".to_string(),
        provider: provider.clone(),
        wallet: wallet.clone(),
        gpus: gpu_list.clone(),
        models: vec!["queenbee-spine".to_string()],
        timestamp,
        nonce,
        sig: None,
    };

    // Sign registration
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Signing init snapshot...");
    pb.enable_steady_tick(Duration::from_millis(100));

    registration.sig = Some(crypto::sign_snapshot(&registration, &private_key).await?);
    pb.finish_with_message(format!("{} Snapshot signed", "✓".green()));

    // Write genesis to canonical IPFS path: /swarmpool/genesis/{provider}.json
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Publishing genesis to IPFS...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let init_cid = ipfs::write_genesis(&provider, &registration).await?;
    pb.finish_with_message(format!("{} Published: {}", "✓".green(), init_cid.cyan()));

    // Announce to pool
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Announcing to pool...");
    pb.enable_steady_tick(Duration::from_millis(100));

    ipfs::pubsub_publish(
        &format!("/{}/providers/init", pool),
        &serde_json::json!({
            "cid": init_cid,
            "provider": provider,
            "action": "init",
            "timestamp": timestamp
        }),
    )
    .await?;

    pb.finish_with_message(format!("{} Announced to {}", "✓".green(), pool));

    // Save config
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Saving configuration...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let config = Config {
        provider_ens: Some(provider.clone()),
        wallet: Some(wallet.clone()),
        gpus: gpu_list,
        models: vec!["queenbee-spine".to_string()],
        pool: pool.to_string(),
        ipfs_api: "http://localhost:5001".to_string(),
    };
    config::save_config(&config)?;

    pb.finish_with_message(format!("{} Config saved", "✓".green()));

    // Summary
    println!();
    println!("{}", "Provider Initialized".green().bold());
    println!();
    println!(
        "  {}",
        format!("ENS: {}", provider.cyan())
    );
    println!(
        "  {}",
        format!("CID: {}", init_cid.bright_black())
    );
    println!();
    println!("  {}", "Next:".yellow());
    println!("    {}", "swarm watch     # Watch for jobs".bright_black());
    println!("    {}", "swarm status    # Check status".bright_black());
    println!();

    Ok(())
}
