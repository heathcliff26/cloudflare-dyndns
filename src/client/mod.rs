use crate::{
    client::cloudflare::*,
    config::ClientConfig,
    dyndns::{ClientData, DynDnsClient},
};
use anyhow::{Context, Result, bail};
use reqwest::Client as HttpClient;
use serde::Deserialize;
use tracing::debug;

pub mod cloudflare;
#[cfg(test)]
mod test;

/// Dyndns client using the server mode as relay for updating records.
/// Will determine the public ip's and send them to the relay server.
pub struct Client {
    api_url: String,
    token: String,
    data: ClientData,
}

impl Client {
    /// Create a new Client instance from the client config.
    pub fn from_config(config: ClientConfig) -> Self {
        Self {
            api_url: DEFAULT_API_URL.to_string(),
            token: config.token,
            data: ClientData::new(config.proxy, config.domains),
        }
    }
    /// Create a new Client instance from the provided data.
    pub fn from_data(token: String, data: ClientData) -> Self {
        Self {
            api_url: DEFAULT_API_URL.to_string(),
            token,
            data,
        }
    }

    /// Send a get request to the endpoint and parse the response.
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        http_client: &HttpClient,
        url: &str,
    ) -> Result<T> {
        let response = http_client
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await
            .context("Failed to send request to cloudflare api")?;

        parse_response(response).await
    }

    /// Verify that the provided token is valid and active.
    /// GET: /user/tokens/verify
    pub async fn verify_token(&self, http_client: &HttpClient) -> Result<()> {
        if self.token.is_empty() {
            bail!("Missing cloudflare api token");
        }

        let url = format!("{}/user/tokens/verify", self.api_url);
        let result: VerifyTokenResult = self
            .get(http_client, &url)
            .await
            .context("Failed to verify token")?;

        if result.status != Status::Active {
            bail!("Token is not active");
        }
        Ok(())
    }

    /// Retrieve the zone id for the given domain.
    /// GET: /zones?name={domain}&status=active
    pub async fn get_zone_id(&self, http_client: &HttpClient, domain: &str) -> Result<String> {
        let domain = base_domain(domain);
        if domain.is_empty() {
            bail!("Invalid domain");
        }

        debug!("Retrieving zone id for domain: {}", domain);
        let url = format!("{}/zones?name={}&status=active", self.api_url, domain);
        let zones: Vec<Zone> = self
            .get(http_client, &url)
            .await
            .context("Failed to list zones")?;

        if zones.is_empty() {
            bail!("No zone found, does the domain exist and is active?");
        }
        Ok(zones[0].id.clone())
    }

    /// Retrieve the dns records for the given domain.
    /// GET: /zones/{zone_id}/dns_records?name={domain}
    pub async fn get_records(
        &self,
        http_client: &HttpClient,
        zone_id: &str,
        domain: &str,
    ) -> Result<Vec<Record>> {
        let url = format!(
            "{}/zones/{}/dns_records?name={}",
            self.api_url, zone_id, domain
        );
        let records: Vec<Record> = self
            .get(http_client, &url)
            .await
            .context("Failed to list dns records")?;
        Ok(records)
    }

    /// Update the dns record with the given content.
    /// PATCH: /zones/{zone_id}/dns_records/{record_id}
    pub async fn update_record(
        &self,
        http_client: &HttpClient,
        zone_id: &str,
        domain: &str,
        record_id: &str,
        record_type: RecordType,
    ) -> Result<()> {
        let (record_type, ip) = match record_type {
            RecordType::A => ("A".to_string(), self.data().ipv4()),
            RecordType::AAAA => ("AAAA".to_string(), self.data().ipv6()),
        };
        let record = Record {
            name: domain.to_string(),
            ttl: 1,
            record_type,
            content: ip.to_string(),
            proxied: self.data().proxy,
            id: record_id.to_string(),
        };
        let url = format!(
            "{}/zones/{}/dns_records/{}",
            self.api_url, zone_id, record_id
        );
        let response = http_client
            .patch(&url)
            .bearer_auth(&self.token)
            .json(&record)
            .send()
            .await
            .context("Failed to send request to cloudflare api")?;

        let _: Record = parse_response(response)
            .await
            .context("Failed to update dns record on cloudflare")?;
        Ok(())
    }

    /// Create the dns record with the given content.
    /// POST: /zones/{zone_id}/dns_records
    pub async fn create_record(
        &self,
        http_client: &HttpClient,
        zone_id: &str,
        domain: &str,
        record_type: RecordType,
    ) -> Result<()> {
        let (record_type, ip) = match record_type {
            RecordType::A => ("A".to_string(), self.data().ipv4()),
            RecordType::AAAA => ("AAAA".to_string(), self.data().ipv6()),
        };
        let record = Record {
            name: domain.to_string(),
            ttl: 1,
            record_type,
            content: ip.to_string(),
            proxied: self.data().proxy,
            id: String::new(),
        };
        let url = format!("{}/zones/{}/dns_records", self.api_url, zone_id);
        let response = http_client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&record)
            .send()
            .await
            .context("Failed to send request to cloudflare api")?;

        let _: Record = parse_response(response)
            .await
            .context("Failed to update dns record on cloudflare")?;
        Ok(())
    }
}

impl DynDnsClient for Client {
    fn data(&self) -> &ClientData {
        &self.data
    }
    fn data_mut(&mut self) -> &mut ClientData {
        &mut self.data
    }
    async fn send_update(&self, http_client: &HttpClient) -> Result<()> {
        self.data().check().context("Invalid client data")?;
        for domain in self.data().domains.iter() {
            let zone_id = self
                .get_zone_id(&http_client, domain)
                .await
                .context(format!("Failed to get zone for '{domain}'"))?;
            let records = self
                .get_records(&http_client, &zone_id, domain)
                .await
                .context(format!("Failed to list dns records for '{domain}'"))?;

            let mut v4 = false;
            let mut v6 = false;
            for record in records.iter() {
                debug!("Received record from '{domain}': {:?}", record.content);
                match record.record_type.as_str() {
                    "A" => {
                        if self.data().ipv4().is_empty() {
                            continue;
                        }
                        v4 = true;
                        if record.content == self.data().ipv4() {
                            debug!(
                                "IPv4 address '{}' for '{}' is up to date",
                                self.data().ipv4(),
                                domain
                            );
                            continue;
                        }

                        debug!(
                            "Updating A record for '{domain}' with ip '{}'",
                            self.data().ipv4()
                        );
                        self.update_record(
                            &http_client,
                            &zone_id,
                            domain,
                            &record.id,
                            cloudflare::RecordType::A,
                        )
                        .await
                        .context(format!("Failed to update A record for '{domain}'"))?;
                    }
                    "AAAA" => {
                        if self.data().ipv6().is_empty() {
                            continue;
                        }
                        v6 = true;
                        if record.content == self.data().ipv6() {
                            debug!(
                                "IPv6 address '{}' for '{}' is up to date",
                                self.data().ipv6(),
                                domain
                            );
                            continue;
                        }

                        debug!(
                            "Updating AAAA record for '{domain}' with ip '{}'",
                            self.data().ipv6()
                        );
                        self.update_record(
                            &http_client,
                            &zone_id,
                            domain,
                            &record.id,
                            cloudflare::RecordType::AAAA,
                        )
                        .await
                        .context(format!("Failed to update AAAA record for '{domain}'"))?;
                    }
                    _ => {
                        continue;
                    }
                }
            }
            // Create A record if necessary
            if !v4 && !self.data().ipv4().is_empty() {
                debug!(
                    "Creating A record for '{domain}' with ip '{}'",
                    self.data().ipv4()
                );
                self.create_record(&http_client, &zone_id, domain, cloudflare::RecordType::A)
                    .await
                    .context(format!("Failed to create A record for '{domain}'"))?;
            }
            // Create AAAA record if necessary
            if !v6 && !self.data().ipv6().is_empty() {
                debug!(
                    "Creating AAAA record for '{domain}' with ip '{}'",
                    self.data().ipv6()
                );
                self.create_record(&http_client, &zone_id, domain, cloudflare::RecordType::AAAA)
                    .await
                    .context(format!("Failed to create AAAA record for '{domain}'"))?;
            }
        }
        Ok(())
    }
}

/// Parse the returned cloudflare response
async fn parse_response<T: for<'de> Deserialize<'de>>(response: reqwest::Response) -> Result<T> {
    if !response.status().is_success() {
        bail!(
            "Request to cloudflare api failed, status: {}",
            response.status()
        );
    }

    let response: CloudflareResponse<T> = response
        .json()
        .await
        .context("Failed to parse response from cloudflare api")?;

    if !response.success {
        bail!("API responded with failure");
    }

    let result = match response.result {
        Some(result) => result,
        None => bail!("API responded with success but returned no result"),
    };
    Ok(result)
}

/// Return the base domain for a given domain.
/// Example: "sub.example.com" -> "example.com"
fn base_domain(domain: &str) -> String {
    let parts: Vec<&str> = domain.split('.').collect();
    // Should be at least 2 entries
    if parts.len() < 2 {
        return String::new();
    }
    parts[parts.len() - 2..].join(".")
}
