use std::net::UdpSocket;

use crate::errors::DynDnsError;

/// Check if the host has IPv6 connectivity by attempting a UDP "connect" to Cloudflare's
/// public IPv6 resolver (no data is actually sent).
pub fn has_ipv6_support() -> bool {
    UdpSocket::bind("[::]:0")
        .and_then(|s| s.connect("[2606:4700:4700::1111]:80").map(|_| s))
        .is_ok()
}

/// Validate whether the given string is a valid IPv4 address.
pub fn valid_ipv4(ip: &str) -> bool {
    ip.parse::<std::net::Ipv4Addr>().is_ok()
}

/// Validate whether the given string is a valid IPv6 address.
pub fn valid_ipv6(ip: &str) -> bool {
    ip.parse::<std::net::Ipv6Addr>().is_ok()
}

/// Fetch the public IP address (v4 or v6) from icanhazip.com.
async fn get_public_ip(version: &str) -> Result<String, DynDnsError> {
    let url = format!("https://{}.icanhazip.com", version);
    let response = reqwest::get(&url).await?;
    let status = response.status().as_u16();
    if status != 200 {
        let body = response.text().await.unwrap_or_default();
        return Err(DynDnsError::HttpRequestFailed { status, body });
    }
    let ip = response.text().await?.trim().to_string();
    if !valid_ipv4(&ip) && !valid_ipv6(&ip) {
        return Err(DynDnsError::InvalidIp(ip));
    }
    Ok(ip)
}

/// Fetch the public IPv4 address. Returns an empty string if no IPv4 is available.
pub async fn get_public_ipv4() -> Result<String, DynDnsError> {
    get_public_ip("ipv4").await
}

/// Fetch the public IPv6 address. Returns an empty string if there is no IPv6 support.
pub async fn get_public_ipv6() -> Result<String, DynDnsError> {
    if !has_ipv6_support() {
        return Ok(String::new());
    }
    get_public_ip("ipv6").await
}
