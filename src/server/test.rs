use std::time::Duration;

use tokio::time::sleep;

use super::*;

#[test]
fn test_verify_domains_allowed() {
    let s = shared_state(vec!["example.org", "example.net"]);
    let d = vec![
        "foo.example.org".to_string(),
        "bar.example.net".to_string(),
        "example.net".to_string(),
    ];
    assert!(s.verify_domains(&d));
}

#[test]
fn test_verify_domains_whitelist_empty() {
    let s = shared_state(vec![]);
    let d = vec![
        "foo.example.org".to_string(),
        "bar.example.net".to_string(),
        "example.net".to_string(),
    ];
    assert!(s.verify_domains(&d));
}

#[test]
fn test_verify_domains_forbidden() {
    let s = shared_state(vec!["example.org"]);
    let d = vec![
        "foo.example.org".to_string(),
        "bar.example.net".to_string(),
        "example.net".to_string(),
    ];
    assert!(!s.verify_domains(&d));
}

#[test]
fn test_verify_domains_partial_match() {
    let s = shared_state(vec!["example.org"]);
    let d = vec![
        "foo.example.org".to_string(),
        "bar.notexample.org".to_string(),
        "notexample.org".to_string(),
    ];
    assert!(!s.verify_domains(&d));
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_handler_forbidden_domains() {
    let state = shared_state(vec!["example.net"]);
    let payload = request_data();

    let (status, Json(response)) = update_handler(state, payload).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(response.msg, MESSAGE_DOMAINS_FORBIDDEN);
    assert!(!response.success);
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_handler_invalid_ipv4() {
    let state = shared_state(vec![]);
    let mut payload = request_data();
    payload.ipv4 = "Not an IPv4".to_string();

    let (status, Json(response)) = update_handler(state, payload).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response.msg, "Invalid IPv4 address: 'Not an IPv4'");
    assert!(!response.success);
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_handler_invalid_ipv6() {
    let state = shared_state(vec![]);
    let mut payload = request_data();
    payload.ipv6 = "Not an IPv6".to_string();

    let (status, Json(response)) = update_handler(state, payload).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response.msg, "Invalid IPv6 address: 'Not an IPv6'");
    assert!(!response.success);
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_handler_missing_ip() {
    let state = shared_state(vec![]);
    let mut payload = request_data();
    payload.ipv4.clear();
    payload.ipv6.clear();

    let (status, Json(response)) = update_handler(state, payload).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response.msg, "At least one IP address must be provided");
    assert!(!response.success);
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_handler_missing_domains() {
    let state = shared_state(vec![]);
    let mut payload = request_data();
    payload.domains.clear();

    let (status, Json(response)) = update_handler(state, payload).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response.msg, "At least one domain must be provided");
    assert!(!response.success);
}

#[tokio::test(flavor = "current_thread")]
async fn test_update_handler_missing_token() {
    let state = shared_state(vec![]);
    let mut payload = request_data();
    payload.token.clear();

    let (status, Json(response)) = update_handler(state, payload).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(response.msg, MESSAGE_UNAUTHORIZED);
    assert!(!response.success);
}

#[tokio::test]
async fn test_endpoint_healthz() {
    let server = Server::from_config(ServerConfig {
        port: 0,
        domains: vec![],
        ssl: Default::default(),
    });
    let mut s = server.clone();

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let server_task = tokio::spawn(async move {
        s.run(Some(shutdown_rx))
            .await
            .expect("Should run the server")
    });

    let s = server.clone();
    let client_task = tokio::spawn(async move {
        sleep(std::time::Duration::from_millis(100)).await;
        let port = s.current_port.load(std::sync::atomic::Ordering::SeqCst);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(100))
            .build()
            .expect("Should build client");
        let res = client
            .get(format!("http://localhost:{port}/healthz"))
            .send()
            .await
            .expect("Should get a response");

        assert!(
            res.status().is_success(),
            "Health check should return 200 Ok"
        );
        assert_eq!(
            "Ok",
            res.text().await.expect("Should return response body"),
            "Health check should return 'Ok'"
        );
    });

    let shutdown_task = tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        shutdown_tx.send(()).expect("Should shutdown the server");
    });

    let (server_res, client_res, shutdown_res) =
        tokio::join!(server_task, client_task, shutdown_task);
    server_res.expect("Server should join without error");
    client_res.expect("Client should join without error");
    shutdown_res.expect("Shutdown should join without error");
}

#[tokio::test]
async fn test_endpoint_update_get() {
    let server = Server::from_config(ServerConfig {
        port: 0,
        domains: vec!["example.org".to_string()],
        ssl: Default::default(),
    });
    let mut s = server.clone();

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let server_task = tokio::spawn(async move {
        s.run(Some(shutdown_rx))
            .await
            .expect("Should run the server")
    });

    let s = server.clone();
    let client_task = tokio::spawn(async move {
        sleep(std::time::Duration::from_millis(100)).await;
        let port = s.current_port.load(std::sync::atomic::Ordering::SeqCst);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(100))
            .build()
            .expect("Should build client");
        let res = client
            .get(format!(
                "http://localhost:{port}/?domains=foo.example.org,bar.example.net&proxy=true"
            ))
            .send()
            .await
            .expect("Should get a response");

        assert_eq!(
            reqwest::StatusCode::FORBIDDEN,
            res.status(),
            "Update check should return 403 Forbidden"
        );
        let res: ResponseMessage = res
            .json()
            .await
            .expect("Should parse response body as JSON");
        assert_eq!(
            MESSAGE_DOMAINS_FORBIDDEN, res.msg,
            "Update check should return correct error message"
        );
        assert!(
            !res.success,
            "Update check should return success=false for forbidden domains"
        );
    });

    let shutdown_task = tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        shutdown_tx.send(()).expect("Should shutdown the server");
    });

    let (server_res, client_res, shutdown_res) =
        tokio::join!(server_task, client_task, shutdown_task);
    server_res.expect("Server should join without error");
    client_res.expect("Client should join without error");
    shutdown_res.expect("Shutdown should join without error");
}

#[tokio::test]
async fn test_endpoint_update_post() {
    let server = Server::from_config(ServerConfig {
        port: 0,
        domains: vec!["example.org".to_string()],
        ssl: Default::default(),
    });
    let mut s = server.clone();

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let server_task = tokio::spawn(async move {
        s.run(Some(shutdown_rx))
            .await
            .expect("Should run the server")
    });

    let s = server.clone();
    let client_task = tokio::spawn(async move {
        sleep(std::time::Duration::from_millis(100)).await;
        let port = s.current_port.load(std::sync::atomic::Ordering::SeqCst);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(100))
            .build()
            .expect("Should build client");
        let payload = RequestData {
            token: String::new(),
            domains: vec!["foo.example.org".to_string(), "bar.example.net".to_string()],
            ipv4: String::new(),
            ipv6: String::new(),
            proxy: true,
        };
        let res = client
            .post(format!("http://localhost:{port}"))
            .json(&payload)
            .send()
            .await
            .expect("Should get a response");

        assert_eq!(
            reqwest::StatusCode::FORBIDDEN,
            res.status(),
            "Update check should return 403 Forbidden"
        );
        let res: ResponseMessage = res
            .json()
            .await
            .expect("Should parse response body as JSON");
        assert_eq!(
            MESSAGE_DOMAINS_FORBIDDEN, res.msg,
            "Update check should return correct error message"
        );
        assert!(
            !res.success,
            "Update check should return success=false for forbidden domains"
        );
    });

    let shutdown_task = tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        shutdown_tx.send(()).expect("Should shutdown the server");
    });

    let (server_res, client_res, shutdown_res) =
        tokio::join!(server_task, client_task, shutdown_task);
    server_res.expect("Server should join without error");
    client_res.expect("Client should join without error");
    shutdown_res.expect("Shutdown should join without error");
}

fn request_data() -> RequestData {
    RequestData {
        token: "testtoken".to_string(),
        domains: vec!["foo.example.org".to_string()],
        ipv4: "100.100.100.100".to_string(),
        ipv6: "fd00::dead".to_string(),
        proxy: true,
    }
}

fn shared_state(domains: Vec<&str>) -> SharedState {
    SharedState {
        domains: domains
            .into_iter()
            .map(|domain| domain.to_string())
            .collect(),
    }
}

#[test]
fn test_server_from_config() {
    let config = ServerConfig {
        port: 9090,
        domains: vec!["example.org".to_string()],
        ssl: Default::default(),
    };
    let server = Server::from_config(config.clone());
    assert_eq!(server.options.port, config.port, "Port should match config");
    assert_eq!(
        server.options.domains, config.domains,
        "Domains should match config"
    );
    assert_eq!(
        server.options.ssl.enabled, config.ssl.enabled,
        "SSL setting should match config"
    );
}
