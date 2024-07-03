mod utils;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use tabled::Tabled;
use utoipa::ToSchema;

use crate::{api::utils::deserialize_labels_eq_filter, output::tabled::Option};

#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema, Tabled)]
pub struct DataNamespace {
    pub name: String,
    #[tabled(display_with = "crate::output::tabled::display")]
    pub extraction_graphs: Vec<ExtractionGraph>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListNamespacesResponse {
    pub namespaces: Vec<DataNamespace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Tabled)]
pub struct GetNamespaceResponse {
    pub namespace: DataNamespace, /*  */
}

// TODO: shouldn't this be DataNamespace?
#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateNamespace {
    pub name: String,
    pub extraction_graphs: Vec<ExtractionGraph>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Tabled)]
pub struct ExtractionGraph {
    #[serde(default)]
    #[tabled(skip)]
    pub id: String,
    pub name: String,
    // TODO: should this be Option<String>?
    #[tabled(skip)]
    #[serde(default)]
    pub namespace: String,
    pub description: Option<String>,
    #[tabled(display_with = "crate::output::tabled::display")]
    pub extraction_policies: Vec<ExtractionPolicy>,
}

impl std::fmt::Display for ExtractionGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct ExtractionGraphResponse {
    #[tabled(display_with = "crate::output::tabled::display")]
    pub indexes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Tabled)]
pub struct ExtractionPolicy {
    #[serde(default)]
    pub id: String,
    pub extractor: String,
    pub name: String,
    // TODO: fix once serialization and stuff is fixed.
    #[tabled(skip)]
    #[serde(default, deserialize_with = "deserialize_labels_eq_filter")]
    pub filters_eq: std::option::Option<HashMap<String, serde_json::Value>>,
    pub input_params: Option<serde_json::Value>,
    pub content_source: Option<String>,
    #[serde(default)]
    pub graph_name: String,
}

impl std::fmt::Display for ExtractionPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Tabled)]
pub struct ExtractorDescription {
    pub name: String,
    #[tabled(display_with = "crate::output::tabled::display")]
    pub input_mime_types: Vec<String>,
    pub description: String,
    #[tabled(skip)]
    pub input_params: serde_json::Value,
    #[tabled(skip)]
    pub outputs: HashMap<String, ExtractorOutputSchema>,
}

#[derive(Debug, Serialize, Deserialize, Default, ToSchema)]
pub struct ListExtractorsResponse {
    pub extractors: Vec<ExtractorDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
pub enum ExtractorOutputSchema {
    #[serde(rename = "embedding")]
    Embedding(EmbeddingSchema),
    #[serde(rename = "metadata")]
    Metadata(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EmbeddingSchema {
    pub dim: usize,
    pub distance: IndexDistance,
}

impl std::fmt::Display for EmbeddingSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.dim, self.distance)
    }
}

#[derive(Display, EnumString, Debug, Serialize, Deserialize, Clone, Default, ToSchema)]
#[serde(rename = "distance")]
pub enum IndexDistance {
    #[serde(rename = "dot")]
    #[strum(serialize = "dot")]
    #[default]
    Dot,

    #[serde(rename = "cosine")]
    #[strum(serialize = "cosine")]
    Cosine,

    #[serde(rename = "euclidean")]
    #[strum(serialize = "euclidean")]
    Euclidean,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Tabled)]
pub struct Index {
    pub name: String,
    pub embedding_schema: EmbeddingSchema,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListIndexesResponse {
    pub indexes: Vec<Index>,
}

#[derive(Debug, Serialize, Deserialize, Default, ToSchema)]
pub struct ListContentResponse {
    pub content_list: Vec<ContentMetadata>,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetContentMetadataResponse {
    pub content_metadata: ContentMetadata,
}

#[derive(Debug, Serialize, Deserialize, Default, ToSchema, Clone, Tabled)]
pub struct ContentMetadata {
    pub id: String,
    #[tabled(skip)]
    pub parent_id: String,
    #[tabled(skip)]
    pub root_content_id: String,
    #[tabled(skip)]
    pub namespace: String,
    pub name: String,
    pub mime_type: String,
    // TODO: this feels like it should be in the table.
    #[tabled(skip)]
    pub labels: HashMap<String, serde_json::Value>,
    #[tabled(display_with = "crate::output::tabled::display")]
    pub extraction_graph_names: Vec<String>,
    #[tabled(skip)]
    pub storage_url: String,
    // TODO: convert into something that can be displayed.
    pub created_at: i64,
    pub source: String,
    pub size: u64,
    #[tabled(skip)]
    pub hash: String,
}
