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

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub url: String,
    pub token: String,
}

pub type InnerClient =
    tonic::codegen::InterceptedService<tonic::transport::Channel, AuthHeaderInterceptor>;

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

// Client represents the running bot.
#[cfg(feature = "seabird-client")]
#[derive(Debug)]
pub struct SeabirdClient {
    inner: SeabirdProtoClient<InnerClient>,
}

#[cfg(feature = "seabird-client")]
impl SeabirdClient {
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

    pub fn inner_ref(&self) -> &'_ SeabirdProtoClient<InnerClient> {
        &self.inner
    }

    pub fn inner_mut_ref(&mut self) -> &'_ mut SeabirdProtoClient<InnerClient> {
        &mut self.inner
    }
}

#[cfg(feature = "chat-ingest-client")]
#[derive(Debug)]
pub struct ChatIngestClient {
    inner: ChatIngestProtoClient<InnerClient>,
}

#[cfg(feature = "chat-ingest-client")]
impl ChatIngestClient {
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

    pub fn inner_ref(&self) -> &'_ ChatIngestProtoClient<InnerClient> {
        &self.inner
    }

    pub fn inner_mut_ref(&mut self) -> &'_ mut ChatIngestProtoClient<InnerClient> {
        &mut self.inner
    }
}
