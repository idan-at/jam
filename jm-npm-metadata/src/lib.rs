use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NpmDistMetadata {
    pub shasum: String,
    pub tarball: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NpmVersionMetadata {
    pub dist: NpmDistMetadata,
    pub dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NpmPackageMetadata {
    #[serde(alias = "dist-tags")]
    pub dist_tags: Option<HashMap<String, String>>,
    pub versions: HashMap<String, NpmVersionMetadata>,
}
