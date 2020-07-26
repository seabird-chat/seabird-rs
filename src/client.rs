use anyhow::Context;

use http::Uri;
use tonic::{
    metadata::{Ascii, MetadataValue},
    transport::{Channel, ClientTlsConfig},
};

use crate::error::Result;
use crate::proto;
use crate::proto::seabird::seabird_client::SeabirdClient;

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub url: String,
    pub token: String,
}

// Client represents the running bot.
#[derive(Debug)]
pub struct Client {
    config: ClientConfig,
    inner: SeabirdClient<tonic::transport::Channel>,
}

impl Client {
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
            SeabirdClient::with_interceptor(channel, move |mut req: tonic::Request<()>| {
                req.metadata_mut()
                    .insert("authorization", auth_header.clone());
                Ok(req)
            });

        Ok(Client {
            config,
            inner: seabird_client,
        })
    }

    pub async fn perform_private_action(
        &mut self,
        user_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<()> {
        self.inner
            .perform_private_action(proto::PerformPrivateActionRequest {
                user_id: user_id.into(),
                text: text.into(),
            })
            .await?;
        Ok(())
    }

    pub async fn perform_action(
        &mut self,
        channel_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<()> {
        self.inner
            .perform_action(proto::PerformActionRequest {
                channel_id: channel_id.into(),
                text: text.into(),
            })
            .await?;
        Ok(())
    }

    pub async fn send_message(
        &mut self,
        channel_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<()> {
        self.inner
            .send_message(proto::SendMessageRequest {
                channel_id: channel_id.into(),
                text: text.into(),
            })
            .await?;
        Ok(())
    }

    pub async fn send_private_message(
        &mut self,
        user_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<()> {
        self.inner
            .send_private_message(proto::SendPrivateMessageRequest {
                user_id: user_id.into(),
                text: text.into(),
            })
            .await?;
        Ok(())
    }

    pub fn inner_ref(&self) -> &'_ SeabirdClient<tonic::transport::Channel> {
        &self.inner
    }

    pub fn inner_mut_ref(&mut self) -> &'_ mut SeabirdClient<tonic::transport::Channel> {
        &mut self.inner
    }
}
