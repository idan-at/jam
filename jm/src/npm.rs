use log::{debug, info};
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

pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new() -> Fetcher {
        Fetcher {
            client: Client::new(),
        }
    }

    pub async fn get_package_metadata(&self, package_name: &str) -> PackageMetadata {
        let url = format!("http://npm.dev.wixpress.com/{}", encode(&package_name));

        info!("Getting package metadata from {}", url);

        match self
            .client
            .get(&url)
            .header(header::ACCEPT, NPM_ABBREVIATED_METADATA_ACCEPT_HEADER_VALUE)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let mut metadata: PackageMetadata = response.json().await.expect(&format!(
                    "Unexpected package metadata response for: {}",
                    package_name
                ));
                metadata.package_name = package_name.to_string();

                metadata
            }
            _ => {
                // TODO: retry?
                // TODO: return result instead
                panic!(format!(
                    "Failed to fetch package metadata for {}",
                    package_name
                ))
            }
        }
    }
}
