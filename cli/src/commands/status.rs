//! Status command - check network or provider status

use anyhow::Result;
use colored::Colorize;

use crate::ipfs;
use crate::models::{NetworkStats, PoolState, ProviderInfo};

pub async fn execute(provider: Option<String>, json: bool, pool: &str) -> Result<()> {
    if let Some(provider_ens) = provider {
        // Show specific provider status
        show_provider_status(&provider_ens, json, pool).await
    } else {
        // Show network status
        show_network_status(json, pool).await
    }
}

async fn show_network_status(json: bool, pool: &str) -> Result<()> {
    // Fetch pool state from IPFS
    let state = ipfs::fetch_pool_state(pool).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&state)?);
        return Ok(());
    }

    println!("{}", "SwarmPool Network Status".cyan().bold());
    println!();

    // Network stats
    println!("  {}", "Network".bright_black());
    println!("  {}", "━".repeat(40).bright_black());
    println!(
        "    {} {}",
        "Total Jobs:".bright_black(),
        state.total_jobs.to_string().green()
    );
    println!(
        "    {} {}",
        "Total Volume:".bright_black(),
        format!("${:.2}", state.total_volume_usdc).green()
    );
    println!(
        "    {} {}",
        "Active Providers:".bright_black(),
        state.active_providers.len()
    );
    println!();

    // Current epoch
    if let Some(epoch) = &state.current_epoch {
        println!("  {}", "Current Epoch".bright_black());
        println!("  {}", "━".repeat(40).bright_black());
        println!("    {} {}", "ID:".bright_black(), epoch.cyan());
        println!(
            "    {} {}",
            "Jobs:".bright_black(),
            state.epoch_jobs
        );
        println!(
            "    {} {}",
            "Volume:".bright_black(),
            format!("${:.2}", state.epoch_volume)
        );
        println!();
    }

    // Pending jobs
    println!("  {}", "Queue".bright_black());
    println!("  {}", "━".repeat(40).bright_black());
    println!(
        "    {} {}",
        "Pending Jobs:".bright_black(),
        state.pending_jobs.len()
    );
    println!();

    // Top providers
    if !state.active_providers.is_empty() {
        println!("  {}", "Active Providers".bright_black());
        println!("  {}", "━".repeat(40).bright_black());

        let mut providers: Vec<_> = state.active_providers.values().collect();
        providers.sort_by(|a, b| b.jobs_completed.cmp(&a.jobs_completed));

        for (i, p) in providers.iter().take(5).enumerate() {
            let status_icon = match p.status.as_str() {
                "online" => "🟢",
                "busy" => "🟡",
                _ => "🔴",
            };
            println!(
                "    {} {} {} ({} jobs)",
                status_icon,
                truncate_ens(&p.ens, 24),
                format!("${:.2}", p.total_earnings).green(),
                p.jobs_completed
            );
        }
    }

    Ok(())
}

async fn show_provider_status(provider_ens: &str, json: bool, pool: &str) -> Result<()> {
    let state = ipfs::fetch_pool_state(pool).await?;

    let provider = state
        .active_providers
        .get(provider_ens)
        .ok_or_else(|| anyhow::anyhow!("Provider not found: {}", provider_ens))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&provider)?);
        return Ok(());
    }

    let status_icon = match provider.status.as_str() {
        "online" => "🟢",
        "busy" => "🟡",
        _ => "🔴",
    };

    println!("{}", "Provider Status".cyan().bold());
    println!();
    println!(
        "  {} {} {}",
        status_icon,
        provider.ens.green(),
        format!("({})", provider.status).bright_black()
    );
    println!();

    println!("  {}", "Stats".bright_black());
    println!("  {}", "━".repeat(40).bright_black());
    println!(
        "    {} {}",
        "Jobs Completed:".bright_black(),
        provider.jobs_completed.to_string().green()
    );
    println!(
        "    {} {}",
        "Total Earnings:".bright_black(),
        format!("${:.2}", provider.total_earnings).green()
    );
    println!(
        "    {} {}",
        "Available Balance:".bright_black(),
        format!("${:.2}", provider.available_balance).yellow()
    );
    println!();

    println!("  {}", "Hardware".bright_black());
    println!("  {}", "━".repeat(40).bright_black());
    println!("    {} {:?}", "GPUs:".bright_black(), provider.gpus);
    println!("    {} {:?}", "Models:".bright_black(), provider.models);
    println!();

    println!("  {}", "Wallet".bright_black());
    println!("  {}", "━".repeat(40).bright_black());
    println!("    {} {}", "Address:".bright_black(), provider.wallet);

    Ok(())
}

fn truncate_ens(ens: &str, max_len: usize) -> String {
    if ens.len() <= max_len {
        ens.to_string()
    } else {
        format!("{}...{}", &ens[..8], &ens[ens.len() - 8..])
    }
}
