use mockito::{Server, ServerGuard};

use crate::dyndns::DynDnsClient;

use super::*;

#[test]
fn test_from_config() {
    let client = Client::from_config(ClientConfig::default());
    assert_eq!(
        client.api_url.as_str(),
        DEFAULT_API_URL,
        "Should have default API URL"
    );
}

#[test]
fn test_from_data() {
    let data = ClientData::new(true, vec!["example.com".to_string()]);
    let client = Client::from_data("test-token".to_string(), data);
    assert_eq!(
        client.api_url.as_str(),
        DEFAULT_API_URL,
        "Should have default API URL"
    );
    assert_eq!(client.token, "test-token", "Should have the correct token");
}

#[tokio::test]
async fn test_verify_token_success() {
    let (client, http_client, mut server) = new_test_client().await;

    let token_mock = server
        .mock("GET", "/user/tokens/verify")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"success":true,"errors":[],"messages":[], "result": {"status": "active", "id": "testid"}}"#)
        .create();

    client
        .verify_token(&http_client)
        .await
        .expect("Should verify token");

    token_mock.assert();
}

#[tokio::test]
async fn test_verify_token_empty_token() {
    let data = ClientData::new(true, vec!["example.com".to_string()]);
    let client = Client {
        api_url: "http://localhost".to_string(),
        token: "".to_string(),
        data,
    };

    let e = client
        .verify_token(&HttpClient::new())
        .await
        .expect_err("Should fail with empty token");
    assert!(
        e.to_string().contains("Missing cloudflare api token"),
        "Expected error about missing token but got: {e:#}"
    );
}

#[tokio::test]
async fn test_verify_token_inactive() {
    let (client, http_client, mut server) = new_test_client().await;

    let token_mock = server
        .mock("GET", "/user/tokens/verify")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"success":true,"errors":[],"messages":[], "result": {"status": "inactive", "id": "testid"}}"#)
        .create();

    let e = client
        .verify_token(&http_client)
        .await
        .expect_err("Should fail with inactive token");
    assert!(
        e.to_string().contains("Token is not active"),
        "Expected error about inactive token but got: {e}"
    );

    token_mock.assert();
}

#[tokio::test]
async fn test_verify_token_api_error() {
    let (client, http_client, mut server) = new_test_client().await;

    let token_mock = server
        .mock("GET", "/user/tokens/verify")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"success":false,"errors":[],"messages":[]}"#)
        .create();

    let e = client
        .verify_token(&http_client)
        .await
        .expect_err("Should fail with API error");
    let error_msg = e.to_string();
    assert!(
        error_msg.contains("Failed to verify token"),
        "Expected verify token error but got: {error_msg}"
    );

    token_mock.assert();
}

#[tokio::test]
async fn test_get_zone_id() {
    let (client, http_client, mut server) = new_test_client().await;

    let e = client
        .get_zone_id(&http_client, "com")
        .await
        .expect_err("Should fail with invalid domain");
    assert!(
        e.to_string().contains("Invalid domain"),
        "Expected error about invalid domain but got: {e}"
    );

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(zone_list_response_ok())
        .create();

    client
        .get_zone_id(&http_client, "foo.example.com")
        .await
        .expect("Should get a zone id");
    zones_mock.assert();

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(zone_list_response_empty())
        .create();

    let e = client
        .get_zone_id(&http_client, "foo.example.com")
        .await
        .expect_err("Should fail if no zone is found");
    assert!(
        e.to_string().contains("No zone found"),
        "Expected error about no zone found but got: {e}"
    );
    zones_mock.assert();
}

#[tokio::test]
async fn test_get_records() {
    let (client, http_client, mut server) = new_test_client().await;

    let records_mock = server
        .mock("GET", "/zones/zone-id-123/dns_records?name=foo.example.com")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_list_response(vec![dns_record_a_json(
            "record-id-123",
            "foo.example.com",
            "100.100.100.100",
        )]))
        .create();

    let records = client
        .get_records(&http_client, "zone-id-123", "foo.example.com")
        .await
        .expect("Should get dns records");

    assert_eq!(records.len(), 1, "Expected exactly one DNS record");
    assert_eq!(records[0].id, "record-id-123");
    assert_eq!(records[0].name, "foo.example.com");
    assert_eq!(records[0].content, "100.100.100.100");
    assert_eq!(records[0].record_type, "A");

    records_mock.assert();
}

#[tokio::test]
async fn test_update_record() {
    let (client, http_client, mut server) = new_test_client().await;

    let record_mock = server
        .mock("PATCH", "/zones/zone-id-123/dns_records/record-id-123")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_single_result_response(dns_record_a_json(
            "record-a",
            "foo.example.com",
            "100.100.100.100",
        )))
        .create();

    client
        .update_record(
            &http_client,
            "zone-id-123",
            "foo.example.com",
            "record-id-123",
            RecordType::A,
        )
        .await
        .expect("Should update dns record");

    record_mock.assert();
}

#[tokio::test]
async fn test_create_record() {
    let (client, http_client, mut server) = new_test_client().await;

    let record_mock = server
        .mock("POST", "/zones/zone-id-123/dns_records")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_single_result_response(dns_record_a_json(
            "record-a",
            "foo.example.com",
            "100.100.100.100",
        )))
        .create();

    client
        .create_record(
            &http_client,
            "zone-id-123",
            "foo.example.com",
            RecordType::A,
        )
        .await
        .expect("Should create dns record");

    record_mock.assert();
}

#[tokio::test]
async fn test_send_update_invalid_client_data() {
    let data = ClientData::new(true, vec!["foo.example.com".to_string()]);
    let (client, http_client, _server) = new_test_client_with_data(data).await;

    let e = client
        .send_update(&http_client)
        .await
        .expect_err("Should fail with invalid client data");
    assert!(
        e.to_string().contains("Invalid client data"),
        "Expected error about invalid client data but got: {e}"
    );
}

#[tokio::test]
async fn test_send_update_invalid_domain() {
    let mut data = ClientData::new(true, vec!["com".to_string()]);
    data.set_ipv4("100.100.100.100").expect("Should set ipv4");

    let (client, http_client, _server) = new_test_client_with_data(data).await;

    let e = client
        .send_update(&http_client)
        .await
        .expect_err("Should fail with invalid domain");
    assert!(
        e.to_string().contains("Failed to get zone for 'com'"),
        "Expected error context for invalid domain but got: {e}"
    );
}

#[tokio::test]
async fn test_send_update_get_records_error() {
    let (client, http_client, mut server) = new_test_client().await;

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(zone_list_response_ok())
        .create();

    let records_mock = server
        .mock("GET", "/zones/zone-id-123/dns_records?name=example.com")
        .with_status(500)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"success":false,"errors":[],"messages":[]}"#)
        .create();

    let e = client
        .send_update(&http_client)
        .await
        .expect_err("Should fail when listing records fails");
    assert!(
        e.to_string()
            .contains("Failed to list dns records for 'example.com'"),
        "Expected error about listing records but got: {e}"
    );

    zones_mock.assert();
    records_mock.assert();
}

#[tokio::test]
async fn test_send_update_updates_existing_a_and_aaaa_records() {
    let (client, http_client, mut server) = new_test_client().await;

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(zone_list_response_ok())
        .create();

    let records = vec![
        dns_record_a_json("record-a", "example.com", "1.1.1.1"),
        dns_record_aaaa_json("record-aaaa", "example.com", "fd00::beef"),
        dns_record_cname_json("record-cname", "example.com", "target.example.net"),
    ];
    let records_mock = server
        .mock("GET", "/zones/zone-id-123/dns_records?name=example.com")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_list_response(records))
        .create();

    let update_a_mock = server
        .mock("PATCH", "/zones/zone-id-123/dns_records/record-a")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_single_result_response(dns_record_a_json(
            "record-a",
            "example.com",
            "100.100.100.100",
        )))
        .create();

    let update_aaaa_mock = server
        .mock("PATCH", "/zones/zone-id-123/dns_records/record-aaaa")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_single_result_response(dns_record_aaaa_json(
            "record-aaaa",
            "example.com",
            "fd00::dead",
        )))
        .create();

    let create_mock = server
        .mock("POST", "/zones/zone-id-123/dns_records")
        .expect(0)
        .create();

    client
        .send_update(&http_client)
        .await
        .expect("Should update existing records");

    zones_mock.assert();
    records_mock.assert();
    update_a_mock.assert();
    update_aaaa_mock.assert();
    create_mock.assert();
}

#[tokio::test]
async fn test_send_update_creates_missing_a_and_aaaa_records() {
    let (client, http_client, mut server) = new_test_client().await;

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(zone_list_response_ok())
        .create();

    let records_mock = server
        .mock("GET", "/zones/zone-id-123/dns_records?name=example.com")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_list_response(vec![]))
        .create();

    // As we do not parse the anser we can get by with sending the same response twice.
    let create_mock = server
        .mock("POST", "/zones/zone-id-123/dns_records")
        .expect(2)
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_single_result_response(dns_record_a_json(
            "record-created",
            "example.com",
            "100.100.100.100",
        )))
        .create();

    client
        .send_update(&http_client)
        .await
        .expect("Should create missing records");

    zones_mock.assert();
    records_mock.assert();
    create_mock.assert();
}

#[tokio::test]
async fn test_send_update_up_to_date_records_no_changes() {
    let (client, http_client, mut server) = new_test_client().await;

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(zone_list_response_ok())
        .create();

    let records = vec![
        dns_record_a_json("record-a", "example.com", "100.100.100.100"),
        dns_record_aaaa_json("record-aaaa", "example.com", "fd00::dead"),
    ];
    let records_mock = server
        .mock("GET", "/zones/zone-id-123/dns_records?name=example.com")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_list_response(records))
        .create();

    let create_mock = server
        .mock("POST", "/zones/zone-id-123/dns_records")
        .expect(0)
        .create();

    let update_mock = server
        .mock("PUT", "/zones/zone-id-123/dns_records")
        .expect(0)
        .create();

    client
        .send_update(&http_client)
        .await
        .expect("Should not change up-to-date records");

    zones_mock.assert();
    records_mock.assert();
    create_mock.assert();
    update_mock.assert();
}

#[tokio::test]
async fn test_send_update_creates_only_a_when_ipv6_missing() {
    let mut data = ClientData::new(true, vec!["example.com".to_string()]);
    data.set_ipv4("100.100.100.100").expect("Should set ipv4");
    let (client, http_client, mut server) = new_test_client_with_data(data).await;

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(zone_list_response_ok())
        .create();

    let records_mock = server
        .mock("GET", "/zones/zone-id-123/dns_records?name=example.com")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_list_response(vec![]))
        .create();

    let create_mock = server
        .mock("POST", "/zones/zone-id-123/dns_records")
        .expect(1)
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_single_result_response(dns_record_a_json(
            "record-created-a",
            "example.com",
            "100.100.100.100",
        )))
        .create();

    client
        .send_update(&http_client)
        .await
        .expect("Should create only A record");

    zones_mock.assert();
    records_mock.assert();
    create_mock.assert();
}

#[tokio::test]
async fn test_send_update_creates_only_aaaa_when_ipv4_missing() {
    let mut data = ClientData::new(true, vec!["example.com".to_string()]);
    data.set_ipv6("fd00::dead").expect("Should set ipv6");
    let (client, http_client, mut server) = new_test_client_with_data(data).await;

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(zone_list_response_ok())
        .create();

    let records_mock = server
        .mock("GET", "/zones/zone-id-123/dns_records?name=example.com")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_list_response(vec![]))
        .create();

    let create_mock = server
        .mock("POST", "/zones/zone-id-123/dns_records")
        .expect(1)
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_single_result_response(dns_record_aaaa_json(
            "record-created-aaaa",
            "example.com",
            "fd00::dead",
        )))
        .create();

    client
        .send_update(&http_client)
        .await
        .expect("Should create only AAAA record");

    zones_mock.assert();
    records_mock.assert();
    create_mock.assert();
}

#[test]
fn test_base_domain() {
    assert_eq!(base_domain("com"), "");
    assert_eq!(base_domain("example.com"), "example.com");
    assert_eq!(base_domain("sub.example.com"), "example.com");
    assert_eq!(base_domain("foo.bar.example.com"), "example.com");
}

#[tokio::test]
async fn test_get_zone_id_api_error() {
    let (client, http_client, mut server) = new_test_client().await;

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(500)
        .with_body("Internal Server Error")
        .create();

    let e = client
        .get_zone_id(&http_client, "foo.example.com")
        .await
        .expect_err("Should fail with API error");
    assert!(
        e.to_string().contains("Failed to list zones"),
        "Expected error about listing zones but got: {e}"
    );

    zones_mock.assert();
}

#[tokio::test]
async fn test_update_record_aaaa() {
    let (client, http_client, mut server) = new_test_client().await;

    let record_mock = server
        .mock("PATCH", "/zones/zone-id-123/dns_records/record-id-aaaa")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_single_result_response(dns_record_aaaa_json(
            "record-id-aaaa",
            "foo.example.com",
            "fd00::dead",
        )))
        .create();

    client
        .update_record(
            &http_client,
            "zone-id-123",
            "foo.example.com",
            "record-id-aaaa",
            RecordType::AAAA,
        )
        .await
        .expect("Should update AAAA dns record");

    record_mock.assert();
}

#[tokio::test]
async fn test_create_record_aaaa() {
    let (client, http_client, mut server) = new_test_client().await;

    let record_mock = server
        .mock("POST", "/zones/zone-id-123/dns_records")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_single_result_response(dns_record_aaaa_json(
            "record-aaaa",
            "foo.example.com",
            "fd00::dead",
        )))
        .create();

    client
        .create_record(
            &http_client,
            "zone-id-123",
            "foo.example.com",
            RecordType::AAAA,
        )
        .await
        .expect("Should create AAAA dns record");

    record_mock.assert();
}

#[tokio::test]
async fn test_send_update_update_record_fails() {
    let (client, http_client, mut server) = new_test_client().await;

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(zone_list_response_ok())
        .create();

    let records = vec![dns_record_a_json("record-a", "example.com", "1.1.1.1")];
    let records_mock = server
        .mock("GET", "/zones/zone-id-123/dns_records?name=example.com")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_list_response(records))
        .create();

    let update_mock = server
        .mock("PATCH", "/zones/zone-id-123/dns_records/record-a")
        .with_status(500)
        .with_body("Internal Server Error")
        .create();

    let e = client
        .send_update(&http_client)
        .await
        .expect_err("Should fail when update_record fails");
    assert!(
        e.to_string()
            .contains("Failed to update A record for 'example.com'"),
        "Expected update error but got: {e}"
    );

    zones_mock.assert();
    records_mock.assert();
    update_mock.assert();
}

#[tokio::test]
async fn test_send_update_create_record_fails() {
    let (client, http_client, mut server) = new_test_client().await;

    let zones_mock = server
        .mock("GET", "/zones?name=example.com&status=active")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(zone_list_response_ok())
        .create();

    let records_mock = server
        .mock("GET", "/zones/zone-id-123/dns_records?name=example.com")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(dns_list_response(vec![]))
        .create();

    let create_mock = server
        .mock("POST", "/zones/zone-id-123/dns_records")
        .with_status(500)
        .with_body("Internal Server Error")
        .create();

    let e = client
        .send_update(&http_client)
        .await
        .expect_err("Should fail when create_record fails");
    assert!(
        e.to_string()
            .contains("Failed to create A record for 'example.com'"),
        "Expected create error but got: {e}"
    );

    zones_mock.assert();
    records_mock.assert();
    create_mock.assert();
}

async fn new_test_client() -> (Client, HttpClient, ServerGuard) {
    let mut data = ClientData::new(true, vec!["example.com".to_string()]);
    data.set_ipv4("100.100.100.100").expect("Should set ipv4");
    data.set_ipv6("fd00::dead").expect("Should set ipv6");

    new_test_client_with_data(data).await
}

async fn new_test_client_with_data(data: ClientData) -> (Client, HttpClient, ServerGuard) {
    let server = Server::new_async().await;
    let client = Client {
        api_url: server.url(),
        token: "test".to_string(),
        data,
    };
    (client, HttpClient::new(), server)
}

fn zone_list_response_ok() -> &'static str {
    r#"{
        "result": [
            {
                "id": "zone-id-123",
                "name": "example.com",
                "account": {
                    "id": "account-id-123",
                    "name": "test-account"
                },
                "activated_on": "2026-01-01T00:00:00Z",
                "created_on": "2026-01-01T00:00:00Z",
                "development_mode": 0,
                "meta": {
                    "custom_certificate_quota": 0,
                    "page_rule_quota": 0,
                    "phishing_detected": false
                },
                "modified_on": "2026-01-01T00:00:00Z",
                "name_servers": [],
                "owner": {
                    "type": "user",
                    "id": "owner-id-123",
                    "email": "owner@example.com"
                },
                "paused": false,
                "permissions": [],
                "status": "active",
                "type": "full"
            }
        ],
        "success": true,
        "errors": [],
        "messages": []
    }"#
}

fn zone_list_response_empty() -> &'static str {
    r#"{
        "result": [],
        "success": true,
        "errors": [],
        "messages": []
    }"#
}

fn dns_record_a_json(id: &str, name: &str, ip: &str) -> String {
    format!(
        r#"{{
            "meta": {{}},
            "name": "{name}",
            "ttl": 1,
            "modified_on": "2026-01-01T00:00:00Z",
            "created_on": "2026-01-01T00:00:00Z",
            "proxiable": true,
            "type": "A",
            "content": "{ip}",
            "id": "{id}",
            "proxied": true
        }}"#
    )
}

fn dns_record_aaaa_json(id: &str, name: &str, ip: &str) -> String {
    format!(
        r#"{{
            "meta": {{}},
            "name": "{name}",
            "ttl": 1,
            "modified_on": "2026-01-01T00:00:00Z",
            "created_on": "2026-01-01T00:00:00Z",
            "proxiable": true,
            "type": "AAAA",
            "content": "{ip}",
            "id": "{id}",
            "proxied": true
        }}"#
    )
}

fn dns_record_cname_json(id: &str, name: &str, target: &str) -> String {
    format!(
        r#"{{
            "meta": {{}},
            "name": "{name}",
            "ttl": 1,
            "modified_on": "2026-01-01T00:00:00Z",
            "created_on": "2026-01-01T00:00:00Z",
            "proxiable": true,
            "type": "CNAME",
            "content": "{target}",
            "id": "{id}",
            "proxied": true
        }}"#
    )
}

fn dns_list_response(records: Vec<String>) -> String {
    format!(
        r#"{{
            "result": [{}],
            "success": true,
            "errors": [],
            "messages": []
        }}"#,
        records.join(",")
    )
}

fn dns_single_result_response(record: String) -> String {
    format!(
        r#"{{
            "result": {record},
            "success": true,
            "errors": [],
            "messages": []
        }}"#
    )
}
