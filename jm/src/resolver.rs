use crate::npm::Fetcher;
use crate::npm::PackageMetadata;
use async_trait::async_trait;
use dashmap::DashMap;
use dashmap::DashSet;
use jm_core::dependency::Dependency;
use jm_core::errors::JmError;
use jm_core::package::NpmPackage;
use jm_core::package::Package;
use jm_core::resolver::PackageResolver;
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

    async fn get_dependency(
        &self,
        requester: &str,
        dependency: &Dependency,
    ) -> Result<Package, JmError> {
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

        let version =
            self.helper
                .resolve_version(requester, &package_requested_version, &metadata)?;

        let version_metadata = metadata.versions.get(&version.to_string()).unwrap();

        Ok(Package::Package(NpmPackage::new(
            dependency.name.to_string(),
            version.to_string(),
            Some(version_metadata.dependencies.clone()),
            version_metadata.shasum.clone(),
            version_metadata.tarball.clone(),
        )))
    }

    async fn version_matches(
        &self,
        package: &Package,
        dependency: &Dependency,
    ) -> Result<bool, JmError> {
        let metadata = self
            .fetcher
            .get_package_metadata(&dependency.real_name)
            .await?;

        let package_requested_version = self
            .helper
            .extract_dependency_version_req(dependency, &metadata)?;

        Ok(self
            .helper
            .version_matches(&package_requested_version, &package.version()))
    }
}

#[async_trait]
impl PackageResolver for Resolver {
    async fn get<'a>(
        &self,
        requester: &str,
        dependency: &'a Dependency,
    ) -> Result<(Package, &'a Dependency), JmError> {
        let package_name = &dependency.real_name;

        match self.cache.get(package_name) {
            Some(packages_set) => {
                for reference in packages_set.iter() {
                    let package_ref = reference.deref();

                    if self.version_matches(package_ref, dependency).await? {
                        debug!("Got {} package from cache", package_name);
                        return Ok((package_ref.clone(), dependency));
                    }
                }

                let package = self.get_dependency(requester, dependency).await?;
                debug!("Got {} package from remote", package_name);

                packages_set.insert(package.clone());

                Ok((package, dependency))
            }
            None => {
                let package = self.get_dependency(requester, dependency).await?;
                debug!("Got {} package from remote", package_name);

                let set = DashSet::from_iter(vec![package.clone()].into_iter());
                self.cache.insert(package_name.to_string(), set);

                Ok((package, dependency))
            }
        }
    }
}

impl ResolverHelper {
    pub fn new() -> ResolverHelper {
        ResolverHelper {}
    }

    pub fn extract_dependency_version_req(
        &self,
        dependency: &Dependency,
        package_metadata: &PackageMetadata,
    ) -> Result<VersionReq, JmError> {
        match VersionReq::parse_compat(&dependency.version_or_dist_tag, Compat::Npm) {
            Ok(version) => Ok(version),
            Err(_) => {
                let version = package_metadata
                    .dist_tags
                    .get(&dependency.version_or_dist_tag)
                    .ok_or(JmError::new(format!(
                        "Failed to resolve dist tag {} of package {}",
                        &dependency.version_or_dist_tag, &dependency.real_name
                    )))?;

                if package_metadata.versions.contains_key(version) {
                    Ok(VersionReq::parse(version).unwrap())
                } else {
                    Err(JmError::new(format!(
                        "{}@{} points to version {}, which does not exist",
                        &dependency.real_name, &dependency.version_or_dist_tag, &version
                    )))
                }
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
    ) -> Result<Version, JmError> {
        let mut matching_versions: Vec<Version> = package_metadata
            .versions
            .keys()
            .filter(|version| self.version_matches(requested_version, version))
            .map(|version| Version::from_str(version).unwrap())
            .collect();

        if matching_versions.is_empty() {
            return Err(JmError::new(format!(
                "No matching versions for {}->{} (requested {})",
                parent, package_metadata.package_name, requested_version
            )));
        }

        matching_versions.sort();

        Ok(matching_versions.into_iter().last().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npm::VersionMetadata;
    use maplit::hashmap;
    use std::collections::HashMap;

    #[test]
    fn extract_dependency_version_req_dist_tag_ok() {
        let helper = ResolverHelper::new();
        let package_name = "dep1".to_string();
        let version = "1.0.0".to_string();
        let dist_tag = "beta".to_string();

        let dependency = Dependency {
            name: package_name.clone(),
            real_name: package_name.clone(),
            version_or_dist_tag: dist_tag.clone(),
        };

        let metadata = PackageMetadata {
            package_name: package_name.clone(),
            dist_tags: hashmap! {
                dist_tag.clone() => version.clone()
            },
            versions: hashmap! {
                version.clone() => VersionMetadata {
                    shasum: "a-shasum".to_string(),
                    tarball: "a-tarball".to_string(),
                    dependencies: HashMap::new(),
                }
            },
        };

        let result = helper.extract_dependency_version_req(&dependency, &metadata);

        assert_eq!(result, Ok(VersionReq::from_str(&version).unwrap()));
    }

    #[test]
    fn extract_dependency_version_req_dist_tag_not_found() {
        let helper = ResolverHelper::new();
        let package_name = "dep1".to_string();
        let version = "1.0.0".to_string();
        let dist_tag = "beta".to_string();

        let dependency = Dependency {
            name: package_name.clone(),
            real_name: package_name.clone(),
            version_or_dist_tag: dist_tag.clone(),
        };

        let metadata = PackageMetadata {
            package_name: package_name.clone(),
            dist_tags: hashmap! {
                "not-beta".to_string() => version.clone()
            },
            versions: hashmap! {
                version.clone() => VersionMetadata {
                    shasum: "a-shasum".to_string(),
                    tarball: "a-tarball".to_string(),
                    dependencies: HashMap::new(),
                }
            },
        };

        let result = helper.extract_dependency_version_req(&dependency, &metadata);

        assert_eq!(
            result,
            Err(JmError::new(format!(
                "Failed to resolve dist tag {} of package {}",
                dist_tag, package_name
            )))
        );
    }

    #[test]
    fn extract_dependency_version_req_dist_tag_no_matching_version() {
        let helper = ResolverHelper::new();
        let package_name = "dep1".to_string();
        let version = "1.0.0".to_string();
        let dist_tag = "beta".to_string();

        let dependency = Dependency {
            name: package_name.clone(),
            real_name: package_name.clone(),
            version_or_dist_tag: dist_tag.clone(),
        };

        let metadata = PackageMetadata {
            package_name: package_name.clone(),
            dist_tags: hashmap! {
                dist_tag.to_string() => version.clone()
            },
            versions: hashmap! {
                "2.0.0".to_string() => VersionMetadata {
                    shasum: "a-shasum".to_string(),
                    tarball: "a-tarball".to_string(),
                    dependencies: HashMap::new(),
                }
            },
        };

        let result = helper.extract_dependency_version_req(&dependency, &metadata);

        assert_eq!(
            result,
            Err(JmError::new(format!(
                "{}@{} points to version 1.0.0, which does not exist",
                package_name, dist_tag
            )))
        );
    }

    #[test]
    fn version_matches() {
        let helper = ResolverHelper::new();
        let version_req = VersionReq::parse("~1.0.0").unwrap();

        assert!(helper.version_matches(&version_req, "1.0.0"));
        assert!(!helper.version_matches(&version_req, "2.0.0"));
    }

    #[test]
    fn resolve_version_finds_the_best_match() {
        let helper = ResolverHelper::new();
        let version_req = VersionReq::parse("~1.0.0").unwrap();
        let metadata = PackageMetadata {
            package_name: "a-package".to_string(),
            dist_tags: HashMap::new(),
            versions: hashmap! {
                "1.0.0".to_string() => VersionMetadata {
                    shasum: "a-shasum".to_string(),
                    tarball: "a-tarball".to_string(),
                    dependencies: HashMap::new(),
                },
                "1.0.1".to_string() => VersionMetadata {
                    shasum: "a-shasum".to_string(),
                    tarball: "a-tarball".to_string(),
                    dependencies: HashMap::new(),
                },
                "2.0.0".to_string() => VersionMetadata {
                    shasum: "a-shasum".to_string(),
                    tarball: "a-tarball".to_string(),
                    dependencies: HashMap::new(),
                }
            },
        };

        let result = helper.resolve_version("never-mind", &version_req, &metadata);

        assert_eq!(result, Ok(Version::from_str("1.0.1").unwrap()));
    }

    #[test]
    fn resolve_version_error_when_no_matching_versions_exist() {
        let helper = ResolverHelper::new();
        let version_req = VersionReq::parse("~3.0.0").unwrap();
        let parent = "parent-package".to_string();
        let package_name = "package".to_string();
        let metadata = PackageMetadata {
            package_name: package_name.clone(),
            dist_tags: HashMap::new(),
            versions: hashmap! {
                "1.0.0".to_string() => VersionMetadata {
                    shasum: "a-shasum".to_string(),
                    tarball: "a-tarball".to_string(),
                    dependencies: HashMap::new(),
                }
            },
        };

        let result = helper.resolve_version(&parent, &version_req, &metadata);

        assert_eq!(
            result,
            Err(JmError::new(format!(
                "No matching versions for {}->{} (requested {})",
                parent, package_name, version_req
            )))
        );
    }
}
