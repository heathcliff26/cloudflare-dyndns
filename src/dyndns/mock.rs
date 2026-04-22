use std::sync::{
    Arc,
    atomic::{self, AtomicU64},
};

use crate::dyndns::{ClientData, DynDnsClient};
use anyhow::{Result, bail};

/// A mock implementation of the Client trait for testing purposes.
#[allow(dead_code)]
pub struct MockClient {
    data: ClientData,
    pub success: bool,
    pub update_counter: Arc<AtomicU64>,
}

impl MockClient {
    /// Create a new MockClient instance.
    pub fn new(data: ClientData, success: bool) -> Self {
        Self {
            data,
            success,
            update_counter: Arc::new(AtomicU64::new(0)),
        }
    }
    /// Return the current counter
    pub fn counter(&self) -> u64 {
        self.update_counter.load(atomic::Ordering::SeqCst)
    }
}

impl DynDnsClient for MockClient {
    fn data(&self) -> &ClientData {
        &self.data
    }
    fn data_mut(&mut self) -> &mut ClientData {
        &mut self.data
    }
    async fn send_update(&self, _: &reqwest::Client) -> Result<()> {
        self.update_counter.fetch_add(1, atomic::Ordering::SeqCst);
        if self.success {
            Ok(())
        } else {
            bail!("Mock update failed")
        }
    }
}
