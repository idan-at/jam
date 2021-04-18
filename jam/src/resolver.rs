use async_trait::async_trait;
use dashmap::DashMap;
use dashmap::DashSet;
use jam_core::dependency::Dependency;
use jam_core::errors::JamCoreError;
use jam_core::npm::Fetcher;
use jam_core::package::NpmPackage;
use jam_core::package::Package;
use jam_core::package::WorkspacePackage;
use jam_core::resolver::PackageResolver;
use jam_core::resolver_helpers::{
    extract_dependency_version_req, resolve_version, version_matches,
};
use log::{debug, info};
use std::iter::FromIterator;
use std::ops::Deref;

pub struct Resolver<'a> {
    cache: DashMap<String, DashSet<Package>>,
    fetcher: Fetcher<'a>,
    workspace_packages: &'a Vec<WorkspacePackage>,
}

// TODO: Move to core
impl<'a> Resolver<'a> {
    pub fn new(
        fetcher: Fetcher<'a>,
        workspace_packages: &'a Vec<WorkspacePackage>,
    ) -> Resolver<'a> {
        Resolver {
            cache: DashMap::new(),
            fetcher,
            workspace_packages,
        }
    }

    async fn get_dependency(
        &self,
        requester: &str,
        dependency: &Dependency,
    ) -> Result<Package, JamCoreError> {
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
    ) -> Result<bool, JamCoreError> {
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
    ) -> Result<(Package, &'b Dependency), JamCoreError> {
        let package_name = &dependency.real_name;

        // TODO: skip link if its a different major version?
        if let Some(workspace_package) = self
            .workspace_packages
            .iter()
            .find(|workspace_package| &workspace_package.name == package_name)
        {
            let package = Package::WorkspacePackage(workspace_package.clone());
            return Ok((package, dependency));
        }

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
