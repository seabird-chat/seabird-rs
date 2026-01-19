//! Error types for the seabird client library.
//!
//! Currently wraps `anyhow::Error` for flexible error handling.
//! This may be replaced with more concrete error types in the future.

// TODO: replace these with better, more concrete error types.

/// The error type used throughout this crate.
///
/// This is currently an alias to [`anyhow::Error`] to allow flexible error handling
/// and context propagation. Future versions may replace this with a concrete error type.
pub type Error = anyhow::Error;

/// A `Result` type alias using this crate's [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;