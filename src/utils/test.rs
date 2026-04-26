use super::*;

#[test]
fn test_new_http_client() {
    // Verify that the client is created successfully
    let client = new_http_client();
    drop(client);
}

#[tokio::test]
async fn test_new_interrupt_signal_with_stop_rx() {
    let (stop_tx, stop_rx) = watch::channel(());
    let mut shutdown_rx = new_interrupt_signal(Some(stop_rx));

    // Trigger shutdown via the stop channel
    stop_tx.send(()).expect("Should send stop signal");

    // The shutdown signal should arrive
    shutdown_rx
        .changed()
        .await
        .expect("Should receive shutdown signal");
}

#[tokio::test]
async fn test_new_interrupt_signal_without_stop_rx() {
    // Verify that a signal listener is created without panicking
    let _shutdown_rx = new_interrupt_signal(None);
}
