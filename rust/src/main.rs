mod cf_types;
mod client;
mod config;
mod errors;
mod metrics;
mod utils;

use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, fmt};

/// cloudflare-dyndns provides DynDNS functionality for Cloudflare.
#[derive(Parser)]
#[command(name = "cloudflare-dyndns", about = "cloudflare-dyndns provides DynDNS functionality for Cloudflare.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update DDNS Records by calling the Cloudflare API
    Client {
        /// Path to config file
        #[arg(short = 'c', long = "config", value_name = "FILE")]
        config: String,

        /// Expand environment variables in config file
        #[arg(long = "env", default_value_t = false)]
        env: bool,
    },
    /// Print version information and exit
    Version,
}

#[tokio::main]
async fn main() {
    // Set up a default tracing subscriber; log level may be adjusted after config is loaded.
    init_tracing("info");

    let cli = Cli::parse();

    match cli.command {
        Commands::Client { config, env } => run_client(&config, env).await,
        Commands::Version => print_version(),
    }
}

async fn run_client(config_path: &str, expand_env: bool) {
    let cfg = match config::load_config(config_path, expand_env) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(path = %config_path, err = %e, "Could not load configuration");
            std::process::exit(1);
        }
    };

    // Re-initialise tracing with the log level from the config file.
    init_tracing(&cfg.log_level);

    if let Err(e) = cfg.validate_client() {
        tracing::error!(err = %e, "Invalid configuration");
        std::process::exit(1);
    }

    metrics::init_metrics_and_serve(&cfg.metrics);

    let mut cf_client =
        match client::CloudflareClient::new(&cfg.client.token, cfg.client.proxy).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(err = %e, "Could not create Cloudflare client");
                std::process::exit(1);
            }
        };

    cf_client.data.domains = cfg.client.domains;

    client::run(cf_client, cfg.client.interval).await;
}

fn print_version() {
    let version = env!("CARGO_PKG_VERSION");
    println!("cloudflare-dyndns:");
    println!("    Version: {}", version);
    println!("    Commit:  Unknown");
}

fn init_tracing(level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    let _ = fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}
