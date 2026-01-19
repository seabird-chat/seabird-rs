//! Protocol buffer definitions for seabird.
//!
//! This module contains the auto-generated gRPC and Protocol Buffer types
//! used to communicate with seabird servers. The types are generated at
//! build time from `.proto` files in the `proto/` directory.
//!
//! # Modules
//!
//! - [`common`]: Common types shared across seabird protocols
//! - [`seabird`]: Main seabird protocol types and client definitions

pub mod common {
    tonic::include_proto!("common");
}

pub use self::common::*;

pub mod seabird {
    tonic::include_proto!("seabird");
}

pub use self::seabird::*;
