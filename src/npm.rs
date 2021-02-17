use log::debug;
use reqwest::header;
use reqwest::Client;
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
    #[serde(skip_deserializing)]
    pub package_name: String,
    #[serde(alias = "dist-tags")]
    pub dist_tags: Option<HashMap<String, String>>,
    pub versions: HashMap<String, VersionMetadata>,
}

pub struct NpmFacade {
    client: Client,
}

impl NpmFacade {
    pub fn new() -> NpmFacade {
        NpmFacade {
            client: Client::new(),
        }
    }

    pub async fn get_package_metadata(&self, package_name: &str) -> PackageMetadata {
        let url = format!("http://npm.dev.wixpress.com/{}", encode(&package_name));

        // TODO: remove unwrap and handle network errors
        let response = self
            .client
            .get(&url)
            .header(header::ACCEPT, NPM_ABBREVIATED_METADATA_ACCEPT_HEADER_VALUE)
            .send()
            .await
            .unwrap();

        if response.status().is_success() {
            let mut metadata: PackageMetadata = response.json().await.expect(&format!(
                "Unexpected package metadata response for: {}",
                package_name
            ));
            metadata.package_name = package_name.to_string();

            metadata
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
}
