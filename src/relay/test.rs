use mockito::{Mock, Server, ServerGuard};

use super::*;

#[test]
fn test_relay_from_config() {
    let mut config = ClientConfig::default();
    config.endpoint = "http://example.com/".to_string();
    config.token = "testtoken".to_string();
    config.domains = vec!["example.com".to_string()];

    let relay = Relay::from_config(config.clone());
    assert_eq!(
        config.endpoint, relay.endpoint,
        "Endpoint should match the config"
    );
    assert_eq!(config.token, relay.token, "Token should match the config");
    assert_eq!(
        config.domains, relay.data.domains,
        "Domains should match the config"
    );
    assert_eq!(
        config.proxy, relay.data.proxy,
        "Proxy setting should match the config"
    );
}

#[tokio::test]
async fn test_relay_send_update_invalid_data() {
    let data = ClientData::new(true, vec![]);
    let relay = Relay {
        endpoint: "http://example.com/".to_string(),
        token: "testtoken".to_string(),
        data,
    };
    let e = relay
        .send_update(&Client::new())
        .await
        .expect_err("Should fail to send update");
    assert!(
        e.to_string().contains("Invalid client data"),
        "Should return expected error: {e:#}"
    );
}

#[tokio::test]
async fn test_relay_send_update_server_down() {
    let data = ClientData::new(true, vec!["example.com".to_string()]);
    let mut relay = Relay {
        endpoint: "https://invalid-url-for-testing".to_string(),
        token: "testtoken".to_string(),
        data,
    };
    relay
        .data_mut()
        .set_ipv4("100.100.100.100")
        .expect("Should set ipv4");

    let e = relay
        .send_update(&Client::new())
        .await
        .expect_err("Should fail to send update");
    assert!(
        e.to_string()
            .contains("Failed to send update to relay server"),
        "Should return expected error: {e:#}"
    );
}

#[tokio::test]
async fn test_relay_send_update_invalid_response() {
    let (relay, _server, mock) = new_test_relay_server_and_mock().await;
    let mock = mock
        .with_status(200)
        .with_body("Not a valid JSON response")
        .create();

    let e = relay
        .send_update(&Client::new())
        .await
        .expect_err("Should fail to send update");
    assert!(
        e.to_string()
            .contains("Failed to parse response from relay server"),
        "Should return expected error: {e:#}"
    );

    mock.assert();
}

#[tokio::test]
async fn test_relay_send_update_failed_status() {
    let (relay, _server, mock) = new_test_relay_server_and_mock().await;
    let mock = mock
        .with_status(500)
        .with_body(r#"{"msg": "Updated dyndns records","success": true}"#)
        .create();

    let e = relay
        .send_update(&Client::new())
        .await
        .expect_err("Should fail to send update");
    assert!(
        e.to_string().contains("Failed to update records, status:"),
        "Should return expected error: {e:#}"
    );

    mock.assert();
}

#[tokio::test]
async fn test_relay_send_update_success_false() {
    let (relay, _server, mock) = new_test_relay_server_and_mock().await;
    let mock = mock
        .with_status(200)
        .with_body(r#"{"msg": "Updated dyndns records","success": false}"#)
        .create();

    let e = relay
        .send_update(&Client::new())
        .await
        .expect_err("Should fail to send update");
    assert!(
        e.to_string().contains("Failed to update records, status:"),
        "Should return expected error: {e:#}"
    );

    mock.assert();
}

#[tokio::test]
async fn test_relay_send_update_success() {
    let (relay, _server, mock) = new_test_relay_server_and_mock().await;
    let mock = mock
        .with_status(200)
        .with_body(r#"{"msg": "Updated dyndns records","success": true}"#)
        .create();

    relay
        .send_update(&Client::new())
        .await
        .expect("Should succeed with update");

    mock.assert();
}

async fn new_test_relay_server_and_mock() -> (Relay, ServerGuard, Mock) {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/")
        .match_header("Content-Type", "application/json")
        .with_header("Content-Type", "application/json");

    println!("Mock server running at: {}", server.url());
    let data = ClientData::new(true, vec!["example.com".to_string()]);
    let mut relay = Relay {
        endpoint: server.url(),
        token: "testtoken".to_string(),
        data,
    };
    relay
        .data_mut()
        .set_ipv4("100.100.100.100")
        .expect("Should set ipv4");
    (relay, server, mock)
}
