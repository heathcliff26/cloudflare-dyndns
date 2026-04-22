use tokio::time::timeout;

use super::*;

#[test]
fn test_client_data_set_ipv4() {
    let test_cases = vec![("", true), ("not an ip", false), ("100.100.100.100", true)];
    let mut data = ClientData::new(false, vec![]);

    for (input, should_pass) in test_cases {
        let result = data.set_ipv4(input);
        if should_pass {
            assert!(
                result.is_ok(),
                "Expected valid IPv4 address but got error: {input}"
            );
            assert_eq!(input, data.ipv4(), "IP should match input");
        } else {
            assert!(
                result.is_err(),
                "Expected invalid IPv4 address but got success: {input}"
            );
            assert_ne!(input, data.ipv4(), "IP should not match input");
        }
    }
}

#[test]
fn test_client_data_set_ipv6() {
    let test_cases = vec![("", true), ("not an ip", false), ("fd00::dead", true)];
    let mut data = ClientData::new(false, vec![]);

    for (input, should_pass) in test_cases {
        let result = data.set_ipv6(input);
        if should_pass {
            assert!(
                result.is_ok(),
                "Expected valid IPv6 address but got error: {input}"
            );
            assert_eq!(input, data.ipv6(), "IP should match input");
        } else {
            assert!(
                result.is_err(),
                "Expected invalid IPv6 address but got success: {input}"
            );
            assert_ne!(input, data.ipv6(), "IP should not match input");
        }
    }
}

#[test]
fn test_client_data_check() {
    let mut data = ClientData::new(false, vec![]);

    // Testing with no IPs
    if let Err(e) = data.check() {
        assert!(
            e.to_string()
                .contains("At least one IP address must be provided"),
            "Received wrong error: {e}"
        );
    } else {
        panic!("Expected error but got success");
    }

    // Testing with no Domains and IPv4 only
    data.set_ipv4("100.100.100.100").expect("Should set IP");
    if let Err(e) = data.check() {
        assert!(
            e.to_string()
                .contains("At least one domain must be provided"),
            "Received wrong error: {e}"
        );
    } else {
        panic!("Expected error but got success");
    }

    // Testing with no Domains and dual stack
    data.set_ipv6("fd00::dead").expect("Should set IP");
    if let Err(e) = data.check() {
        assert!(
            e.to_string()
                .contains("At least one domain must be provided"),
            "Received wrong error: {e}"
        );
    } else {
        panic!("Expected error but got success");
    }

    // Testing with no Domains and IPv6 only
    data.set_ipv4("").expect("Should set IP");
    if let Err(e) = data.check() {
        assert!(
            e.to_string()
                .contains("At least one domain must be provided"),
            "Received wrong error: {e}"
        );
    } else {
        panic!("Expected error but got success");
    }

    // Testing with Domains set
    data.domains.push("example.com".to_string());
    assert!(data.check().is_ok(), "Should pass with 1 IP and 1 domain");
}

#[tokio::test]
async fn test_update() {
    let data = ClientData::new(false, vec!["foo.example.com".to_string()]);

    let mut client = mock::MockClient::new(data, true);

    client
        .update(false)
        .await
        .expect("First update should succeed");
    assert_eq!(client.counter(), 1, "Update counter should be 1");
    assert!(!client.data().ipv4().is_empty(), "IPv4 should be set");

    client
        .update(false)
        .await
        .expect("Second update should succeed");
    assert_eq!(client.counter(), 1, "Update counter should stay 1");

    client.success = false;
    client.update(true).await.expect_err("Update should fail");
    assert_eq!(client.counter(), 2, "Update counter should be 2");
}

#[tokio::test]
async fn test_run_failure() {
    let data = ClientData::new(false, vec!["foo.example.com".to_string()]);

    let mut client = mock::MockClient::new(data, false);
    let counter = client.update_counter.clone();

    let (stop_tx, stop_rx) = watch::channel(());

    // Run the client in the background
    let handle = tokio::spawn(async move {
        client.run(1, stop_rx).await;
    });

    // Wait for a few intervals and then check the counter
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // The counter should be at least 2 (initial update + at least one interval update)
    assert!(
        counter.load(std::sync::atomic::Ordering::SeqCst) >= 2,
        "Should have called update at least twice"
    );

    stop_tx.send(()).expect("Should send stop signal");

    timeout(Duration::from_secs(1), handle)
        .await
        .expect("Client should stop")
        .expect("Should join client handle");
}

#[tokio::test]
async fn test_run_success() {
    let data = ClientData::new(false, vec!["foo.example.com".to_string()]);

    let mut client = mock::MockClient::new(data, true);
    let counter = client.update_counter.clone();

    let (stop_tx, stop_rx) = watch::channel(());

    // Run the client in the background
    let handle = tokio::spawn(async move {
        client.run(1, stop_rx).await;
    });

    // Wait for a few intervals and then check the counter
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // The counter should be at least 2 (initial update + at least one interval update)
    assert!(
        counter.load(std::sync::atomic::Ordering::SeqCst) == 1,
        "Should have called update once"
    );

    stop_tx.send(()).expect("Should send stop signal");

    timeout(Duration::from_secs(1), handle)
        .await
        .expect("Client should stop")
        .expect("Should join client handle");
}

#[tokio::test]
async fn test_get_ipv4() {
    let client = reqwest::Client::new();
    get_public_ipv4(&client)
        .await
        .expect("Should get public ip address");
}

#[tokio::test]
async fn test_get_ipv6() {
    let client = reqwest::Client::new();
    let ip = get_public_ipv6(&client)
        .await
        .expect("Should get public ip address");
    if has_ipv6_support().await {
        assert!(
            !ip.is_empty(),
            "IPv6 support detected but no address returned"
        );
    } else {
        assert!(
            ip.is_empty(),
            "No IPv6 support detected but address returned"
        );
    }
}

#[test]
fn test_validate_ipv4() {
    let test_cases = vec![
        ("", false),
        ("not an ip", false),
        ("100.100.100.100", true),
        ("172.198.10.100", true),
        ("fd00::dead", false),
    ];

    for (input, should_pass) in test_cases {
        let result = validate_ipv4(input);
        if should_pass {
            assert!(
                result.is_ok(),
                "Expected valid IPv4 address but got error: {input}"
            );
        } else {
            assert!(
                result.is_err(),
                "Expected invalid IPv4 address but got success: {input}"
            );
        }
    }
}

#[test]
fn test_validate_ipv6() {
    let test_cases = vec![
        ("", false),
        ("not an ip", false),
        ("100.100.100.100", false),
        ("fd69::dead", true),
        ("fd00::dead", true),
    ];

    for (input, should_pass) in test_cases {
        let result = validate_ipv6(input);
        if should_pass {
            assert!(
                result.is_ok(),
                "Expected valid IPv6 address but got error: {input}"
            );
        } else {
            assert!(
                result.is_err(),
                "Expected invalid IPv6 address but got success: {input}"
            );
        }
    }
}
