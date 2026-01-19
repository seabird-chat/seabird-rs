# seabird-rs

[![Crates.io](https://img.shields.io/crates/v/seabird.svg)](https://crates.io/crates/seabird)
[![docs.rs](https://img.shields.io/docsrs/seabird)](https://docs.rs/seabird)
[![License](https://img.shields.io/crates/l/seabird.svg)](https://github.com/seabird-chat/seabird-rs#license)

A Rust client library for the seabird-chat ecosystem.

## Example

```rust
use seabird::{ClientConfig, SeabirdClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig {
        url: "https://seabird.example.com".to_string(),
        token: "your-token-here".to_string(),
    };

    let mut client = SeabirdClient::new(config).await?;
    client.send_message("channel-id", "Hello, world!", None).await?;

    Ok(())
}
```
