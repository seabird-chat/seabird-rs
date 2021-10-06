#![warn(missing_debug_implementations, rust_2018_idioms)]

mod client;
pub mod error;
pub mod proto;

pub use client::{Client, ClientConfig, InnerClient};
