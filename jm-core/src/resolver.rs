use crate::dependency::Dependency;
use crate::errors::JmCoreError;
use crate::package::Package;
use async_trait::async_trait;

#[async_trait]
pub trait PackageResolver {
    async fn get<'a>(
        &self,
        requester: &str,
        dependency: &'a Dependency,
    ) -> Result<(Package, &'a Dependency), JmCoreError>;
}
