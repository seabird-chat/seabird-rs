use anyhow::Context;
use std::collections::HashMap;

use http::Uri;
use tonic::{
    metadata::{Ascii, MetadataValue},
    transport::{Channel, ClientTlsConfig},
};

use crate::error::Result;
use crate::proto;

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
        text: impl Into<String>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<()> {
        self.inner
            .perform_private_action(proto::PerformPrivateActionRequest {
                user_id: user_id.into(),
                text: text.into(),
                root_block: None,
                tags: tags.unwrap_or_default(),
            })
            .await?;
        Ok(())
    }

    pub async fn perform_action(
        &mut self,
        channel_id: impl Into<String>,
        text: impl Into<String>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<()> {
        self.inner
            .perform_action(proto::PerformActionRequest {
                channel_id: channel_id.into(),
                text: text.into(),
                root_block: None,
                tags: tags.unwrap_or_default(),
            })
            .await?;
        Ok(())
    }

    pub async fn send_message(
        &mut self,
        channel_id: impl Into<String>,
        text: impl Into<String>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<()> {
        self.inner
            .send_message(proto::SendMessageRequest {
                channel_id: channel_id.into(),
                text: text.into(),
                root_block: None,
                tags: tags.unwrap_or_default(),
            })
            .await?;
        Ok(())
    }

    pub async fn send_private_message(
        &mut self,
        user_id: impl Into<String>,
        text: impl Into<String>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<()> {
        self.inner
            .send_private_message(proto::SendPrivateMessageRequest {
                user_id: user_id.into(),
                text: text.into(),
                root_block: None,
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
