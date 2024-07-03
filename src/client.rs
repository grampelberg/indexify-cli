mod prelude;
pub mod traits;
pub mod types;

pub const DEFAULT_SERVICE_URL: &str = "http://localhost:8900";

use std::{any::type_name, fmt::Debug, vec};

use clap::{
    builder::{TypedValueParser, ValueParserFactory},
    error::ErrorKind,
};
use color_eyre::{Section, SectionExt};
use eyre::{eyre, Ok, Result};
use futures::TryStreamExt;
use serde::{de::DeserializeOwned, Serialize};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::instrument;

use crate::client::traits::*;

#[derive(Debug, Clone)]
pub struct Client {
    service_url: reqwest::Url,
    namespace: Option<String>,
}

// TODO: fold namespaced client into this
impl Client {
    pub fn new(service_url: &str) -> Result<Self> {
        let url = reqwest::Url::parse(service_url)?;

        Ok(Self {
            service_url: url,
            namespace: None,
        })
    }

    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    fn url<T>(&self, id: Option<&str>) -> Result<reqwest::Url>
    where
        T: Namespaced,
    {
        let mut url = self.service_url.clone();

        {
            let mut url_segments = url
                .path_segments_mut()
                .map_err(|_| eyre!("cannot be base"))?;

            if T::is_namespaced() {
                match &self.namespace {
                    Some(namespace) => url_segments.extend(vec!["namespaces", namespace]),
                    None => {
                        return Err(eyre!(
                            "namespace is required for {}. Call with_namespace first.",
                            type_name::<T>()
                        ))
                    }
                };
            }

            url_segments.extend(T::segments(id)?);
        }

        Ok(url)
    }

    #[instrument(level = "trace")]
    async fn _get(&self, path: reqwest::Url) -> Result<String> {
        reqwest::get(path.as_str()).await?.text_or_error().await
    }

    #[instrument(level = "trace", skip(body))]
    async fn _post(&self, path: reqwest::Url, body: impl Serialize) -> Result<String> {
        reqwest::Client::new()
            .post(path.as_str())
            .json(&body)
            .send()
            .await?
            .text_or_error()
            .await
    }

    fn deserialize<T>(&self, resp: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let content: Result<T, _> =
            serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(resp));

        content.with_section(move || resp.to_string().header("Body:"))
    }

    pub async fn list<T>(&self) -> Result<T>
    where
        T: List,
        T::Response: Into<T>,
        T::Item: Namespaced,
    {
        let resp = self._get(self.url::<T::Item>(None)?).await?;

        Ok(self.deserialize::<T::Response>(&resp)?.into())
    }

    pub async fn get<T>(&self, id: &str) -> Result<T>
    where
        T: Get,
        T::Response: Into<T>,
    {
        let resp = self._get(self.url::<T>(Some(id))?).await?;

        Ok(self.deserialize::<T::Response>(&resp)?.into())
    }

    pub async fn get_stream<T>(
        &self,
        id: Option<&str>,
    ) -> Result<(Option<u64>, impl tokio::io::AsyncRead)>
    where
        T: Namespaced,
    {
        let resp = reqwest::get(self.url::<T>(id)?).await?.error_for_status()?;

        Ok((
            resp.content_length(),
            resp.bytes_stream()
                .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
                .into_async_read()
                .compat(),
        ))
    }

    pub async fn create<T>(&self, obj: &T) -> Result<T::Response>
    where
        T: Create + Serialize,
    {
        let resp = self._post(self.url::<T>(None)?, obj).await?;

        Ok(self.deserialize::<T::Response>(&resp)?)
    }

    pub async fn delete<T>(&self, body: T) -> Result<T::Response>
    where
        T: Delete,
        T::Response: DeserializeOwned,
    {
        let resp = reqwest::Client::new()
            .delete(self.url::<T>(None)?)
            .json(&body)
            .send()
            .await?
            .text_or_error()
            .await?;

        self.deserialize(&resp)
    }

    // TODO: figure out how to do progress bars from streams
    pub async fn upload<T>(&self, content: T) -> Result<()>
    where
        T: Upload + Namespaced,
    {
        reqwest::Client::new()
            .post(self.url::<T>(None)?)
            .multipart(content.form().await?)
            .query(&content.query())
            .send()
            .await?
            .text_or_error()
            .await?;

        Ok(())
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            service_url: reqwest::Url::parse(DEFAULT_SERVICE_URL).unwrap(),
            namespace: Some("default".to_string()),
        }
    }
}

trait WithBody {
    async fn text_or_error(self) -> Result<String>;
}

impl WithBody for reqwest::Response {
    async fn text_or_error(self) -> Result<String> {
        let status = self.status();
        let url = self.url().clone();
        let out = self.text().await?;

        if !status.is_success() {
            return Err(eyre!("{} for {}", status, url)).with_section(move || out.header("Body:"));
        }

        Ok(out)
    }
}

impl TypedValueParser for Client {
    type Value = Self;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        Self::new(value.to_str().unwrap()).map_err(|e| {
            cmd.clone().error(
                ErrorKind::InvalidValue,
                if let Some(arg) = arg {
                    format!(
                        "Invalid value for {}: {:?} is not a valid URL - {}",
                        arg, value, e
                    )
                } else {
                    format!("{:?} is not a valid URL - {}", value, e)
                },
            )
        })
    }
}

impl ValueParserFactory for Client {
    type Parser = Self;

    fn value_parser() -> Self {
        Self::default()
    }
}
