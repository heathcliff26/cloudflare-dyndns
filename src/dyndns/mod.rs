use anyhow::{Context, Result, bail};
use reqwest::Client;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
    time::Duration,
};
use tokio::{net::UdpSocket, sync::watch};
use tracing::{debug, error, info};

use crate::utils::new_http_client;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

/// Implement the update loop for a dynamic DNS client.
pub trait DynDnsClient {
    /// Get the current state of the client.
    fn data(&self) -> &ClientData;
    /// Get a mutable reference to the current state of the client.
    fn data_mut(&mut self) -> &mut ClientData;
    /// Perform an update to the DNS provider with the current state of the client.
    async fn send_update(&self, http_client: &Client) -> Result<()>;
    /// Update the current state and call send_update if it has changed.
    /// Updates can be forced by setting the update parameter to true.
    async fn update(&mut self, update: bool) -> Result<()> {
        let client = new_http_client();
        let ipv4 = get_public_ipv4(&client)
            .await
            .context("Failed to get ipv4")?;
        let ipv6 = get_public_ipv6(&client)
            .await
            .context("Failed to get ipv6")?;
        let mut changed = false;
        if ipv4 != self.data().ipv4() {
            self.data_mut().set_ipv4(&ipv4)?;
            changed = true;
        }
        if ipv6 != self.data().ipv6() {
            self.data_mut().set_ipv6(&ipv6)?;
            changed = true;
        }
        if changed || update {
            info!(
                "IP address changed, updating dns records, ipv4: '{}', ipv6: '{}'",
                self.data().ipv4(),
                self.data().ipv6()
            );
            self.send_update(&client)
                .await
                .context("Failed to update dns records")?;
            debug!(
                "Updated dns records, ipv4: '{}', ipv6: '{}'",
                self.data().ipv4(),
                self.data().ipv6()
            );
        } else {
            debug!("No changed detected");
        }
        Ok(())
    }
    /// Continuously check for IP changes and update the DNS provider when they occur.
    /// Arguments:
    /// - `interval`: The interval in seconds at which to check for IP changes.
    /// - `stop_rx`: A channel to signal the loop to stop.
    async fn run(&mut self, interval: u64, mut stop_rx: watch::Receiver<()>) {
        let mut updated = false;
        let mut interval = tokio::time::interval(Duration::from_secs(interval));
        loop {
            tokio::select! {
                _ = stop_rx.changed() => {
                    info!("Received stop signal, shutting down client");
                    return;
                }
                _ = interval.tick() => {
                    if let Err(e) = self.update(!updated).await {
                        error!("Failed to update client: {e:#}");
                        updated = false;
                    } else {
                        updated = true;
                    }
                }
            }
        }
    }
}

/// Holds the current state of a dyndns client.
pub struct ClientData {
    pub proxy: bool,
    pub domains: Vec<String>,
    ipv4: String,
    ipv6: String,
}

impl ClientData {
    /// Create a new ClientData instance.
    /// IP Addresses need to be set separately to allow for verification.
    pub fn new(proxy: bool, domains: Vec<String>) -> Self {
        Self {
            proxy,
            domains,
            ipv4: String::new(),
            ipv6: String::new(),
        }
    }
    /// Get the IPv4 address
    pub fn ipv4(&self) -> &str {
        &self.ipv4
    }
    /// Get the IPv6 address
    pub fn ipv6(&self) -> &str {
        &self.ipv6
    }
    /// Set the IPv4 address
    pub fn set_ipv4(&mut self, ip: &str) -> Result<()> {
        if !ip.is_empty() {
            validate_ipv4(ip)?;
        }
        self.ipv4 = ip.to_string();
        Ok(())
    }
    /// Set the IPv6 address
    pub fn set_ipv6(&mut self, ip: &str) -> Result<()> {
        if !ip.is_empty() {
            validate_ipv6(ip)?;
        }
        self.ipv6 = ip.to_string();
        Ok(())
    }
    /// Checks if data contains at least one IP and one domain
    pub fn check(&self) -> Result<()> {
        if self.ipv4.is_empty() && self.ipv6.is_empty() {
            bail!("At least one IP address must be provided");
        }
        if self.domains.is_empty() {
            bail!("At least one domain must be provided");
        }
        Ok(())
    }
}

/// Check if host can call public IPv6 addresses.
pub async fn has_ipv6_support() -> bool {
    let sock = match UdpSocket::bind("[::]:0").await {
        Ok(s) => s,
        Err(_) => return false,
    };
    sock.connect("[2606:4700:4700::1111]:80").await.is_ok()
}

/// Return the public IPv4 address of the host
pub async fn get_public_ipv4(client: &Client) -> Result<String> {
    let ip = call_public_ip_service(client, "ipv4")
        .await
        .context("Failed to get public IPv4 address")?;

    validate_ipv4(&ip)?;
    Ok(ip)
}

/// Return the public IPv6 address of the host.
/// If the host does not have IPv6 support, an empty string is returned.
pub async fn get_public_ipv6(client: &Client) -> Result<String> {
    if !has_ipv6_support().await {
        return Ok(String::new());
    }
    let ip = call_public_ip_service(client, "ipv6")
        .await
        .context("Failed to get public IPv6 address")?;

    validate_ipv6(&ip)?;
    Ok(ip)
}

/// Call icanhazip to get the public IP address of the host.
async fn call_public_ip_service(client: &Client, version: &str) -> reqwest::Result<String> {
    let ip = client
        .get(format!("https://{version}.icanhazip.com"))
        .send()
        .await?
        .text()
        .await?
        .trim()
        .to_string();
    Ok(ip)
}

/// Validate if the provided string is a valid IPv4 address.
fn validate_ipv4(ip: &str) -> Result<()> {
    Ipv4Addr::from_str(ip).context(format!("Invalid IPv4 address: '{ip}'"))?;
    Ok(())
}

/// Validate if the provided string is a valid IPv6 address.
fn validate_ipv6(ip: &str) -> Result<()> {
    Ipv6Addr::from_str(ip).context(format!("Invalid IPv6 address: '{ip}'"))?;
    Ok(())
}
