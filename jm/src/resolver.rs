use crate::dependency::Dependency;
use crate::npm::Fetcher;
use crate::npm::PackageMetadata;
use crate::package::Package;
use dashmap::DashMap;
use dashmap::DashSet;
use log::{debug, info};
use semver::{Compat, Version, VersionReq};
use std::iter::FromIterator;
use std::ops::Deref;
use std::str::FromStr;

pub struct Resolver {
    cache: DashMap<String, DashSet<Package>>,
    fetcher: Fetcher,
    helper: ResolverHelper,
}

struct ResolverHelper {}

impl Resolver {
    pub fn new(fetcher: Fetcher) -> Resolver {
        Resolver {
            cache: DashMap::new(),
            fetcher,
            helper: ResolverHelper::new(),
        }
    }

    pub async fn get(&self, requester: &str, dependency: &Dependency) -> Result<Package, String> {
        let package_name = &dependency.real_name;

        match self.cache.get(package_name) {
            Some(packages_set) => {
                debug!("Got {} package from cache", package_name);

                for reference in packages_set.iter() {
                    let package_ref = reference.deref();

                    if self.version_matches(package_ref, dependency).await? {
                        return Ok(package_ref.clone());
                    }
                }

                let package = self.get_dependency(requester, dependency).await?;
                debug!("Got {} package from remote", package_name);

                packages_set.insert(package.clone());

                Ok(package)
            }
            None => {
                let package = self.get_dependency(requester, dependency).await?;
                debug!("Got {} package from remote", package_name);

                let set = DashSet::from_iter(vec![package.clone()].into_iter());
                self.cache.insert(package_name.to_string(), set);

                Ok(package)
            }
        }
    }

    async fn get_dependency(
        &self,
        requester: &str,
        dependency: &Dependency,
    ) -> Result<Package, String> {
        info!(
            "Fetching dependency {}@{}",
            dependency.real_name, dependency.version_or_dist_tag
        );

        let metadata = self
            .fetcher
            .get_package_metadata(&dependency.real_name)
            .await?;

        let package_requested_version = self
            .helper
            .extract_dependency_version_req(dependency, &metadata)?;

        let version = self.helper.resolve_version(requester, &package_requested_version, &metadata)?;

        let version_metadata = metadata.versions.get(&version.to_string()).unwrap();

        Ok(Package::new(
            dependency.name.to_string(),
            version.to_string(),
            Some(version_metadata.dependencies.clone()),
            None,
        ))
    }

    async fn version_matches(
        &self,
        package: &Package,
        dependency: &Dependency,
    ) -> Result<bool, String> {
        let metadata = self
            .fetcher
            .get_package_metadata(&dependency.real_name)
            .await?;

        let package_requested_version = self
            .helper
            .extract_dependency_version_req(dependency, &metadata)?;

        Ok(self
            .helper
            .version_matches(&package_requested_version, &package.version))
    }
}

// TODO: test all helpers.
impl ResolverHelper {
    pub fn new() -> ResolverHelper {
        ResolverHelper {}
    }

    pub fn extract_dependency_version_req(
        &self,
        dependency: &Dependency,
        package_metadata: &PackageMetadata,
    ) -> Result<VersionReq, String> {
        match VersionReq::parse_compat(&dependency.version_or_dist_tag, Compat::Npm) {
            Ok(version) => Ok(version),
            Err(_) => {
                let dist_tag = package_metadata
                    .dist_tags
                    .get(&dependency.version_or_dist_tag)
                    .ok_or(format!(
                        "Failed to resolve dist tag {} of package {}",
                        &dependency.version_or_dist_tag, &dependency.real_name
                    ))?;

                Ok(VersionReq::parse(dist_tag).unwrap())
            }
        }
    }

    pub fn version_matches(&self, requested_version: &VersionReq, version: &str) -> bool {
        requested_version.matches(&Version::from_str(version).unwrap())
    }

    pub fn resolve_version(
        &self,
        parent: &str,
        requested_version: &VersionReq,
        package_metadata: &PackageMetadata,
    ) -> Result<Version, String> {
        let mut matching_versions: Vec<Version> = package_metadata
            .versions
            .keys()
            .filter(|version| self.version_matches(requested_version, version))
            .map(|version| Version::from_str(version).unwrap())
            .collect();

        if matching_versions.is_empty() {
            return Err(format!(
                "No matching versions for {}->{} (requested {})",
                parent, package_metadata.package_name, requested_version
            ));
        }

        matching_versions.sort();

        Ok(matching_versions.into_iter().last().unwrap())
    }
}
