use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::Level;
use tracing_subscriber::filter::EnvFilter;

#[cfg(test)]
mod test;

const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_SERVER_PORT: u16 = 8080;
const DEFAULT_CLIENT_INTERVAL: u64 = 300; // 5 minutes in seconds

#[derive(PartialEq)]
pub enum Mode {
    Server,
    Client,
    Relay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    /// Log level of the application
    pub log_level: String,
    /// Config for running in server mode
    pub server: ServerConfig,
    /// Config for running in client/relay mode
    pub client: ClientConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Port to listen on. Default: 8080
    pub port: u16,
    /// List of root domains that are allowed to be updated. Allows all when empty.
    pub domains: Vec<String>,
    /// SSL config, default is no ssl
    pub ssl: SSLConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SSLConfig {
    /// Enable SSL. Default: false
    pub enabled: bool,
    /// SSL certificate, needs to contain the whole chain
    pub cert: String,
    /// SSL private key
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClientConfig {
    /// Token for accessing the cloudflare api
    pub token: String,
    /// Indicate if entries should be proxied by cloudflare. Default: true
    pub proxy: bool,
    /// List of domains to update
    pub domains: Vec<String>,
    /// Interval in seconds in which the client should check for ip changes. Default: 300 (5m)
    pub interval: u64,
    /// Endpoint to call when using relay mode
    pub endpoint: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            server: ServerConfig::default(),
            client: ClientConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_SERVER_PORT,
            domains: Vec::new(),
            ssl: SSLConfig::default(),
        }
    }
}

impl Default for SSLConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert: String::new(),
            key: String::new(),
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            proxy: true,
            domains: vec![],
            interval: DEFAULT_CLIENT_INTERVAL,
            endpoint: String::new(),
        }
    }
}

impl Config {
    /// Loads config from file, returns error if config is invalid
    /// Arguments:
    ///
    ///	path: Path to config file
    ///	mode: Mode used, determines how the config will be validated and which values will be processed
    ///	env: Determines if environment variables in the file will be expanded before decoding
    pub fn from_file(path: &str, mode: Mode, expand_env: bool) -> Result<Self> {
        if path.is_empty() && mode == Mode::Server {
            set_log_level(DEFAULT_LOG_LEVEL)?;
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .context(format!("Failed to read config file at '{}'", path))?;

        let content = if expand_env {
            shellexpand::env(&content)
                .context("Failed to expand environment variables in config file")?
                .to_string()
        } else {
            content
        };

        let config: Self =
            serde_yaml::from_str(&content).context("Failed to parse config file.")?;

        set_log_level(&config.log_level)?;

        match mode {
            Mode::Server => config.server.validate()?,
            Mode::Client => {
                config.client.validate()?;
            }
            Mode::Relay => {
                config.client.validate()?;
                if config.client.endpoint.is_empty() {
                    bail!("Client endpoint cannot be empty");
                }
            }
        }

        Ok(config)
    }
}

impl ServerConfig {
    /// Validates the server config, returns error if config is invalid
    fn validate(&self) -> Result<()> {
        if self.ssl.enabled {
            if self.ssl.cert.is_empty() {
                bail!("SSL is enabled but cert is empty");
            }
            if self.ssl.key.is_empty() {
                bail!("SSL is enabled but key is empty");
            }
        }
        Ok(())
    }
}

impl ClientConfig {
    /// Validate the client part of the config
    fn validate(&self) -> Result<()> {
        if self.token.is_empty() {
            bail!("Client token cannot be empty");
        }
        if self.domains.is_empty() {
            bail!("Client domains cannot be empty");
        }
        if self.interval < 30 {
            bail!("Client interval cannot be less than 30 seconds");
        }
        Ok(())
    }
}

/// Parse a given string and set the resulting log level
fn set_log_level(level: &str) -> Result<()> {
    let level = match level.to_lowercase().as_str() {
        "error" => Level::ERROR,
        "warn" => Level::WARN,
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        _ => bail!(format!("Invalid log level: {level}.")),
    };
    let filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env()?
        .add_directive(format!("cloudflare_dyndns={level}").parse()?);
    let logger = tracing_subscriber::fmt().with_env_filter(filter);
    #[cfg(not(test))]
    logger.init();

    // We can only initialize the logger once, but testing might call the parent function multiple times.
    #[cfg(test)]
    logger.try_init().unwrap_or_default();

    Ok(())
}
