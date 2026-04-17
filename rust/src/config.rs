use std::time::Duration;

use serde::{Deserialize, Serialize};

use humantime_serde::re::humantime;

use crate::errors::DynDnsError;

pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const DEFAULT_SERVER_PORT: u16 = 8080;
pub const DEFAULT_CLIENT_INTERVAL: Duration = Duration::from_secs(5 * 60);
pub const DEFAULT_METRICS_PORT: u16 = 9090;

fn default_log_level() -> String {
    DEFAULT_LOG_LEVEL.to_string()
}

fn default_server_port() -> u16 {
    DEFAULT_SERVER_PORT
}

fn default_client_proxy() -> bool {
    true
}

fn default_client_interval() -> Duration {
    DEFAULT_CLIENT_INTERVAL
}

fn default_metrics_port() -> u16 {
    DEFAULT_METRICS_PORT
}

fn default_go_collector() -> bool {
    true
}

fn default_process_collector() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "logLevel", default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub client: ClientConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_port")]
    pub port: u16,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub ssl: SslConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SslConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub cert: String,
    #[serde(default)]
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientConfig {
    #[serde(default)]
    pub token: String,
    #[serde(default = "default_client_proxy")]
    pub proxy: bool,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(
        default = "default_client_interval",
        with = "humantime_serde"
    )]
    pub interval: Duration,
    #[serde(default)]
    pub endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_metrics_port")]
    pub port: u16,
    #[serde(rename = "goCollector", default = "default_go_collector")]
    pub go_collector: bool,
    #[serde(rename = "processCollector", default = "default_process_collector")]
    pub process_collector: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            server: ServerConfig::default(),
            client: ClientConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            port: DEFAULT_SERVER_PORT,
            domains: Vec::new(),
            ssl: SslConfig::default(),
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            token: String::new(),
            proxy: true,
            domains: Vec::new(),
            interval: DEFAULT_CLIENT_INTERVAL,
            endpoint: String::new(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        MetricsConfig {
            enabled: false,
            port: DEFAULT_METRICS_PORT,
            go_collector: true,
            process_collector: true,
        }
    }
}

impl Config {
    /// Validate config for client mode
    pub fn validate_client(&self) -> Result<(), DynDnsError> {
        if self.client.token.is_empty() {
            return Err(DynDnsError::MissingToken);
        }
        if self.client.domains.is_empty() {
            return Err(DynDnsError::NoDomain);
        }
        if self.client.interval < Duration::from_secs(30) {
            return Err(DynDnsError::InvalidInterval(
                humantime::format_duration(self.client.interval).to_string(),
            ));
        }

        tracing::info!(
            proxy = self.client.proxy,
            domains = ?self.client.domains,
            interval = %humantime::format_duration(self.client.interval),
            endpoint = %self.client.endpoint,
            "Loaded client config"
        );

        Ok(())
    }
}

/// Load config from file, validate for client mode.
/// If `expand_env` is true, environment variable references in the file are substituted before
/// parsing.
pub fn load_config(path: &str, expand_env: bool) -> Result<Config, DynDnsError> {
    let raw = std::fs::read_to_string(path)?;

    let raw = if expand_env {
        expand_env_vars(&raw)
    } else {
        raw
    };

    let cfg: Config = serde_yml::from_str(&raw)?;

    set_log_level(&cfg.log_level)?;

    Ok(cfg)
}

/// Expand `$VAR` and `${VAR}` patterns in the given string using the process environment,
/// matching Go's `os.ExpandEnv` semantics.
fn expand_env_vars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '$' {
            result.push(c);
            continue;
        }

        match chars.peek().copied() {
            None => {
                result.push('$');
            }
            Some('{') => {
                // ${VAR} syntax
                chars.next(); // consume '{'
                let var: String = chars.by_ref().take_while(|&c| c != '}').collect();
                let val = std::env::var(&var).unwrap_or_default();
                result.push_str(&val);
            }
            Some(c2) if c2.is_alphabetic() || c2 == '_' => {
                // $VAR syntax — consume word characters
                let mut var = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        var.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let val = std::env::var(&var).unwrap_or_default();
                result.push_str(&val);
            }
            _ => {
                result.push('$');
            }
        }
    }

    result
}

/// Parse log level string and configure the global tracing subscriber
pub fn set_log_level(level: &str) -> Result<(), DynDnsError> {
    match level.to_lowercase().as_str() {
        "debug" | "info" | "warn" | "error" => Ok(()),
        _ => Err(DynDnsError::UnknownLogLevel(level.to_string())),
    }
}
