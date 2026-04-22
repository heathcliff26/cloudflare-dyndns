use std::io;
use tokio::runtime::{Builder, Runtime};

#[cfg(test)]
mod test;

/// Create a single threaded async runtime
pub fn single_threaded_runtime() -> io::Result<Runtime> {
    new_runtime(Builder::new_current_thread())
}

/// Create a multi threaded async runtime
pub fn multi_threaded_runtime() -> io::Result<Runtime> {
    new_runtime(Builder::new_multi_thread())
}

/// Configure the runtime with the correct features
fn new_runtime(mut rt: Builder) -> io::Result<Runtime> {
    rt.name("cloudflare-dyndns-runtime")
        .enable_io()
        .enable_time()
        .build()
}
