use again::RetryPolicy;
use log::info;
use reqwest::header;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use urlencoding::encode;

const NPM_ABBREVIATED_METADATA_ACCEPT_HEADER_VALUE: &'static str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

const FETCH_METADATA_EXPONENTIAL_BACK_OFF_MILLIS: u64 = 100;
const FETCH_METADATA_MAX_RETRIES: usize = 3;

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
    registry: String,
    client: Client,
}

impl Fetcher {
    pub fn new(registry: String) -> Fetcher {
        Fetcher {
            registry,
            client: Client::new(),
        }
    }

    pub async fn get_package_metadata(&self, package_name: &str) -> Result<PackageMetadata, String> {
        let url = format!("{}/{}", self.registry, encode(&package_name));

        info!("Getting package metadata from {}", url);

        let retry_policy = RetryPolicy::exponential(Duration::from_millis(
            FETCH_METADATA_EXPONENTIAL_BACK_OFF_MILLIS,
        ))
        .with_max_retries(FETCH_METADATA_MAX_RETRIES)
        .with_jitter(true);

        match retry_policy
            .retry(|| {
                self.client
                    .get(&url)
                    .header(header::ACCEPT, NPM_ABBREVIATED_METADATA_ACCEPT_HEADER_VALUE)
                    .send()
            })
            .await
        {
            Ok(response) if response.status().is_success() => {
                let mut metadata: PackageMetadata = response.json().await.expect(&format!(
                    "Unexpected package metadata response for: {}",
                    package_name
                ));
                metadata.package_name = package_name.to_string();

                Ok(metadata)
            }
            _ => {
                Err(format!(
                    "Failed to fetch package metadata for {}",
                    package_name
                ))
            }
        }
    }
}
