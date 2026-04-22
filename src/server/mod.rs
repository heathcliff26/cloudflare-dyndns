use crate::{
    client::Client,
    config::ServerConfig,
    dyndns::{ClientData, DynDnsClient},
    utils::{self, new_http_client},
};
use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
    routing::{any, get, post},
};
use serde::{Deserialize, Serialize};
use serde_qs::axum::QsQuery;
use std::{
    net::SocketAddr,
    sync::{Arc, atomic::AtomicU16},
};
use tokio::{net::TcpListener, sync::watch};
use tracing::info;

#[cfg(test)]
mod test;
mod tls;

const MESSAGE_DOMAINS_FORBIDDEN: &str =
    "At least one of the provided domains is not whitelisted by the server";
const MESSAGE_UNAUTHORIZED: &str =
    "Failed to authenticate to Cloudflare, please provide a valid API token";
const MESSAGE_FAILED_UPDATE: &str = "Failed to update the records";
const MESSAGE_SUCCESS: &str = "Updated dyndns records";

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestData {
    /// Cloudflare API token
    #[serde(default = "String::new")]
    pub token: String,
    /// List of domains to update, requires at least one domain.
    pub domains: Vec<String>,
    /// IPv4 Address, optional, when IPv6 set
    #[serde(default = "String::new", skip_serializing_if = "String::is_empty")]
    pub ipv4: String,
    /// IPv6 Address, optional, when IPv4 set
    #[serde(default = "String::new", skip_serializing_if = "String::is_empty")]
    pub ipv6: String,
    /// Indicate if domain should be proxied, defaults to true
    #[serde(default = "default_true")]
    pub proxy: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// Message describing the result of the request
    pub msg: String,
    /// Indicates if the request was successful
    pub success: bool,
}

/// HTTP Server for receiving updates from clients and relaying them to the cloudflare API.
#[derive(Clone)]
pub struct Server {
    options: ServerConfig,
    current_port: Arc<AtomicU16>,
}

/// Shared state for the server
#[derive(Clone)]
struct SharedState {
    domains: Vec<String>,
}

impl SharedState {
    /// Create a new shared state with the given domains.
    fn new(domains: Vec<String>) -> Self {
        Self { domains }
    }
    /// Verify that all domains are whitelisted by the server.
    /// Returns false if any domain is not whitelisted.
    /// Returns true if all domains are whitelisted or if the whitelist is empty.
    fn verify_domains(&self, domains: &[String]) -> bool {
        if self.domains.is_empty() {
            return true;
        }
        for domain in domains.iter() {
            let mut forbidden = true;
            let d: Vec<&str> = domain.split(".").collect();
            let mut name = String::new();
            for part in d.iter().rev() {
                if name.is_empty() {
                    name = part.to_string();
                } else {
                    name = format!("{}.{}", part, name);
                }
                if self.domains.contains(&name) {
                    forbidden = false;
                    break;
                }
            }
            if forbidden {
                return false;
            }
        }
        true
    }
}

impl Server {
    /// Create a new server from the given configuration.
    pub fn from_config(config: ServerConfig) -> Self {
        Self {
            options: config,
            current_port: Arc::new(AtomicU16::new(0)),
        }
    }
    /// Run the server
    /// Server will shutdown gracefully on Ctrl+C or SIGTERM
    pub async fn run(&mut self, stop_rx: Option<watch::Receiver<()>>) -> Result<()> {
        let health_router: Router = Router::new().route("/healthz", any(healthz));

        let state = SharedState::new(self.options.domains.clone());

        let update_router: Router = Router::new()
            .route("/", post(update_handler_post))
            .route("/", get(update_handler_get))
            .with_state(state);

        let router = Router::new().merge(health_router).merge(update_router);

        let addr = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], self.options.port));
        info!("Starting server on {}", addr);

        let shutdown = async move {
            utils::new_interrupt_signal(stop_rx)
                .changed()
                .await
                .expect("Failed to create shutdown signal");
        };

        if self.options.ssl.enabled {
            let listener =
                tls::TlsListener::bind(addr, &self.options.ssl.key, &self.options.ssl.cert)
                    .await
                    .context("Failed to open port")?;

            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown)
                .await
                .context("Failed to start server")?;
        } else {
            let listener = TcpListener::bind(addr)
                .await
                .context("Failed to open port")?;

            self.current_port.store(
                listener
                    .local_addr()
                    .context("Failed to get listener port")?
                    .port(),
                std::sync::atomic::Ordering::SeqCst,
            );

            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown)
                .await
                .context("Failed to start server")?;
        }

        Ok(())
    }
}

/// Expose health check endpoint
/// Can be used when running under kubernetes to check if the server is running
/// GET /healthz
async fn healthz() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    (headers, "Ok")
}

/// Handle dyndns update requests
async fn update_handler(
    state: SharedState,
    payload: RequestData,
) -> (StatusCode, Json<ResponseMessage>) {
    if !state.verify_domains(&payload.domains) {
        return (
            StatusCode::FORBIDDEN,
            Json(ResponseMessage {
                msg: MESSAGE_DOMAINS_FORBIDDEN.to_string(),
                success: false,
            }),
        );
    }

    let mut data = ClientData::new(payload.proxy, payload.domains);
    if let Err(e) = data.set_ipv4(&payload.ipv4) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ResponseMessage {
                msg: e.to_string(),
                success: false,
            }),
        );
    }
    if let Err(e) = data.set_ipv6(&payload.ipv6) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ResponseMessage {
                msg: e.to_string(),
                success: false,
            }),
        );
    }

    if let Err(e) = data.check() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ResponseMessage {
                msg: e.to_string(),
                success: false,
            }),
        );
    }

    let client = Client::from_data(payload.token, data);
    let http_client = new_http_client();
    if let Err(e) = client.verify_token(&http_client).await {
        info!("Failed to verify token: {e:#}");
        return (
            StatusCode::UNAUTHORIZED,
            Json(ResponseMessage {
                msg: MESSAGE_UNAUTHORIZED.to_string(),
                success: false,
            }),
        );
    }

    if let Err(e) = client.send_update(&http_client).await {
        info!(
            "Failed to update records, domains='{}', ipv4='{}', ipv6='{}', proxy='{}': {e:#}",
            client.data().domains.join(","),
            client.data().ipv4(),
            client.data().ipv6(),
            client.data().proxy
        );
        return (
            StatusCode::OK,
            Json(ResponseMessage {
                msg: MESSAGE_FAILED_UPDATE.to_string(),
                success: false,
            }),
        );
    }
    info!(
        "Successfully updated records for domains='{}', ipv4='{}', ipv6='{}', proxy='{}'",
        client.data().domains.join(","),
        client.data().ipv4(),
        client.data().ipv6(),
        client.data().proxy
    );
    (
        StatusCode::OK,
        Json(ResponseMessage {
            msg: MESSAGE_SUCCESS.to_string(),
            success: true,
        }),
    )
}

/// Handle dyndns update requests, calls update_handler
/// Method: POST
async fn update_handler_post(
    state: State<SharedState>,
    Json(payload): Json<RequestData>,
) -> (StatusCode, Json<ResponseMessage>) {
    update_handler(state.0, payload).await
}

/// Handle dyndns update requests, calls update_handler
/// Method: GET
async fn update_handler_get(
    state: State<SharedState>,
    params: QsQuery<RequestData>,
) -> (StatusCode, Json<ResponseMessage>) {
    update_handler(state.0, params.0).await
}
