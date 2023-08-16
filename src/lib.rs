#![warn(missing_debug_implementations, rust_2018_idioms)]

mod client;
pub mod error;
pub mod proto;

pub use client::{ClientConfig, InnerClient};

#[cfg(feature = "seabird-client")]
pub use client::SeabirdClient;

#[cfg(feature = "chat-ingest-client")]
pub use client::ChatIngestClient;
