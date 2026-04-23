#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
use anyhow::{Context, Error, Result};
use clap::{Args, Parser, Subcommand};

use crate::dyndns::DynDnsClient;

mod client;
mod config;
mod dyndns;
mod relay;
mod runtime;
mod server;
mod utils;
mod version;

fn main() {
    if let Err(e) = Cli::parse().run() {
        exit_with_error(e);
    }
}

#[derive(Parser)]
#[command(name = "cloudflare-dyndns")]
struct Cli {
    /// The subcommand to run
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Command::Server { cli_opts } => {
                let config_path = if let Some(path) = &cli_opts.config {
                    path
                } else {
                    ""
                };
                let config =
                    config::Config::from_file(config_path, config::Mode::Server, cli_opts.env)?;

                let mut server = server::Server::from_config(config.server);
                let rt =
                    runtime::multi_threaded_runtime().context("Failed to create tokio runtime")?;
                rt.block_on(async {
                    if let Err(e) = server.run(None).await {
                        eprintln!("Error: {e:#}");
                        std::process::exit(1);
                    }
                })
            }
            Command::Client { cli_opts } => {
                let rt =
                    runtime::single_threaded_runtime().context("Failed to create tokio runtime")?;

                let config = config::Config::from_file(
                    &cli_opts.config,
                    config::Mode::Client,
                    cli_opts.env,
                )?;

                let interval = config.client.interval;
                let mut client = client::Client::from_config(config.client);

                rt.block_on(async move {
                    let stop_rx = utils::new_interrupt_signal(None);
                    client.run(interval, stop_rx).await;
                })
            }
            Command::Relay { cli_opts } => {
                let rt =
                    runtime::single_threaded_runtime().context("Failed to create tokio runtime")?;

                let config =
                    config::Config::from_file(&cli_opts.config, config::Mode::Relay, cli_opts.env)?;

                let interval = config.client.interval;
                let mut relay = relay::Relay::from_config(config.client);
                rt.block_on(async move {
                    let stop_rx = utils::new_interrupt_signal(None);
                    relay.run(interval, stop_rx).await;
                })
            }
            Command::Version => {
                version::print_version_and_exit();
            }
        }
        Ok(())
    }
}

/// cloudflare-dyndns provides DynDNS functionality for Cloudflare.
#[derive(Subcommand)]
enum Command {
    /// Run a server for relay clients
    Server {
        #[clap(flatten)]
        cli_opts: ServerOptions,
    },
    /// Update DDNS Records by calling the Cloudflare API
    Client {
        #[clap(flatten)]
        cli_opts: ClientOptions,
    },
    /// Update DDNS Records but relay the calls through a server
    Relay {
        #[clap(flatten)]
        cli_opts: ClientOptions,
    },
    /// Print version information and exit
    Version,
}

/// Flags used for the relay and client subcommands
#[derive(Args)]
struct ClientOptions {
    /// Path to config file
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    config: String,

    /// Expand environment variables in config file
    #[arg(long = "env", default_value_t = false)]
    env: bool,
}

/// Flags used for server subcommand
#[derive(Args)]
struct ServerOptions {
    /// Path to config file
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    config: Option<String>,

    /// Expand environment variables in config file
    #[arg(long = "env", default_value_t = false)]
    env: bool,
}

/// Exit the program with an error message and a non-zero exit code.
fn exit_with_error(e: Error) -> ! {
    eprintln!("Error: {e:#}");
    std::process::exit(1);
}
