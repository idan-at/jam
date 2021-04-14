use async_trait::async_trait;
use dashmap::DashMap;
use dashmap::DashSet;
use jm_core::dependency::Dependency;
use jm_core::errors::JmCoreError;
use jm_core::npm::Fetcher;
use jm_core::package::NpmPackage;
use jm_core::package::Package;
use jm_core::resolver::PackageResolver;
use jm_core::resolver_helpers::{extract_dependency_version_req, resolve_version, version_matches};
use log::{debug, info};
use std::iter::FromIterator;
use std::ops::Deref;

pub struct Resolver<'a> {
    cache: DashMap<String, DashSet<Package>>,
    fetcher: Fetcher<'a>,
}

// TODO: Move to core, support searching packages in the workspace first
impl<'a> Resolver<'a> {
    pub fn new(fetcher: Fetcher<'a>) -> Resolver<'a> {
        Resolver {
            cache: DashMap::new(),
            fetcher,
        }
    }

    async fn get_dependency(
        &self,
        requester: &str,
        dependency: &Dependency,
    ) -> Result<Package, JmCoreError> {
        info!(
            "Fetching dependency {}@{}",
            dependency.real_name, dependency.version_or_dist_tag
        );

        let metadata = self
            .fetcher
            .get_package_metadata(&dependency.real_name)
            .await?;

        let package_requested_version = extract_dependency_version_req(dependency, &metadata)?;

        let version = resolve_version(requester, &package_requested_version, &metadata)?;

        let version_metadata = metadata.versions.get(&version.to_string()).unwrap();

        Ok(Package::NpmPackage(NpmPackage::new(
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
    ) -> Result<bool, JmCoreError> {
        let metadata = self
            .fetcher
            .get_package_metadata(&dependency.real_name)
            .await?;

        let package_requested_version = extract_dependency_version_req(dependency, &metadata)?;

        Ok(version_matches(
            &package_requested_version,
            &package.version(),
        ))
    }
}

#[async_trait]
impl<'a> PackageResolver for Resolver<'a> {
    async fn get<'b>(
        &self,
        requester: &str,
        dependency: &'b Dependency,
    ) -> Result<(Package, &'b Dependency), JmCoreError> {
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
