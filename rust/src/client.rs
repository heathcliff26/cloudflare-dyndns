use reqwest::{Client, Method};
use std::time::Duration;
use tracing::{debug, info};

use crate::cf_types::{CloudflareRecord, CloudflareResponse, CloudflareZone};
use crate::errors::DynDnsError;
use crate::metrics;
use crate::utils::{get_public_ipv4, get_public_ipv6};

const CLOUDFLARE_API_ENDPOINT: &str = "https://api.cloudflare.com/client/v4/";

/// State shared across update cycles.
#[derive(Debug)]
pub struct ClientData {
    pub proxy: bool,
    pub domains: Vec<String>,
    pub ipv4: String,
    pub ipv6: String,
}

impl ClientData {
    pub fn new(proxy: bool) -> Self {
        ClientData {
            proxy,
            domains: Vec::new(),
            ipv4: String::new(),
            ipv6: String::new(),
        }
    }
}

/// Cloudflare DDNS client.
#[derive(Debug)]
pub struct CloudflareClient {
    endpoint: String,
    token: String,
    http: Client,
    pub data: ClientData,
}

impl CloudflareClient {
    /// Create a new client, verifying the token against the Cloudflare API.
    pub async fn new(token: &str, proxy: bool) -> Result<Self, DynDnsError> {
        if token.is_empty() {
            return Err(DynDnsError::MissingToken);
        }

        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let client = CloudflareClient {
            endpoint: CLOUDFLARE_API_ENDPOINT.to_string(),
            token: token.to_string(),
            http,
            data: ClientData::new(proxy),
        };

        // Verify the token is valid by listing zones
        client
            .cloudflare::<Vec<CloudflareZone>>(Method::GET, "zones", None)
            .await?;

        Ok(client)
    }

    /// Send a request to the Cloudflare API and deserialize the typed result.
    async fn cloudflare<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<CloudflareResponse<T>, DynDnsError> {
        let url = format!("{}{}", self.endpoint, path);
        debug!(url = %url, method = %method, "New request to Cloudflare API");

        let mut req = self
            .http
            .request(method.clone(), &url)
            .header("Authorization", format!("Bearer {}", self.token));

        if let Some(json_body) = body {
            req = req.json(&json_body);
        }

        let res = req.send().await?;
        let status = res.status().as_u16();

        if status != 200 {
            let body = res.text().await.unwrap_or_default();
            return Err(DynDnsError::HttpRequestFailed { status, body });
        }

        let parsed: CloudflareResponse<T> = res.json().await?;
        if !parsed.success {
            return Err(DynDnsError::OperationFailed(
                parsed
                    .errors
                    .iter()
                    .map(|e| format!("[{}] {}", e.code, e.message))
                    .collect::<Vec<_>>()
                    .join("; "),
            ));
        }

        Ok(parsed)
    }

    /// Look up the Cloudflare zone ID for a domain's base domain (e.g. `foo.example.org` → zone
    /// for `example.org`).
    async fn get_zone_id(&self, domain: &str) -> Result<String, DynDnsError> {
        let zone = get_base_domain(domain);
        let path = format!("zones?name={}&status=active", zone);

        info!(zone = %zone, "Fetching zone id");
        let res = self
            .cloudflare::<Vec<CloudflareZone>>(Method::GET, &path, None)
            .await?;

        let zones = res.result.unwrap_or_default();
        if zones.is_empty() {
            return Err(DynDnsError::NoDomain);
        }
        Ok(zones[0].id.clone())
    }

    /// Retrieve all DNS records for a given zone + domain name.
    async fn get_records(
        &self,
        zone: &str,
        domain: &str,
    ) -> Result<Vec<CloudflareRecord>, DynDnsError> {
        let path = format!("zones/{}/dns_records?name={}", zone, domain);

        info!(zone = %zone, domain = %domain, "Fetching records");
        let res = self
            .cloudflare::<Vec<CloudflareRecord>>(Method::GET, &path, None)
            .await?;

        Ok(res.result.unwrap_or_default())
    }

    /// Create or update a single DNS record. TTL 1 means "automatic".
    async fn update_record(
        &self,
        zone: &str,
        domain: &str,
        record_type: &str,
        record_id: &str,
    ) -> Result<(), DynDnsError> {
        let ip = match record_type {
            "A" => self.data.ipv4.clone(),
            "AAAA" => self.data.ipv6.clone(),
            _ => String::new(),
        };

        let record = CloudflareRecord {
            content: ip.clone(),
            name: domain.to_string(),
            proxied: self.data.proxy,
            record_type: record_type.to_string(),
            ttl: 1,
            ..Default::default()
        };

        let (method, path) = if record_id.is_empty() {
            (
                Method::POST,
                format!("zones/{}/dns_records", zone),
            )
        } else {
            (
                Method::PUT,
                format!("zones/{}/dns_records/{}", zone, record_id),
            )
        };

        info!(
            zone = %zone,
            domain = %domain,
            record_type = %record_type,
            record_id = %record_id,
            proxied = self.data.proxy,
            content = %ip,
            "Updating record"
        );

        let body = serde_json::to_value(&record)?;
        self.cloudflare::<serde_json::Value>(method, &path, Some(body))
            .await?;
        Ok(())
    }

    /// Update all configured domains with the current IPs.
    pub async fn update(&self) -> Result<(), DynDnsError> {
        if self.data.ipv4.is_empty() && self.data.ipv6.is_empty() {
            return Err(DynDnsError::NoIp);
        }
        if self.data.domains.is_empty() {
            return Err(DynDnsError::NoDomain);
        }

        for domain in &self.data.domains {
            let zone = self.get_zone_id(domain).await?;
            let records = self.get_records(&zone, domain).await?;

            let mut v4_found = false;
            let mut v6_found = false;

            for record in &records {
                debug!(
                    domain = %domain,
                    record_type = %record.record_type,
                    content = %record.content,
                    modified_on = %record.modified_on,
                    "Received record"
                );

                match record.record_type.as_str() {
                    "A" => {
                        if self.data.ipv4.is_empty() {
                            continue;
                        }
                        v4_found = true;
                        if record.content == self.data.ipv4 {
                            continue;
                        }
                        self.update_record(&zone, domain, "A", &record.id).await?;
                    }
                    "AAAA" => {
                        if self.data.ipv6.is_empty() {
                            continue;
                        }
                        v6_found = true;
                        if record.content == self.data.ipv6 {
                            continue;
                        }
                        self.update_record(&zone, domain, "AAAA", &record.id)
                            .await?;
                    }
                    _ => continue,
                }
            }

            // Create A record if no existing one was found
            if !v4_found && !self.data.ipv4.is_empty() {
                self.update_record(&zone, domain, "A", "").await?;
            }
            // Create AAAA record if no existing one was found
            if !v6_found && !self.data.ipv6.is_empty() {
                self.update_record(&zone, domain, "AAAA", "").await?;
            }
        }
        Ok(())
    }
}

/// Fetch public IPs and run `client.update()` if anything changed or the last attempt failed.
async fn run_update(client: &mut CloudflareClient, updated: &mut bool) {
    let ipv4 = match get_public_ipv4().await {
        Ok(ip) => ip,
        Err(e) => {
            tracing::error!(err = %e, "Failed to get public IPv4, abort update");
            return;
        }
    };
    let ipv6 = match get_public_ipv6().await {
        Ok(ip) => ip,
        Err(e) => {
            tracing::error!(err = %e, "Failed to get public IPv6, abort update");
            return;
        }
    };

    let changed = ipv4 != client.data.ipv4 || ipv6 != client.data.ipv6;

    if changed && ipv4 != client.data.ipv4 {
        metrics::changed_ipv4(&client.data.domains);
        client.data.ipv4 = ipv4;
    }
    if changed && ipv6 != client.data.ipv6 {
        metrics::changed_ipv6(&client.data.domains);
        client.data.ipv6 = ipv6;
    }

    if changed || !*updated {
        *updated = false;
        info!(
            ipv4 = %client.data.ipv4,
            ipv6 = %client.data.ipv6,
            "Detected changed IP"
        );
        match client.update().await {
            Err(e) => tracing::error!(err = %e, "Failed to update records"),
            Ok(_) => {
                info!(
                    ipv4 = %client.data.ipv4,
                    ipv6 = %client.data.ipv6,
                    "Updated records"
                );
                *updated = true;
            }
        }
    } else {
        debug!("No change detected");
    }
}

/// Periodically fetch public IPs and update Cloudflare DNS records.
/// Runs until a SIGTERM/SIGINT signal is received.
pub async fn run(mut client: CloudflareClient, interval: std::time::Duration) {
    use tokio::signal::unix::{SignalKind, signal};
    use tokio::time;

    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to register SIGINT handler");

    let mut updated = false;

    loop {
        run_update(&mut client, &mut updated).await;

        // Wait for the next tick or a shutdown signal, matching Go's ticker behaviour where the
        // first tick arrives after the full interval has elapsed.
        tokio::select! {
            _ = time::sleep(interval) => {},
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down client");
                return;
            }
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down client");
                return;
            }
        }
    }
}

/// Extract the base domain from a FQDN (e.g. `foo.example.org` → `example.org`).
fn get_base_domain(domain: &str) -> &str {
    let parts: Vec<&str> = domain.splitn(usize::MAX, '.').collect();
    let n = parts.len();
    if n < 2 {
        return "";
    }
    let idx = domain.len() - parts[n - 2].len() - parts[n - 1].len() - 1;
    &domain[idx..]
}

#[cfg(test)]
impl CloudflareClient {
    /// Create a test client with a custom endpoint (no token verification).
    fn new_test(endpoint: &str, token: &str, proxy: bool) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");
        CloudflareClient {
            endpoint: endpoint.to_string(),
            token: token.to_string(),
            http,
            data: ClientData::new(proxy),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Json, Router,
        extract::{Path, Query, State},
        http::StatusCode,
        routing::{get, put},
    };
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;

    // ── Shared mock-server state ──────────────────────────────────────────────

    #[derive(Clone, Default)]
    struct MockState {
        records: Arc<Mutex<Vec<crate::cf_types::CloudflareRecord>>>,
        update_count: Arc<Mutex<usize>>,
    }

    // Query extractor for zone list
    #[allow(dead_code)]
    #[derive(Deserialize)]
    struct ZoneQuery {
        name: Option<String>,
        status: Option<String>,
    }

    // Query extractor for record list
    #[allow(dead_code)]
    #[derive(Deserialize)]
    struct RecordQuery {
        name: Option<String>,
    }

    fn ok_response<T: Serialize>(result: T) -> Json<Value> {
        Json(json!({
            "success": true,
            "errors": [],
            "messages": [],
            "result": serde_json::to_value(result).unwrap()
        }))
    }

    async fn zones_handler(Query(q): Query<ZoneQuery>) -> Json<Value> {
        let _ = q;
        ok_response(vec![json!({"id": "test-zone-id"})])
    }

    async fn get_records_handler(
        State(state): State<MockState>,
        Path(_zone): Path<String>,
        Query(_q): Query<RecordQuery>,
    ) -> Json<Value> {
        let records = state.records.lock().unwrap().clone();
        ok_response(records)
    }

    async fn post_record_handler(
        State(state): State<MockState>,
        Path(_zone): Path<String>,
        Json(_body): Json<Value>,
    ) -> (StatusCode, Json<Value>) {
        *state.update_count.lock().unwrap() += 1;
        (StatusCode::OK, ok_response(json!({})))
    }

    async fn put_record_handler(
        State(state): State<MockState>,
        Path((_zone, _id)): Path<(String, String)>,
        Json(_body): Json<Value>,
    ) -> (StatusCode, Json<Value>) {
        *state.update_count.lock().unwrap() += 1;
        (StatusCode::OK, ok_response(json!({})))
    }

    async fn start_mock_server(state: MockState) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind test listener");
        let addr = listener.local_addr().unwrap();

        let app = Router::new()
            .route("/zones", get(zones_handler))
            .route(
                "/zones/{zone}/dns_records",
                get(get_records_handler).post(post_record_handler),
            )
            .route(
                "/zones/{zone}/dns_records/{id}",
                put(put_record_handler),
            )
            .with_state(state);

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{}/", addr)
    }

    // ── Utility function tests ────────────────────────────────────────────────

    #[test]
    fn test_get_base_domain() {
        let cases = vec![
            ("", ""),
            ("not a domain", ""),
            ("foo.example.org", "example.org"),
            ("bar.example.org", "example.org"),
            ("example.net", "example.net"),
        ];
        for (input, expected) in cases {
            assert_eq!(get_base_domain(input), expected, "input: {input:?}");
        }
    }

    #[test]
    fn test_valid_ipv4() {
        use crate::utils::valid_ipv4;
        let cases = vec![
            ("", false),
            ("not an ip", false),
            ("100.100.100.100", true),
            ("172.198.10.100", true),
            ("fd00::dead", false),
        ];
        for (ip, ok) in cases {
            assert_eq!(valid_ipv4(ip), ok, "ip: {ip:?}");
        }
    }

    #[test]
    fn test_valid_ipv6() {
        use crate::utils::valid_ipv6;
        let cases = vec![
            ("", false),
            ("not an ip", false),
            ("100.100.100.100", false),
            ("fd69::dead", true),
            ("fd00::dead", true),
        ];
        for (ip, ok) in cases {
            assert_eq!(valid_ipv6(ip), ok, "ip: {ip:?}");
        }
    }

    // ── Client constructor tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn test_new_client_missing_token() {
        let err = CloudflareClient::new("", true).await.unwrap_err();
        assert!(matches!(err, DynDnsError::MissingToken));
    }

    // ── Cloudflare API method tests ───────────────────────────────────────────

    #[tokio::test]
    async fn test_cloudflare_authentication() {
        let state = MockState::default();
        let base = start_mock_server(state).await;

        let c = CloudflareClient::new_test(&base, "testtoken", false);
        // Verifying that a GET to /zones succeeds (authentication header is sent)
        let res = c
            .cloudflare::<Vec<CloudflareZone>>(Method::GET, "zones", None)
            .await;
        assert!(res.is_ok(), "expected ok, got: {res:?}");
    }

    #[tokio::test]
    async fn test_get_zone_id() {
        let state = MockState::default();
        let base = start_mock_server(state).await;

        let c = CloudflareClient::new_test(&base, "testtoken", false);
        let zone = c.get_zone_id("foo.example.org").await.unwrap();
        assert_eq!(zone, "test-zone-id");
    }

    #[tokio::test]
    async fn test_get_records() {
        let records = vec![
            crate::cf_types::CloudflareRecord {
                content: "100.100.100.100".into(),
                id: "21d167bb587e1d3e".into(),
                record_type: "A".into(),
                name: "foo.example.org".into(),
                ..Default::default()
            },
            crate::cf_types::CloudflareRecord {
                content: "fd00::dead".into(),
                id: "ff0012854eddab59".into(),
                record_type: "AAAA".into(),
                name: "foo.example.org".into(),
                ..Default::default()
            },
        ];
        let state = MockState {
            records: Arc::new(Mutex::new(records.clone())),
            ..Default::default()
        };
        let base = start_mock_server(state).await;

        let c = CloudflareClient::new_test(&base, "testtoken", false);
        let res = c.get_records("test-zone-id", "foo.example.org").await.unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].content, records[0].content);
        assert_eq!(res[1].content, records[1].content);
    }

    // ── Update logic tests ────────────────────────────────────────────────────

    struct UpdateCase {
        name: &'static str,
        records: Vec<crate::cf_types::CloudflareRecord>,
        ipv4: &'static str,
        ipv6: &'static str,
        expected_updates: usize,
        expect_error: bool,
    }

    fn record(content: &str, id: &str, rtype: &str) -> crate::cf_types::CloudflareRecord {
        crate::cf_types::CloudflareRecord {
            content: content.into(),
            id: id.into(),
            record_type: rtype.into(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_update() {
        let cases = vec![
            UpdateCase {
                name: "InvalidData (no IP)",
                records: vec![],
                ipv4: "",
                ipv6: "",
                expected_updates: 0,
                expect_error: true,
            },
            UpdateCase {
                name: "SingleStackIPv4Update",
                records: vec![record("100.100.100.120", "1234", "A")],
                ipv4: "100.100.100.100",
                ipv6: "",
                expected_updates: 1,
                expect_error: false,
            },
            UpdateCase {
                name: "SingleStackIPv4Create",
                records: vec![],
                ipv4: "100.100.100.100",
                ipv6: "",
                expected_updates: 1,
                expect_error: false,
            },
            UpdateCase {
                name: "SingleStackIPv4NoUpdate",
                records: vec![record("100.100.100.100", "1234", "A")],
                ipv4: "100.100.100.100",
                ipv6: "",
                expected_updates: 0,
                expect_error: false,
            },
            UpdateCase {
                name: "SingleStackIPv6Update",
                records: vec![record("fd69::1234", "1234", "AAAA")],
                ipv4: "",
                ipv6: "fd69::dead",
                expected_updates: 1,
                expect_error: false,
            },
            UpdateCase {
                name: "SingleStackIPv6Create",
                records: vec![],
                ipv4: "",
                ipv6: "fd69::dead",
                expected_updates: 1,
                expect_error: false,
            },
            UpdateCase {
                name: "SingleStackIPv6NoUpdate",
                records: vec![record("fd69::dead", "1234", "AAAA")],
                ipv4: "",
                ipv6: "fd69::dead",
                expected_updates: 0,
                expect_error: false,
            },
            UpdateCase {
                name: "DualStackUpdate",
                records: vec![
                    record("100.100.100.120", "1234", "A"),
                    record("fd69::1234", "1234", "AAAA"),
                ],
                ipv4: "100.100.100.100",
                ipv6: "fd69::dead",
                expected_updates: 2,
                expect_error: false,
            },
            UpdateCase {
                name: "DualStackCreate",
                records: vec![],
                ipv4: "100.100.100.100",
                ipv6: "fd69::dead",
                expected_updates: 2,
                expect_error: false,
            },
            UpdateCase {
                name: "DualStackNoUpdate",
                records: vec![
                    record("100.100.100.100", "1234", "A"),
                    record("fd69::dead", "1234", "AAAA"),
                ],
                ipv4: "100.100.100.100",
                ipv6: "fd69::dead",
                expected_updates: 0,
                expect_error: false,
            },
            UpdateCase {
                name: "DualStackIPv4Changed",
                records: vec![
                    record("100.100.100.120", "1234", "A"),
                    record("fd69::dead", "1234", "AAAA"),
                ],
                ipv4: "100.100.100.100",
                ipv6: "fd69::dead",
                expected_updates: 1,
                expect_error: false,
            },
            UpdateCase {
                name: "DualStackIPv6Changed",
                records: vec![
                    record("100.100.100.100", "1234", "A"),
                    record("fd69::1234", "1234", "AAAA"),
                ],
                ipv4: "100.100.100.100",
                ipv6: "fd69::dead",
                expected_updates: 1,
                expect_error: false,
            },
            UpdateCase {
                name: "SingleStackIPv4ToDualStack",
                records: vec![record("100.100.100.100", "1234", "A")],
                ipv4: "100.100.100.100",
                ipv6: "fd69::dead",
                expected_updates: 1,
                expect_error: false,
            },
            UpdateCase {
                name: "SingleStackIPv6ToDualStack",
                records: vec![record("fd69::dead", "1234", "AAAA")],
                ipv4: "100.100.100.100",
                ipv6: "fd69::dead",
                expected_updates: 1,
                expect_error: false,
            },
            UpdateCase {
                name: "DualStackIPv4OnlyUpdate",
                records: vec![
                    record("100.100.100.120", "1234", "A"),
                    record("fd69::dead", "1234", "AAAA"),
                ],
                ipv4: "100.100.100.100",
                ipv6: "",
                expected_updates: 1,
                expect_error: false,
            },
            UpdateCase {
                name: "DualStackIPv6OnlyUpdate",
                records: vec![
                    record("100.100.100.100", "1234", "A"),
                    record("fd69::1234", "1234", "AAAA"),
                ],
                ipv4: "",
                ipv6: "fd69::dead",
                expected_updates: 1,
                expect_error: false,
            },
        ];

        for tc in cases {
            let state = MockState {
                records: Arc::new(Mutex::new(tc.records)),
                ..Default::default()
            };
            let update_count = state.update_count.clone();
            let base = start_mock_server(state).await;

            let mut c = CloudflareClient::new_test(&base, "testtoken", false);
            c.data.domains = vec!["foo.example.org".to_string()];
            c.data.ipv4 = tc.ipv4.to_string();
            c.data.ipv6 = tc.ipv6.to_string();

            let result = c.update().await;
            if tc.expect_error {
                assert!(result.is_err(), "{}: expected error", tc.name);
            } else {
                assert!(result.is_ok(), "{}: unexpected error: {:?}", tc.name, result);
                let count = *update_count.lock().unwrap();
                assert_eq!(count, tc.expected_updates, "{}: expected {} updates, got {}", tc.name, tc.expected_updates, count);
            }
        }
    }

    // ── Config tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_load_config_valid() {
        use std::time::Duration;
        // Write a temp config file
        let cfg_content = r#"
logLevel: "info"
client:
  token: "my-secret-token"
  proxy: true
  domains:
    - foo.example.org
  interval: "5m"
metrics:
  enabled: false
  port: 9090
"#;
        let dir = std::env::temp_dir();
        let path = dir.join("test_valid_config.yaml");
        std::fs::write(&path, cfg_content).unwrap();

        let cfg = crate::config::load_config(path.to_str().unwrap(), false).unwrap();
        assert_eq!(cfg.log_level, "info");
        assert_eq!(cfg.client.token, "my-secret-token");
        assert!(cfg.client.proxy);
        assert_eq!(cfg.client.domains, vec!["foo.example.org"]);
        assert_eq!(cfg.client.interval, Duration::from_secs(300));
        assert!(!cfg.metrics.enabled);
        assert_eq!(cfg.metrics.port, 9090);
    }

    #[test]
    fn test_load_config_env_expansion() {
        let cfg_content = "logLevel: \"info\"\nclient:\n  token: \"${TEST_CF_TOKEN}\"\n  domains:\n    - foo.example.org\n  interval: \"5m\"\n";
        let dir = std::env::temp_dir();
        let path = dir.join("test_env_config.yaml");
        std::fs::write(&path, cfg_content).unwrap();

        // SAFETY: test runs single-threaded and no other test touches TEST_CF_TOKEN.
        unsafe { std::env::set_var("TEST_CF_TOKEN", "token-from-env") };
        let cfg = crate::config::load_config(path.to_str().unwrap(), true).unwrap();
        assert_eq!(cfg.client.token, "token-from-env");
        // SAFETY: same as above.
        unsafe { std::env::remove_var("TEST_CF_TOKEN") };
    }

    #[test]
    fn test_load_config_missing_file() {
        let err = crate::config::load_config("file-does-not-exist.yaml", false).unwrap_err();
        assert!(matches!(err, crate::errors::DynDnsError::Io(_)));
    }

    #[test]
    fn test_validate_client_missing_token() {
        let mut cfg = crate::config::Config::default();
        cfg.client.domains = vec!["foo.example.org".to_string()];
        let err = cfg.validate_client().unwrap_err();
        assert!(matches!(err, crate::errors::DynDnsError::MissingToken));
    }

    #[test]
    fn test_validate_client_no_domain() {
        let mut cfg = crate::config::Config::default();
        cfg.client.token = "some-token".to_string();
        let err = cfg.validate_client().unwrap_err();
        assert!(matches!(err, crate::errors::DynDnsError::NoDomain));
    }

    #[test]
    fn test_validate_client_invalid_interval() {
        let mut cfg = crate::config::Config::default();
        cfg.client.token = "some-token".to_string();
        cfg.client.domains = vec!["foo.example.org".to_string()];
        cfg.client.interval = std::time::Duration::from_secs(10); // less than 30s
        let err = cfg.validate_client().unwrap_err();
        assert!(matches!(err, crate::errors::DynDnsError::InvalidInterval(_)));
    }
}
