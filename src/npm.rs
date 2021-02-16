use log::debug;
use reqwest::blocking::Client;
use reqwest::header;
use serde::Deserialize;
use std::collections::HashMap;
use urlencoding::encode;

const NPM_ABBREVIATED_METADATA_ACCEPT_HEADER_VALUE: &'static str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

#[derive(Debug, Deserialize)]
pub struct DistMetadata {
    pub shasum: String,
    pub tarball: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionMetadata {
    pub dist: DistMetadata,
}

#[derive(Debug, Deserialize)]
pub struct PackageMetadata {
    #[serde(alias = "dist-tags")]
    pub dist_tags: Option<HashMap<String, String>>,
    pub versions: HashMap<String, VersionMetadata>,
}

pub fn get_package_metadata(package_name: &str) -> PackageMetadata {
    let url = format!("http://npm.dev.wixpress.com/{}", encode(&package_name));

    let client = Client::new();
    // TODO: remove unwrap and handle network errors
    let response = client
        .get(&url)
        .header(header::ACCEPT, NPM_ABBREVIATED_METADATA_ACCEPT_HEADER_VALUE)
        .send()
        .unwrap();

    if response.status().is_success() {
        response.json().expect(&format!(
            "Unexpected package metadata response for: {}",
            package_name
        ))
    } else {
        debug!(
            "Failed to get package metadata for {}. status: {}",
            package_name,
            response.status()
        );
        // TODO: retry?
        panic!(format!(
            "Failed to fetch package metadata for {}",
            package_name
        ))
    }
}
