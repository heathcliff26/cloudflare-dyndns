use crate::{
    config::ClientConfig,
    dyndns::{ClientData, DynDnsClient},
    server::{RequestData, ResponseMessage},
};
use anyhow::{Context, Result, bail};
use reqwest::{
    Client,
    header::{self, HeaderMap, HeaderValue},
};

/// Dyndns client using the server mode as relay for updating records.
/// Will determine the public ip's and send them to the relay server.
pub struct Relay {
    endpoint: String,
    token: String,
    data: ClientData,
}

impl Relay {
    /// Create a new Relay instance from the client config.
    pub fn from_config(config: ClientConfig) -> Self {
        Self {
            endpoint: config.endpoint,
            token: config.token,
            data: ClientData::new(config.proxy, config.domains),
        }
    }
}

impl DynDnsClient for Relay {
    fn data(&self) -> &ClientData {
        &self.data
    }
    fn data_mut(&mut self) -> &mut ClientData {
        &mut self.data
    }
    async fn send_update(&self, http_client: &Client) -> Result<()> {
        self.data().check().context("Invalid client data")?;

        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let payload = RequestData {
            token: self.token.clone(),
            domains: self.data.domains.clone(),
            ipv4: self.data.ipv4().to_string(),
            ipv6: self.data.ipv6().to_string(),
            proxy: self.data.proxy,
        };

        let response = http_client
            .post(&self.endpoint)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .context("Failed to send update to relay server")?;

        let status = response.status();

        let msg: ResponseMessage = response
            .json()
            .await
            .context("Failed to parse response from relay server")?;

        if !status.is_success() || !msg.success {
            bail!(format!(
                "Failed to update records, status: '{}': {}",
                status, msg.msg
            ));
        }
        Ok(())
    }
}
