// TODO: replace these with better, more concrete error types.
pub type Error = anyhow::Error;
pub type Result<T> = std::result::Result<T, Error>;
