use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum NpmBinMetadata {
    String(String),
    Object(HashMap<String, String>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NpmDistMetadata {
    pub shasum: String,
    pub tarball: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NpmVersionMetadata {
    pub bin: Option<NpmBinMetadata>,
    pub dist: NpmDistMetadata,
    pub dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NpmPackageMetadata {
    #[serde(alias = "dist-tags")]
    pub dist_tags: Option<HashMap<String, String>>,
    pub versions: HashMap<String, NpmVersionMetadata>,
}
