use reqwest::Client;
use std::time::Duration;
use tokio::{select, signal, sync::watch};

/// Listen for SIGINT or SIGTERM signals and notify over the returned channel.
/// Should be used to gracefully shutdown application loops.
pub fn new_interrupt_signal(stop_rx: Option<watch::Receiver<()>>) -> watch::Receiver<()> {
    let (shutdown_tx, shutdown_rx) = watch::channel(());
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler");
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("Failed to install SIGINT handler");

        match stop_rx {
            Some(stop_rx) => {
                let mut stop_rx = stop_rx.clone();
                select! {
                    _ = sigterm.recv() => {},
                    _ = sigint.recv() => {},
                    _ = stop_rx.changed() => {},
                }
            }
            None => {
                select! {
                    _ = sigterm.recv() => {},
                    _ = sigint.recv() => {},
                }
            }
        }
        shutdown_tx
            .send(())
            .expect("Failed to send shutdown signal");
    });
    shutdown_rx
}

/// Create a new http client with default timeout of 10 seconds.
pub fn new_http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
}
