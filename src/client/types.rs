// New types that should probably be part of crate::api

use std::{collections::HashMap, fmt::Debug, path::Path, vec};

use eyre::{eyre, Ok, Result};
use reqwest::{multipart, Body};
use serde::Serialize;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::client::traits::*;

#[derive(Debug, Clone, Serialize)]
pub struct ContentIds {
    pub content_ids: Vec<String>,
}

impl ContentIds {
    pub fn new(content_ids: Vec<String>) -> Self {
        Self { content_ids }
    }
}

impl Namespaced for ContentIds {
    fn segments(_: Option<&str>) -> Result<Vec<&str>> {
        Ok(vec!["content"])
    }
}

impl Delete for ContentIds {
    type Response = HashMap<String, String>;
}

pub struct ContentUpload<'a> {
    path: &'a Path,
    graph_names: Vec<String>,
}

impl<'a> ContentUpload<'a> {
    pub fn new(path: &'a Path) -> ContentUpload<'a> {
        Self {
            path,
            graph_names: vec![],
        }
    }

    pub fn with_graph_names(mut self, graph_names: &[String]) -> Self {
        self.graph_names = graph_names.to_vec();
        self
    }
}

impl Namespaced for ContentUpload<'_> {
    fn segments(_: Option<&str>) -> Result<Vec<&str>> {
        Ok(vec!["upload_file"])
    }
}

impl Upload for ContentUpload<'_> {
    async fn form(&self) -> Result<multipart::Form> {
        let fname = self
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let mime = mime_guess::from_path(self.path).first_or_octet_stream();
        let fobj = File::open(self.path).await?;

        let stream = FramedRead::new(fobj, BytesCodec::new());
        let body = Body::wrap_stream(stream);
        let part = multipart::Part::stream(body)
            .file_name(fname)
            .mime_str(mime.as_ref())
            .unwrap();
        let form = multipart::Form::new().part("file", part);

        Ok(form)
    }

    fn query(&self) -> impl Serialize {
        vec![("extraction_graph_names", self.graph_names.join(","))]
    }
}

pub struct Download;

impl Namespaced for Download {
    fn segments(id: Option<&str>) -> Result<Vec<&str>> {
        match id {
            Some(id) => Ok(vec!["content", id, "download"]),
            None => Err(eyre!("Cannot download without an ID.")),
        }
    }
}
