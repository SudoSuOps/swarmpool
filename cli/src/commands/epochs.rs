//! Epochs command - view epoch history

use anyhow::Result;
use colored::Colorize;

use crate::ipfs;

pub async fn execute(id: Option<String>, limit: u32, pool: &str) -> Result<()> {
    if let Some(epoch_id) = id {
        show_epoch_detail(&epoch_id, pool).await
    } else {
        show_epoch_list(limit, pool).await
    }
}

async fn show_epoch_list(limit: u32, pool: &str) -> Result<()> {
    println!("{}", "Epoch History".cyan().bold());
    println!();

    let epochs = ipfs::fetch_epochs(pool, limit).await?;

    if epochs.is_empty() {
        println!("  {}", "No epochs found".bright_black());
        return Ok(());
    }

    // Header
    println!(
        "  {:<12} {:<12} {:<8} {:<12} {:<10}",
        "Epoch".bright_black(),
        "Name".bright_black(),
        "Jobs".bright_black(),
        "Volume".bright_black(),
        "Status".bright_black()
    );
    println!("  {}", "━".repeat(60).bright_black());

    for epoch in epochs {
        let status = match epoch.status.as_str() {
            "active" => "🟢 Active".green().to_string(),
            "sealed" => "✅ Sealed".bright_black().to_string(),
            _ => epoch.status.clone(),
        };

        println!(
            "  {:<12} {:<12} {:<8} {:<12} {}",
            epoch.epoch_id.cyan(),
            epoch.name,
            epoch.jobs_count,
            format!("${}", epoch.total_volume_usdc).green(),
            status
        );
    }

    println!();
    println!(
        "  {}",
        format!("View details: swarm epochs --id <epoch_id>").bright_black()
    );

    Ok(())
}

async fn show_epoch_detail(epoch_id: &str, pool: &str) -> Result<()> {
    let epoch = ipfs::fetch_epoch(pool, epoch_id).await?;

    println!("{}", format!("Epoch: {}", epoch_id).cyan().bold());
    println!();

    // Status badge
    let status_badge = match epoch.status.as_str() {
        "active" => "🟢 ACTIVE".green(),
        "sealed" => "✅ SEALED".cyan(),
        _ => epoch.status.normal(),
    };
    println!("  {} {}", "Status:".bright_black(), status_badge);
    println!("  {} {}", "Name:".bright_black(), epoch.name);
    println!();

    // Timing
    println!("  {}", "Timing".bright_black());
    println!("  {}", "━".repeat(40).bright_black());

    let started = chrono::DateTime::from_timestamp(epoch.started_at, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    println!("    {} {}", "Started:".bright_black(), started);

    if let Some(ended_at) = epoch.ended_at {
        let ended = chrono::DateTime::from_timestamp(ended_at, 0)
            .map(|d| d.format("%Y-%m-%d %H:%M UTC").to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        println!("    {} {}", "Ended:".bright_black(), ended);

        let duration = ended_at - epoch.started_at;
        let hours = duration / 3600;
        let minutes = (duration % 3600) / 60;
        println!(
            "    {} {}h {}m",
            "Duration:".bright_black(),
            hours,
            minutes
        );
    }
    println!();

    // Stats
    println!("  {}", "Stats".bright_black());
    println!("  {}", "━".repeat(40).bright_black());
    println!(
        "    {} {}",
        "Jobs:".bright_black(),
        epoch.jobs_count.to_string().green()
    );
    println!(
        "    {} {}",
        "Volume:".bright_black(),
        format!("${}", epoch.total_volume_usdc).green()
    );

    if let Some(merkle_root) = &epoch.merkle_root {
        println!(
            "    {} {}",
            "Merkle Root:".bright_black(),
            &merkle_root[..18].cyan()
        );
    }
    println!();

    // Settlements
    if let Some(settlements) = &epoch.settlements {
        println!("  {}", "Settlements".bright_black());
        println!("  {}", "━".repeat(40).bright_black());
        println!(
            "    {} {}",
            "Miner Pool (75%):".bright_black(),
            format!("${:.2}", settlements.miner_pool).green()
        );
        println!(
            "    {} {}",
            "Hive Ops (25%):".bright_black(),
            format!("${:.2}", settlements.hive_ops)
        );
        println!();

        if !settlements.providers.is_empty() {
            println!("  {}", "Provider Payouts".bright_black());
            println!("  {}", "━".repeat(40).bright_black());

            let mut providers: Vec<_> = settlements.providers.iter().collect();
            providers.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

            for (ens, amount) in providers {
                println!(
                    "    {} {}",
                    truncate_ens(ens, 28),
                    format!("${:.2}", amount).green()
                );
            }
        }
    }

    Ok(())
}

fn truncate_ens(ens: &str, max_len: usize) -> String {
    if ens.len() <= max_len {
        format!("{:<width$}", ens, width = max_len)
    } else {
        format!("{}...{}", &ens[..10], &ens[ens.len() - 10..])
    }
}
