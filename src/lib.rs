#![warn(missing_debug_implementations, rust_2018_idioms)]

mod block;
mod client;
pub mod error;
pub mod proto;

pub use block::Block;
pub use client::{ClientConfig, InnerClient};

#[cfg(feature = "seabird-client")]
pub use client::SeabirdClient;

#[cfg(feature = "chat-ingest-client")]
pub use client::ChatIngestClient;
