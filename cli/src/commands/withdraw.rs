//! Withdraw command - withdraw earnings to wallet

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::crypto;
use crate::ipfs;

pub async fn execute(
    amount: Option<String>,
    provider: String,
    key: Option<String>,
    pool: &str,
) -> Result<()> {
    println!("{}", "Withdraw Earnings".cyan().bold());
    println!();

    // Get private key
    let private_key = key
        .or_else(|| std::env::var("SWARM_PRIVATE_KEY").ok())
        .context("Private key required. Use --key or set SWARM_PRIVATE_KEY")?;

    // Fetch current balance
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Fetching balance...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let state = ipfs::fetch_pool_state(pool).await?;
    let provider_info = state
        .active_providers
        .get(&provider)
        .context("Provider not found")?;

    let available = provider_info.available_balance;
    pb.finish_with_message(format!(
        "{} Available: {}",
        "✓".green(),
        format!("${:.2}", available).green()
    ));

    if available <= 0.0 {
        println!();
        println!("{}", "No balance available to withdraw".yellow());
        return Ok(());
    }

    // Determine withdrawal amount
    let withdraw_amount = match amount.as_deref() {
        Some("all") | None => available,
        Some(amt) => amt.parse::<f64>().context("Invalid amount")?,
    };

    if withdraw_amount > available {
        println!();
        println!(
            "{} Requested ${:.2} but only ${:.2} available",
            "⚠️".yellow(),
            withdraw_amount,
            available
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} ${:.2}",
        "Withdrawing:".bright_black(),
        withdraw_amount
    );
    println!(
        "  {} {}",
        "To wallet:".bright_black(),
        &provider_info.wallet
    );
    println!();

    // Create withdrawal request
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Creating withdrawal request...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let timestamp = chrono::Utc::now().timestamp();
    let nonce = crypto::random_hex(16);

    let withdrawal = serde_json::json!({
        "type": "withdrawal",
        "provider": provider,
        "amount": format!("{:.2}", withdraw_amount),
        "wallet": provider_info.wallet,
        "timestamp": timestamp,
        "nonce": nonce,
    });

    let signed = crypto::sign_json(&withdrawal, &private_key)?;
    pb.finish_with_message(format!("{} Request signed", "✓".green()));

    // Submit withdrawal
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Submitting withdrawal...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let cid = ipfs::upload_json(&signed).await?;

    ipfs::pubsub_publish(
        &format!("/{}/withdrawals", pool),
        &serde_json::json!({
            "cid": cid,
            "provider": provider,
            "amount": withdraw_amount,
            "timestamp": timestamp
        }),
    )
    .await?;

    pb.finish_with_message(format!("{} Withdrawal submitted", "✓".green()));

    // Summary
    println!();
    println!("{}", "✅ Withdrawal Requested".green().bold());
    println!();
    println!(
        "  {} {}",
        "Amount:".bright_black(),
        format!("${:.2} USDC", withdraw_amount).green()
    );
    println!("  {} {}", "To:".bright_black(), provider_info.wallet);
    println!("  {} {}", "CID:".bright_black(), cid.cyan());
    println!();
    println!(
        "  {}",
        "Withdrawal will be processed in the next epoch settlement.".bright_black()
    );

    Ok(())
}
