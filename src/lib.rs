#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, rust_2018_idioms)]

//! A client library for the seabird-chat ecosystem.
//!
//! This crate provides gRPC clients for interacting with seabird chat servers,
//! supporting both the main seabird protocol and the chat ingest protocol.
//!
//! # Features
//!
//! - `seabird-client` (default): Enables the main SeabirdClient for bot interactions
//! - `chat-ingest-client`: Enables the ChatIngestClient for ingesting chat data
//!
//! # Example
//!
//! ```rust,no_run
//! use seabird::{ClientConfig, SeabirdClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ClientConfig {
//!         url: "https://seabird.example.com".to_string(),
//!         token: "your-token-here".to_string(),
//!     };
//!
//!     let mut client = SeabirdClient::new(config).await?;
//!     client.send_message("channel-id", "Hello, world!", None).await?;
//!
//!     Ok(())
//! }
//! ```

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
