use std::sync::OnceLock;

use axum::{Router, routing::get};
use prometheus::{CounterVec, Encoder, Registry, TextEncoder, opts, register_counter_vec_with_registry};
use tracing::{debug, error, info};

use crate::config::MetricsConfig;

static METRICS: OnceLock<Metrics> = OnceLock::new();

pub struct Metrics {
    registry: Registry,
    ipv4_counter: CounterVec,
    ipv6_counter: CounterVec,
    #[allow(dead_code)]
    request_counter: CounterVec,
}

impl Metrics {
    fn new() -> Self {
        let registry = Registry::new();

        let ipv4_counter = register_counter_vec_with_registry!(
            opts!(
                "dyndns_changed_ipv4_total",
                "Total number of times the IPv4 address has changed"
            ),
            &["domains"],
            registry
        )
        .expect("Failed to register ipv4 counter");

        let ipv6_counter = register_counter_vec_with_registry!(
            opts!(
                "dyndns_changed_ipv6_total",
                "Total number of times the IPv6 address has changed"
            ),
            &["domains"],
            registry
        )
        .expect("Failed to register ipv6 counter");

        let request_counter = register_counter_vec_with_registry!(
            opts!(
                "dyndns_requests_total",
                "Total number of requests made to update the DNS records"
            ),
            &["method", "status"],
            registry
        )
        .expect("Failed to register request counter");

        Metrics {
            registry,
            ipv4_counter,
            ipv6_counter,
            request_counter,
        }
    }

    fn gather_text(&self) -> String {
        debug!("Starting collection of metrics for cloudflare-dyndns");
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder
            .encode(&self.registry.gather(), &mut buffer)
            .unwrap_or_default();
        String::from_utf8(buffer).unwrap_or_default()
    }
}

/// Initialise and start serving Prometheus metrics in a background task.
pub fn init_metrics_and_serve(opts: &MetricsConfig) {
    if !opts.enabled {
        debug!("Metrics are disabled");
        return;
    }

    let m = Metrics::new();
    if METRICS.set(m).is_err() {
        panic!("Metrics already initialised");
    }

    let port = opts.port;
    tokio::spawn(async move {
        let app = Router::new().route("/metrics", get(metrics_handler));

        let addr = format!("0.0.0.0:{}", port);
        info!(addr = %addr, "Starting metrics server");

        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                error!(err = %e, "Failed to bind metrics server");
                std::process::exit(1);
            }
        };

        if let Err(e) = axum::serve(listener, app).await {
            error!(err = %e, "Metrics server error");
            std::process::exit(1);
        }
    });
}

async fn metrics_handler() -> String {
    match METRICS.get() {
        Some(m) => m.gather_text(),
        None => String::new(),
    }
}

/// Increment the IPv4 changed counter for the given domains.
pub fn changed_ipv4(domains: &[String]) {
    if let Some(m) = METRICS.get() {
        m.ipv4_counter
            .with_label_values(&[&domains_to_string(domains)])
            .inc();
    }
}

/// Increment the IPv6 changed counter for the given domains.
pub fn changed_ipv6(domains: &[String]) {
    if let Some(m) = METRICS.get() {
        m.ipv6_counter
            .with_label_values(&[&domains_to_string(domains)])
            .inc();
    }
}

/// Increment the request counter for the given method and status.
#[allow(dead_code)]
pub fn record_request(method: &str, status: &str) {
    if let Some(m) = METRICS.get() {
        m.request_counter
            .with_label_values(&[method, status])
            .inc();
    }
}

fn domains_to_string(domains: &[String]) -> String {
    domains.join(";")
}
