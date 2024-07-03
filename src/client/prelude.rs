// Trait implementations for crate::api

use std::{any::type_name, collections::HashMap};

use eyre::{eyre, Result};

use crate::{api::*, client::traits::*};

impl From<ListNamespacesResponse> for Vec<DataNamespace> {
    fn from(resp: ListNamespacesResponse) -> Self {
        resp.namespaces
    }
}

impl List for Vec<DataNamespace> {
    type Item = DataNamespace;
    type Response = ListNamespacesResponse;
}

impl std::fmt::Display for DataNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<GetNamespaceResponse> for DataNamespace {
    fn from(resp: GetNamespaceResponse) -> Self {
        resp.namespace
    }
}

impl Namespaced for DataNamespace {
    fn is_namespaced() -> bool {
        false
    }

    fn segments(id: Option<&str>) -> Result<Vec<&str>> {
        match id {
            Some(id) => Ok(vec!["namespaces", id]),
            None => Ok(vec!["namespaces"]),
        }
    }
}

// TODO: write a derive macro for this stuff
impl Get for DataNamespace {
    type Response = GetNamespaceResponse;
}

impl Create for CreateNamespace {
    type Response = HashMap<String, String>;
}

impl Namespaced for CreateNamespace {
    fn segments(_: Option<&str>) -> Result<Vec<&str>> {
        Ok(vec!["namespaces"])
    }
}

impl From<ListExtractorsResponse> for Vec<ExtractorDescription> {
    fn from(resp: ListExtractorsResponse) -> Self {
        resp.extractors
    }
}

impl List for Vec<ExtractorDescription> {
    type Item = ExtractorDescription;
    type Response = ListExtractorsResponse;
}

impl Namespaced for ExtractorDescription {
    fn is_namespaced() -> bool {
        false
    }

    fn segments(_: Option<&str>) -> Result<Vec<&str>> {
        Ok(vec!["extractors"])
    }
}

impl Namespaced for ExtractionGraph {
    fn segments(id: Option<&str>) -> Result<Vec<&str>> {
        match id {
            // TODO: generate this boilerplate with a proper error
            Some(_) => Err(eyre!(
                "{} does not support getting by ID.",
                type_name::<Self>()
            )),
            None => Ok(vec!["extraction_graphs"]),
        }
    }
}

impl Create for ExtractionGraph {
    type Response = ExtractionGraphResponse;
}

impl From<ListIndexesResponse> for Vec<Index> {
    fn from(resp: ListIndexesResponse) -> Self {
        resp.indexes
    }
}

impl List for Vec<Index> {
    type Item = Index;
    type Response = ListIndexesResponse;
}

impl Namespaced for Index {
    fn segments(id: Option<&str>) -> Result<Vec<&str>> {
        match id {
            Some(_) => Err(eyre!(
                "{} does not support getting by ID.",
                type_name::<Self>()
            )),
            None => Ok(vec!["indexes"]),
        }
    }
}

impl From<ListContentResponse> for Vec<ContentMetadata> {
    fn from(resp: ListContentResponse) -> Self {
        resp.content_list
    }
}

impl List for Vec<ContentMetadata> {
    type Item = ContentMetadata;
    type Response = ListContentResponse;
}

impl Namespaced for ContentMetadata {
    fn segments(id: Option<&str>) -> Result<Vec<&str>> {
        match id {
            Some(id) => Ok(vec!["content", id]),
            None => Ok(vec!["content"]),
        }
    }
}

impl From<GetContentMetadataResponse> for ContentMetadata {
    fn from(resp: GetContentMetadataResponse) -> Self {
        resp.content_metadata
    }
}

impl Get for ContentMetadata {
    type Response = GetContentMetadataResponse;
}
