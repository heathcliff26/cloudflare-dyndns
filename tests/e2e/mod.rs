use reqwest::Client;
use std::process::Command;
use std::sync::Once;

static CONTAINER_BUILD: Once = Once::new();
static CONTAINER_IMAGE: &str = "localhost/cloudflare-dyndns:e2e-test";

#[tokio::test]
async fn server_healthcheck_http() {
    let _container = RunningContainer::setup(
        "cloudflare-dyndns-http",
        "./tests/e2e/testdata/http/",
        "server",
        "",
    )
    .await;

    let url = "http://localhost:8080/healthz";

    let response = Client::new()
        .get(url)
        .send()
        .await
        .expect("Should perform health check request");
    assert!(
        response.status().is_success(),
        "Health check failed: {}",
        response.status()
    );
}

#[tokio::test]
async fn server_healthcheck_https() {
    let server_cert = TlsCertificate::create("tests/e2e/testdata/https/server");
    let _container = RunningContainer::setup(
        "cloudflare-dyndns-https",
        "./tests/e2e/testdata/https/",
        "server",
        "",
    )
    .await;

    let url = "https://localhost:8443/healthz";

    let certificate = server_cert.certificate();
    let response = Client::builder()
        .add_root_certificate(certificate)
        .build()
        .expect("Failed to build HTTPS client")
        .get(url)
        .send()
        .await
        .expect("Should perform health check request");
    assert!(
        response.status().is_success(),
        "Health check failed: {}",
        response.status()
    );
}

#[tokio::test]
async fn relay_update() {
    let mut server = mockito::Server::new_async().await;
    let update_mock = server
        .mock("POST", "/")
        .expect(1)
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"msg": "Updated dyndns records","success": true}"#)
        .create();

    let _container = RunningContainer::setup(
        "cloudflare-dyndns-relay",
        "./tests/e2e/testdata/relay/",
        "relay",
        &server.url(),
    )
    .await;

    update_mock.assert();
}

fn build_image() {
    CONTAINER_BUILD.call_once(|| {
        // This function is called only once, even if multiple threads call it.
        // Here you would put the code to build your container image.
        println!("Building container image...");

        let output = Command::new("podman")
            .args(["build", "-t", CONTAINER_IMAGE, "."])
            .output()
            .expect("Failed to execute podman build command");

        if !output.status.success() {
            panic!(
                "Failed to build container image: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!("Container image built successfully.");
    });
}

struct RunningContainer {
    name: String,
}

impl RunningContainer {
    /// Start a container
    async fn setup(name: &str, config_dir: &str, mode: &str, relay_endpoint: &str) -> Self {
        build_image();

        println!("Starting container: {}", name);
        let output = Command::new("podman")
            .args([
                "run",
                "-d",
                "--net",
                "host",
                "--name",
                name,
                "-e",
                format!("E2E_RELAY_ENDPOINT={relay_endpoint}").as_str(),
                "-v",
                format!("{config_dir}:/config:z").as_str(),
                CONTAINER_IMAGE,
                mode,
                "--config",
                "/config/config.yaml",
                "--env",
            ])
            .output()
            .expect("Failed to execute podman run command");

        if !output.status.success() {
            panic!(
                "Failed to start container: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        println!("Container {} started successfully.", name);
        RunningContainer {
            name: name.to_string(),
        }
    }

    /// Print the container log
    fn log(&self) {
        println!("Fetching logs for container: {}", self.name);
        let output = Command::new("podman")
            .args(["logs", &self.name])
            .output()
            .expect("Failed to execute podman logs command");

        if !output.status.success() {
            panic!(
                "Failed to fetch logs for container: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!(
            "Logs for container {}:\nstdout:\n{}stderr:\n{}",
            self.name,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

impl Drop for RunningContainer {
    /// Stop and remove the container.
    fn drop(&mut self) {
        self.log();
        println!("Stopping and removing container: {}", self.name);
        let output = Command::new("podman")
            .args(["rm", "-f", &self.name])
            .output()
            .expect("Failed to execute podman rm command");

        if !output.status.success() {
            panic!(
                "Failed to remove container: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!("Container {} removed successfully.", self.name);
    }
}

/// Randomly generated self-signed TLS certificate and key pair.
/// Will be cleaned up when it goes out of scope.
pub struct TlsCertificate {
    pub key: String,
    pub crt: String,
}

impl TlsCertificate {
    /// Create a self signed TLS certificate and key pair.
    pub fn create(name: &str) -> Self {
        let key = format!("{name}.key");
        let crt = format!("{name}.crt");
        println!("Creating TLS certificate '{crt}' and key '{key}' ");
        let output = Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-nodes",
                "-days",
                "1",
                "-newkey",
                "rsa:2048",
                "-keyout",
                &key,
                "-out",
                &crt,
                "-subj",
                "/CN=localhost",
            ])
            .output()
            .expect("Failed to execute openssl command");

        if !output.status.success() {
            panic!(
                "Failed to create TLS certificate: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let output = Command::new("chmod")
            .args(["644", &key])
            .output()
            .expect("Failed to execute chmod command");
        if !output.status.success() {
            panic!(
                "Failed to set permissions for TLS key: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!("TLS certificate created successfully.");
        TlsCertificate { key, crt }
    }
    /// Returns the certificate as a reqwest::tls::Certificate
    pub fn certificate(&self) -> reqwest::tls::Certificate {
        let cert_data = std::fs::read(&self.crt).expect("Failed to read TLS certificate file");
        reqwest::tls::Certificate::from_pem(&cert_data)
            .expect("Failed to create TLS certificate from PEM data")
    }
}

impl Drop for TlsCertificate {
    fn drop(&mut self) {
        println!("Removing TLS certificate: {}", self.crt);

        let res_key = std::fs::remove_file(&self.key);
        let res_crt = std::fs::remove_file(&self.crt);
        res_key.expect("Failed to remove TLS key file");
        res_crt.expect("Failed to remove TLS certificate file");

        println!("TLS certificate removed successfully.");
    }
}
