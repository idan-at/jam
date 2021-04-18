use crate::errors::JamCoreError;
use again::RetryPolicy;
use jam_cache::{Cache, CacheFactory};
use jam_common::sanitize_package_name;
use jam_npm_metadata::NpmPackageMetadata;
use log::debug;
use reqwest::header;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use urlencoding::encode;

const NPM_ABBREVIATED_METADATA_ACCEPT_HEADER_VALUE: &'static str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

const FETCH_METADATA_EXPONENTIAL_BACK_OFF_MILLIS: u64 = 100;
const FETCH_METADATA_MAX_RETRIES: usize = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub shasum: String,
    pub tarball: String,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub package_name: String,
    pub dist_tags: HashMap<String, String>,
    pub versions: HashMap<String, VersionMetadata>,
}

pub struct Fetcher<'a> {
    cache: Cache,
    registry: &'a str,
    client: Client,
}

impl<'a> Fetcher<'a> {
    pub fn new(
        cache_factory: &CacheFactory,
        registry: &'a str,
    ) -> Result<Fetcher<'a>, JamCoreError> {
        let cache = cache_factory.create_cache("metadata")?;

        Ok(Fetcher {
            cache,
            registry,
            client: Client::new(),
        })
    }

    pub async fn get_package_metadata(
        &self,
        package_name: &str,
    ) -> Result<PackageMetadata, JamCoreError> {
        match self.cache.get(package_name) {
            Some(file_path) => {
                debug!("Got metadata for {} from cache", package_name);

                let file = File::open(file_path)?;
                let reader = BufReader::new(file);

                match serde_json::from_reader(reader) {
                    Ok(package_metadata) => Ok(package_metadata),
                    Err(_) => Err(JamCoreError::new(String::from(
                        "Failed to read package metadata from cache",
                    ))),
                }
            }
            None => {
                let metadata = self.get_package_metadata_from_npm(package_name).await?;

                self.cache.set(
                    &sanitize_package_name(package_name),
                    serde_json::to_string(&metadata).unwrap().as_bytes(),
                )?;

                Ok(metadata)
            }
        }
    }

    async fn get_package_metadata_from_npm(
        &self,
        package_name: &str,
    ) -> Result<PackageMetadata, String> {
        let now = Instant::now();
        let url = format!("{}/{}", self.registry, encode(&package_name));

        debug!("Getting {} metadata", package_name);
        let retry_policy = RetryPolicy::exponential(Duration::from_millis(
            FETCH_METADATA_EXPONENTIAL_BACK_OFF_MILLIS,
        ))
        .with_max_retries(1)
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
                debug!(
                    "Got {} package metadata in {} milliseconds",
                    package_name,
                    now.elapsed().as_millis()
                );

                let npm_metadata: NpmPackageMetadata = response.json().await.expect(&format!(
                    "{}: Unexpected package metadata response",
                    package_name
                ));

                let metadata = PackageMetadata {
                    package_name: package_name.to_string(),
                    dist_tags: npm_metadata.dist_tags.unwrap_or(HashMap::new()),
                    versions: npm_metadata
                        .versions
                        .iter()
                        .map(|(version, npm_version_metadata)| {
                            (
                                version.clone(),
                                VersionMetadata {
                                    shasum: npm_version_metadata.dist.shasum.clone(),
                                    tarball: npm_version_metadata.dist.tarball.clone(),
                                    dependencies: npm_version_metadata
                                        .dependencies
                                        .clone()
                                        .unwrap_or(HashMap::new()),
                                },
                            )
                        })
                        .collect(),
                };

                Ok(metadata)
            }
            _ => Err(format!(
                "{}: Failed to fetch package metadata with {} retries",
                package_name, FETCH_METADATA_MAX_RETRIES
            )),
        }
    }
}
