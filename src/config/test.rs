use super::*;
use std::env;

fn valid_config_1() -> Config {
    Config {
        log_level: "info".to_string(),
        server: ServerConfig {
            port: 8080,
            domains: vec!["example.org".to_string(), "example.net".to_string()],
            ssl: SSLConfig::default(),
        },
        client: ClientConfig {
            token: "test-token-1".to_string(),
            proxy: true,
            domains: vec!["foo.example.org".to_string()],
            interval: 300,
            endpoint: "dyndns.example.org".to_string(),
        },
    }
}

fn valid_config_2() -> Config {
    Config {
        log_level: "debug".to_string(),
        server: ServerConfig {
            port: 80,
            domains: vec!["example.com".to_string()],
            ssl: SSLConfig::default(),
        },
        client: ClientConfig {
            token: "test-token-2".to_string(),
            proxy: false,
            domains: vec!["bar.example.net".to_string()],
            interval: 600,
            endpoint: "dyndns.example.net".to_string(),
        },
    }
}

macro_rules! test_valid_config {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (path, mode, expected) = $value;
            let path = if path.is_empty() {
                path.to_string()
            } else {
                format!("src/config/testdata/{path}")
            };
            let config = Config::from_file(&path, mode, false).expect("Failed to load config");
            assert_eq!(expected, config, "Config did not match expected value");
        }
    )*
    }
}

test_valid_config! {
    test_config_valid_empty: (
        "",
        Mode::Server,
        Config::default(),
    ),
    test_config_valid_server_1: (
        "valid-config-1.yaml",
        Mode::Server,
        valid_config_1(),
    ),
    test_config_valid_server_2: (
        "valid-config-2.yaml",
        Mode::Server,
        valid_config_2(),
    ),
    test_config_valid_client_1: (
        "valid-config-1.yaml",
        Mode::Client,
        valid_config_1(),
    ),
    test_config_valid_client_2: (
        "valid-config-2.yaml",
        Mode::Client,
        valid_config_2(),
    ),
    test_config_valid_relay_1: (
        "valid-config-1.yaml",
        Mode::Relay,
        valid_config_1(),
    ),
    test_config_valid_relay_2: (
        "valid-config-2.yaml",
        Mode::Relay,
        valid_config_2(),
    ),
    test_config_valid_server_ssl: (
        "valid-config-ssl.yaml",
        Mode::Server,
        Config {
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            server: ServerConfig {
                port: 443,
                domains: vec![],
                ssl: SSLConfig {
                    enabled: true,
                    cert: "server.crt".to_string(),
                    key: "server.key".to_string(),
                },
            },
            client: ClientConfig::default(),
        },
    ),
}

macro_rules! test_invalid_config {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (path, mode, expected) = $value;
            let path = format!("src/config/testdata/{path}");
            let error = Config::from_file(&path, mode, false);
            if let Err(e) = &error {
                assert!(e.to_string().contains(expected), "Expected error message '{expected}' but got '{e}'");

            } else {
                panic!("Expected error but got success");
            }
        }
    )*
    }
}

test_invalid_config! {
    test_config_invalid_path: (
        "file-does-not-exist.yaml",
        Mode::Server,
        "Failed to read config file at 'src/config/testdata/file-does-not-exist.yaml'",
    ),
    test_config_not_yaml: (
        "not-a-config.txt",
        Mode::Server,
        "Failed to parse config file.",
    ),
    test_config_client_missing_token: (
        "invalid-config-1.yaml",
        Mode::Client,
        "Client token cannot be empty",
    ),
    test_config_client_empty_domains: (
        "invalid-config-2.yaml",
        Mode::Client,
        "Client domains cannot be empty",
    ),
    test_config_client_wrong_interval: (
        "invalid-config-3.yaml",
        Mode::Client,
        "Failed to parse config file.",
    ),
    test_config_client_invalid_interval: (
        "invalid-config-5.yaml",
        Mode::Client,
        "Client interval cannot be less than 30 seconds",
    ),
    test_config_relay_missing_token: (
        "invalid-config-1.yaml",
        Mode::Relay,
        "Client token cannot be empty",
    ),
    test_config_relay_empty_domains: (
        "invalid-config-2.yaml",
        Mode::Relay,
        "Client domains cannot be empty",
    ),
    test_config_relay_wrong_interval: (
        "invalid-config-3.yaml",
        Mode::Relay,
        "Failed to parse config file.",
    ),
    test_config_relay_empty_endpoint: (
        "invalid-config-4.yaml",
        Mode::Relay,
        "Client endpoint cannot be empty",
    ),
    test_config_relay_invalid_interval: (
        "invalid-config-5.yaml",
        Mode::Relay,
        "Client interval cannot be less than 30 seconds",
    ),
    test_config_server_ssl_missing_key: (
        "invalid-config-ssl-1.yaml",
        Mode::Server,
        "SSL is enabled but key is empty",
    ),
    test_config_server_ssl_missing_cert: (
        "invalid-config-ssl-2.yaml",
        Mode::Server,
        "SSL is enabled but cert is empty",
    ),
}

#[test]
fn test_config_environment_variable_expansion() {
    let expected = Config {
        log_level: "debug".to_string(),
        server: ServerConfig {
            port: 2080,
            domains: vec!["example.org".to_string(), "example.net".to_string()],
            ssl: SSLConfig::default(),
        },
        client: ClientConfig {
            token: "token-from-env".to_string(),
            proxy: true,
            domains: vec!["foo.example.org".to_string()],
            interval: 900,
            endpoint: "dyndns.example.org".to_string(),
        },
    };
    unsafe {
        let c = &expected;
        env::set_var("DYNDNS_TEST_LOG_LEVEL", c.log_level.clone());
        env::set_var("DYNDNS_TEST_SERVER_PORT", c.server.port.to_string());
        env::set_var("DYNDNS_TEST_SERVER_DOMAIN1", c.server.domains[0].clone());
        env::set_var("DYNDNS_TEST_SERVER_DOMAIN2", c.server.domains[1].clone());
        env::set_var("DYNDNS_TEST_CLIENT_TOKEN", c.client.token.clone());
        env::set_var("DYNDNS_TEST_CLIENT_PROXY", c.client.proxy.to_string());
        env::set_var("DYNDNS_TEST_CLIENT_DOMAIN", c.client.domains[0].clone());
        env::set_var("DYNDNS_TEST_CLIENT_INTERVAL", c.client.interval.to_string());
        env::set_var("DYNDNS_TEST_CLIENT_ENDPOINT", c.client.endpoint.clone());
    }

    let config = Config::from_file("src/config/testdata/env-config.yaml", Mode::Server, true)
        .expect("Failed to load config with environment variable expansion");

    assert_eq!(
        expected, config,
        "Config did not match expected value after environment variable expansion"
    );

    // Clean up environment variables
    unsafe {
        env::remove_var("DYNDNS_TEST_LOG_LEVEL");
        env::remove_var("DYNDNS_TEST_SERVER_PORT");
        env::remove_var("DYNDNS_TEST_SERVER_DOMAIN1");
        env::remove_var("DYNDNS_TEST_SERVER_DOMAIN2");
        env::remove_var("DYNDNS_TEST_CLIENT_TOKEN");
        env::remove_var("DYNDNS_TEST_CLIENT_PROXY");
        env::remove_var("DYNDNS_TEST_CLIENT_DOMAIN");
        env::remove_var("DYNDNS_TEST_CLIENT_INTERVAL");
        env::remove_var("DYNDNS_TEST_CLIENT_ENDPOINT");
    }
}

macro_rules! test_set_log_level {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (level, should_error) = $value;
            assert_eq!(should_error, set_log_level(level).is_err(), "set_log_level did not return expected error status for level '{level}'");
        }
    )*
    }
}

test_set_log_level! {
    test_set_log_level_debug: ("debug", false),
    test_set_log_level_info: ("info", false),
    test_set_log_level_warn: ("warn", false),
    test_set_log_level_error: ("error", false),
    test_set_log_level_invalid: ("unknown", true),
}
