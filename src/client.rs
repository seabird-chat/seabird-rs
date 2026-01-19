use anyhow::Context;
use std::collections::HashMap;

use http::Uri;
use tonic::{
    metadata::{Ascii, MetadataValue},
    transport::{Channel, ClientTlsConfig},
};

use crate::error::Result;
use crate::proto;

/// MessageContent represents either plain text or structured blocks for messages.
#[derive(Debug)]
pub enum MessageContent {
    Text(String),
    Blocks(proto::Block),
}

impl MessageContent {
    /// Converts the message content into its internal representation.
    ///
    /// Returns a tuple of (text, optional block), where text-only messages
    /// return the text with None, and block messages return empty text with Some(block).
    fn into_inner(self) -> (String, Option<proto::Block>) {
        match self {
            MessageContent::Text(text) => (text, None),
            MessageContent::Blocks(block) => (String::from(""), Some(block)),
        }
    }
}

impl From<String> for MessageContent {
    fn from(text: String) -> Self {
        MessageContent::Text(text)
    }
}

impl From<&str> for MessageContent {
    fn from(text: &str) -> Self {
        MessageContent::Text(text.to_string())
    }
}

impl From<proto::Block> for MessageContent {
    fn from(block: proto::Block) -> Self {
        MessageContent::Blocks(block)
    }
}

#[cfg(feature = "seabird-client")]
use crate::proto::seabird::seabird_client::SeabirdClient as SeabirdProtoClient;

#[cfg(feature = "chat-ingest-client")]
use crate::proto::seabird::chat_ingest_client::ChatIngestClient as ChatIngestProtoClient;

/// Configuration for connecting to a seabird instance.
///
/// # Examples
///
/// ```rust
/// use seabird::ClientConfig;
///
/// let config = ClientConfig {
///     url: "https://seabird.example.com".to_string(),
///     token: "your-bot-token".to_string(),
/// };
/// ```
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// The URL of the seabird instance (e.g., "https://seabird.example.com" or "http://localhost:11235")
    pub url: String,
    /// The auth token for the bot
    pub token: String,
}

/// A convenience wrapper around the raw gRPC client type with authentication added.
///
/// This type alias represents a tonic channel with an authentication interceptor
/// that automatically adds authorization headers to all requests.
pub type InnerClient =
    tonic::codegen::InterceptedService<tonic::transport::Channel, AuthHeaderInterceptor>;

/// A tonic interceptor that adds authentication headers to gRPC requests.
///
/// Most users should not need to use this directly. This interceptor
/// automatically adds the "authorization" header with a Bearer token to every
/// outgoing request.
#[derive(Debug)]
pub struct AuthHeaderInterceptor {
    auth_header: MetadataValue<Ascii>,
}

impl tonic::service::Interceptor for AuthHeaderInterceptor {
    fn call(
        &mut self,
        mut req: tonic::Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        req.metadata_mut()
            .insert("authorization", self.auth_header.clone());
        Ok(req)
    }
}

/// Client for interacting with a seabird instance, generally as a bot.
///
/// This client provides methods for sending messages and performing actions in
/// channels and private conversations. It requires the `seabird-client` feature
/// to be enabled.
///
/// # Examples
///
/// ```rust,no_run
/// use seabird::{ClientConfig, SeabirdClient};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ClientConfig {
///         url: "https://seabird.example.com".to_string(),
///         token: "your-token-here".to_string(),
///     };
///
///     let mut client = SeabirdClient::new(config).await?;
///     client.send_message("channel-id", "Hello!", None).await?;
///
///     Ok(())
/// }
/// ```
#[cfg(feature = "seabird-client")]
#[derive(Debug)]
pub struct SeabirdClient {
    inner: SeabirdProtoClient<InnerClient>,
}

#[cfg(feature = "seabird-client")]
impl SeabirdClient {
    /// Creates a new SeabirdClient from the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The client configuration containing the server URL and authentication token
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL cannot be parsed
    /// - TLS configuration fails (for https URLs)
    /// - Connection to the server fails
    /// - The authentication token is an invalid format
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use seabird::{ClientConfig, SeabirdClient};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ClientConfig {
    ///     url: "https://seabird.example.com".to_string(),
    ///     token: "your-token".to_string(),
    /// };
    ///
    /// let client = SeabirdClient::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let uri: Uri = config.url.parse().context("failed to parse seabird URL")?;
        let mut channel_builder = Channel::builder(uri.clone());

        match uri.scheme_str() {
            None | Some("https") => {
                channel_builder = channel_builder.tls_config(ClientTlsConfig::new())?;
            }
            _ => {}
        }

        let channel = channel_builder
            .connect()
            .await
            .context("Failed to connect to seabird")?;

        let auth_header: MetadataValue<Ascii> = format!("Bearer {}", config.token).parse()?;

        let seabird_client =
            SeabirdProtoClient::with_interceptor(channel, AuthHeaderInterceptor { auth_header });

        Ok(Self {
            inner: seabird_client,
        })
    }

    /// Performs an action in a private conversation with a user.
    ///
    /// Actions are typically displayed differently than regular messages (e.g., "/me waves").
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user to send the action to
    /// * `content` - The message content (text or formatted blocks)
    /// * `tags` - Optional metadata tags for the message
    ///
    /// # Errors
    ///
    /// Returns an error if the gRPC request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use seabird::{ClientConfig, SeabirdClient};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SeabirdClient::new(ClientConfig {
    /// #     url: "https://example.com".to_string(),
    /// #     token: "token".to_string(),
    /// # }).await?;
    /// client.perform_private_action("user-id", "waves", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn perform_private_action(
        &mut self,
        user_id: impl Into<String>,
        content: impl Into<MessageContent>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let (text, root_block) = content.into().into_inner();

        self.inner
            .perform_private_action(proto::PerformPrivateActionRequest {
                user_id: user_id.into(),
                text,
                root_block,
                tags: tags.unwrap_or_default(),
            })
            .await?;
        Ok(())
    }

    /// Performs an action in a channel.
    ///
    /// Actions are typically displayed differently than regular messages (e.g., "/me waves").
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The ID of the channel to send the action to
    /// * `content` - The message content (text or formatted blocks)
    /// * `tags` - Optional metadata tags for the message
    ///
    /// # Errors
    ///
    /// Returns an error if the gRPC request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use seabird::{ClientConfig, SeabirdClient};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SeabirdClient::new(ClientConfig {
    /// #     url: "https://example.com".to_string(),
    /// #     token: "token".to_string(),
    /// # }).await?;
    /// client.perform_action("channel-id", "dances", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn perform_action(
        &mut self,
        channel_id: impl Into<String>,
        content: impl Into<MessageContent>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let (text, root_block) = content.into().into_inner();

        self.inner
            .perform_action(proto::PerformActionRequest {
                channel_id: channel_id.into(),
                text,
                root_block,
                tags: tags.unwrap_or_default(),
            })
            .await?;
        Ok(())
    }

    /// Sends a message to a channel.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The ID of the channel to send the message to
    /// * `content` - The message content (text or formatted blocks)
    /// * `tags` - Optional metadata tags for the message
    ///
    /// # Errors
    ///
    /// Returns an error if the gRPC request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use seabird::{ClientConfig, SeabirdClient, Block};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SeabirdClient::new(ClientConfig {
    /// #     url: "https://example.com".to_string(),
    /// #     token: "token".to_string(),
    /// # }).await?;
    /// // Send plain text
    /// client.send_message("channel-id", "Hello!", None).await?;
    ///
    /// // Send formatted blocks
    /// let formatted = Block::new().text("Hello ").bold("world");
    /// client.send_message("channel-id", formatted, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message(
        &mut self,
        channel_id: impl Into<String>,
        content: impl Into<MessageContent>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let (text, root_block) = content.into().into_inner();

        self.inner
            .send_message(proto::SendMessageRequest {
                channel_id: channel_id.into(),
                text,
                root_block,
                tags: tags.unwrap_or_default(),
            })
            .await?;
        Ok(())
    }

    /// Sends a private message to a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user to send the message to
    /// * `content` - The message content (text or formatted blocks)
    /// * `tags` - Optional metadata tags for the message
    ///
    /// # Errors
    ///
    /// Returns an error if the gRPC request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use seabird::{ClientConfig, SeabirdClient};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = SeabirdClient::new(ClientConfig {
    /// #     url: "https://example.com".to_string(),
    /// #     token: "token".to_string(),
    /// # }).await?;
    /// client.send_private_message("user-id", "Hello!", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_private_message(
        &mut self,
        user_id: impl Into<String>,
        content: impl Into<MessageContent>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let (text, root_block) = content.into().into_inner();

        self.inner
            .send_private_message(proto::SendPrivateMessageRequest {
                user_id: user_id.into(),
                text,
                root_block,
                tags: tags.unwrap_or_default(),
            })
            .await?;
        Ok(())
    }

    /// Returns a reference to the inner gRPC client.
    ///
    /// This provides access to the underlying tonic-generated client for
    /// advanced use cases not covered by the high-level API.
    pub fn inner_ref(&self) -> &'_ SeabirdProtoClient<InnerClient> {
        &self.inner
    }

    /// Returns a mutable reference to the inner gRPC client.
    ///
    /// This provides mutable access to the underlying tonic-generated client
    /// for advanced use cases not covered by the high-level API.
    pub fn inner_mut_ref(&mut self) -> &'_ mut SeabirdProtoClient<InnerClient> {
        &mut self.inner
    }
}

/// Client for ingesting chat data into seabird.
///
/// This client is used to send chat data from external sources into the seabird
/// chat ingest service. It requires the `chat-ingest-client` feature to be enabled.
///
/// # Examples
///
/// ```rust,no_run
/// use seabird::{ClientConfig, ChatIngestClient};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ClientConfig {
///         url: "https://seabird.example.com".to_string(),
///         token: "your-token-here".to_string(),
///     };
///
///     let client = ChatIngestClient::new(config).await?;
///     // Use the inner client to access chat ingest methods
///
///     Ok(())
/// }
/// ```
#[cfg(feature = "chat-ingest-client")]
#[derive(Debug)]
pub struct ChatIngestClient {
    inner: ChatIngestProtoClient<InnerClient>,
}

#[cfg(feature = "chat-ingest-client")]
impl ChatIngestClient {
    /// Creates a new ChatIngestClient from the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The client configuration containing the server URL and authentication token
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL cannot be parsed
    /// - TLS configuration fails (for https URLs)
    /// - Connection to the server fails
    /// - The authentication token is invalid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use seabird::{ClientConfig, ChatIngestClient};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ClientConfig {
    ///     url: "https://seabird.example.com".to_string(),
    ///     token: "your-token".to_string(),
    /// };
    ///
    /// let client = ChatIngestClient::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let uri: Uri = config.url.parse().context("failed to parse seabird URL")?;
        let mut channel_builder = Channel::builder(uri.clone());

        match uri.scheme_str() {
            None | Some("https") => {
                channel_builder = channel_builder.tls_config(ClientTlsConfig::new())?;
            }
            _ => {}
        }

        let channel = channel_builder
            .connect()
            .await
            .context("Failed to connect to seabird")?;

        let auth_header: MetadataValue<Ascii> = format!("Bearer {}", config.token).parse()?;

        let chat_ingest_client =
            ChatIngestProtoClient::with_interceptor(channel, AuthHeaderInterceptor { auth_header });

        Ok(Self {
            inner: chat_ingest_client,
        })
    }

    /// Returns a reference to the inner gRPC client.
    ///
    /// This provides access to the underlying tonic-generated client for
    /// accessing chat ingest protocol methods.
    pub fn inner_ref(&self) -> &'_ ChatIngestProtoClient<InnerClient> {
        &self.inner
    }

    /// Returns a mutable reference to the inner gRPC client.
    ///
    /// This provides mutable access to the underlying tonic-generated client
    /// for accessing chat ingest protocol methods.
    pub fn inner_mut_ref(&mut self) -> &'_ mut ChatIngestProtoClient<InnerClient> {
        &mut self.inner
    }
}
