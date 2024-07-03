use eyre::Result;
use reqwest::multipart;
use serde::{de::DeserializeOwned, Serialize};

pub trait Namespaced {
    fn is_namespaced() -> bool {
        true
    }

    fn segments(id: Option<&str>) -> Result<Vec<&str>>;
}
pub trait Create: Namespaced {
    type Response: DeserializeOwned;
}

pub trait Delete: Namespaced + Serialize {
    type Response: DeserializeOwned;
}

pub trait Get: Namespaced {
    type Response: DeserializeOwned;
}

pub trait List {
    type Item: Namespaced;
    type Response: DeserializeOwned;
}

pub trait Upload: Namespaced {
    async fn form(&self) -> Result<multipart::Form>;
    fn query(&self) -> impl Serialize;
}
