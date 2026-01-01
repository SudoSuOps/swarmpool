//! SwarmPool CLI - Decentralized Medical Compute Network
//!
//! A command-line tool for interacting with the SwarmPool network.
//! Submit jobs, run as a compute provider, check status, and manage earnings.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;
mod config;
mod crypto;
mod ipfs;
mod models;
mod provider;
mod schema;

use commands::{claim, epochs, init, prove, seal, status, submit, validate, watch, withdraw};

/// SwarmPool CLI - Decentralized Medical Compute Network
#[derive(Parser)]
#[command(name = "swarm")]
#[command(author = "SudoHash LLC")]
#[command(version = "0.2.0")]
#[command(about = "Decentralized medical AI inference network", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Pool ENS address
    #[arg(long, global = true, default_value = "swarmpool.eth")]
    pool: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize provider and register with pool (one-time genesis)
    Init {
        /// Provider ENS name (e.g., myprovider.swarmbee.eth)
        #[arg(long, env = "SWARM_PROVIDER_ENS")]
        provider: String,

        /// Wallet address for USDC payouts
        #[arg(long, env = "SWARM_WALLET")]
        wallet: String,

        /// GPU models (comma-separated, or auto-detect if omitted)
        #[arg(long)]
        gpus: Option<String>,

        /// Private key for signing
        #[arg(long, env = "SWARM_PRIVATE_KEY")]
        key: Option<String>,
    },

    /// Watch the pool for available jobs (read-only observation)
    Watch {
        /// Models to watch for (comma-separated)
        #[arg(long)]
        models: Option<String>,

        /// Provider ENS (if different from config)
        #[arg(long, env = "SWARM_PROVIDER_ENS")]
        provider: Option<String>,
    },

    /// Submit an inference job to the network (client action)
    Submit {
        /// Path to job JSON file
        #[arg(short, long)]
        file: Option<String>,

        /// Model to use (e.g., queenbee-spine)
        #[arg(long)]
        model: String,

        /// Input file path or IPFS CID
        #[arg(long)]
        input: String,

        /// Client ENS name
        #[arg(long, env = "SWARM_CLIENT_ENS")]
        client: String,

        /// Private key for signing (or use SWARM_PRIVATE_KEY env)
        #[arg(long, env = "SWARM_PRIVATE_KEY")]
        key: Option<String>,
    },

    /// Claim a job for execution (miner intent)
    Claim {
        /// Job CID to claim
        #[arg(long)]
        job: String,

        /// Execution mode: SOLO (winner takes all) or PPL (proportional payout)
        #[arg(long, default_value = "SOLO")]
        mode: String,

        /// Provider ENS (if different from config)
        #[arg(long, env = "SWARM_PROVIDER_ENS")]
        provider: Option<String>,

        /// Private key for signing
        #[arg(long, env = "SWARM_PRIVATE_KEY")]
        key: Option<String>,
    },

    /// Process a claimed job and submit proof of work
    Prove {
        /// Job CID to process
        #[arg(long)]
        job: String,

        /// Claim CID (optional, for verification)
        #[arg(long)]
        claim: Option<String>,

        /// Provider ENS (if different from config)
        #[arg(long, env = "SWARM_PROVIDER_ENS")]
        provider: Option<String>,

        /// Private key for signing
        #[arg(long, env = "SWARM_PRIVATE_KEY")]
        key: Option<String>,
    },

    /// Seal an epoch and calculate settlements (Merlin controller only)
    Seal {
        /// Epoch ID to seal (defaults to current active epoch)
        #[arg(long)]
        epoch: Option<String>,

        /// Private key for signing (must be Merlin's key)
        #[arg(long, env = "SWARM_PRIVATE_KEY")]
        key: Option<String>,
    },

    /// Check network or provider status
    Status {
        /// Provider ENS to check (optional, shows network stats if omitted)
        #[arg(long)]
        provider: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Withdraw earnings to wallet
    Withdraw {
        /// Amount to withdraw (or "all")
        #[arg(long)]
        amount: Option<String>,

        /// Provider ENS
        #[arg(long, env = "SWARM_PROVIDER_ENS")]
        provider: String,

        /// Private key for signing
        #[arg(long, env = "SWARM_PRIVATE_KEY")]
        key: Option<String>,
    },

    /// Show configuration
    Config {
        /// Show config file path
        #[arg(long)]
        path: bool,
    },

    /// List available models
    Models,

    /// Show epoch information
    Epochs {
        /// Specific epoch ID
        #[arg(long)]
        id: Option<String>,

        /// Number of recent epochs to show
        #[arg(long, default_value = "10")]
        limit: u32,
    },

    /// Validate a snapshot against its schema (debug tool)
    Validate {
        /// Path to JSON file to validate
        #[arg(long)]
        file: String,

        /// Schema type: genesis, job, claim, proof, epoch
        #[arg(long)]
        schema: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    // Print banner
    print_banner();

    match cli.command {
        Commands::Init {
            provider,
            wallet,
            gpus,
            key,
        } => {
            init::execute(provider, wallet, gpus, key, &cli.pool).await?;
        }

        Commands::Watch { models, provider } => {
            watch::execute(models, provider, &cli.pool).await?;
        }

        Commands::Submit {
            file,
            model,
            input,
            client,
            key,
        } => {
            submit::execute(file, model, input, client, key, &cli.pool).await?;
        }

        Commands::Claim { job, mode, provider, key } => {
            claim::execute(job, mode, provider, key, &cli.pool).await?;
        }

        Commands::Prove {
            job,
            claim,
            provider,
            key,
        } => {
            prove::execute(job, claim, provider, key, &cli.pool).await?;
        }

        Commands::Seal { epoch, key } => {
            seal::execute(epoch, key, &cli.pool).await?;
        }

        Commands::Status { provider, json } => {
            status::execute(provider, json, &cli.pool).await?;
        }

        Commands::Withdraw {
            amount,
            provider,
            key,
        } => {
            withdraw::execute(amount, provider, key, &cli.pool).await?;
        }

        Commands::Config { path } => {
            let config_path = config::get_config_path()?;
            if path {
                println!("{}", config_path.display());
            } else {
                println!("Config file: {}", config_path.display());
                if config_path.exists() {
                    let config = config::load_config()?;
                    println!("\n{}", serde_json::to_string_pretty(&config)?);
                } else {
                    println!("(not created yet - run 'swarm init' first)");
                }
            }
        }

        Commands::Models => {
            print_models();
        }

        Commands::Epochs { id, limit } => {
            epochs::execute(id, limit, &cli.pool).await?;
        }

        Commands::Validate { file, schema } => {
            validate::execute(file, schema).await?;
        }
    }

    Ok(())
}

fn print_banner() {
    let banner = r#"
   _____ _       __   ___    ____  __  ___
  / ___/| |     / /  /   |  / __ \/  |/  /
  \__ \ | | /| / /  / /| | / /_/ / /|_/ /
 ___/ / | |/ |/ /  / ___ |/ _, _/ /  / /
/____/  |__/|__/  /_/  |_/_/ |_/_/  /_/
                                           "#;

    println!("{}", banner.cyan());
    println!(
        "{}",
        "  Decentralized Medical Compute Network".bright_black()
    );
    println!(
        "{}",
        "  https://swarmpool.eth.limo".bright_black()
    );
    println!();
}

fn print_models() {
    println!("{}", "Available Models".cyan().bold());
    println!();

    let models = vec![
        ("queenbee-spine", "24 GB", "Lumbar MRI stenosis classification"),
        ("queenbee-chest", "24 GB", "Chest X-ray/CT analysis"),
        ("queenbee-foot", "16 GB", "Foot/ankle pathology detection"),
        ("queenbee-brain", "32 GB", "Brain MRI segmentation (Beta)"),
        ("queenbee-knee", "24 GB", "Knee MRI analysis (Beta)"),
    ];

    println!(
        "  {:<18} {:<10} {}",
        "Model".bright_black(),
        "VRAM".bright_black(),
        "Description".bright_black()
    );
    println!("  {}", "-".repeat(60).bright_black());

    for (name, vram, desc) in models {
        println!("  {:<18} {:<10} {}", name.green(), vram.yellow(), desc);
    }

    println!();
    println!(
        "  {}",
        "Use --models flag with 'swarm watch' to filter".bright_black()
    );
}
