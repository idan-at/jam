use crate::npm::PackageMetadata;
use crate::Dependency;
use crate::JmCoreError;
use semver::{Compat, Version, VersionReq};
use std::str::FromStr;

pub fn extract_dependency_version_req(
    dependency: &Dependency,
    package_metadata: &PackageMetadata,
) -> Result<VersionReq, JmCoreError> {
    match VersionReq::parse_compat(&dependency.version_or_dist_tag, Compat::Npm) {
        Ok(version) => Ok(version),
        Err(_) => {
            let version = package_metadata
                .dist_tags
                .get(&dependency.version_or_dist_tag)
                .ok_or(JmCoreError::new(format!(
                    "Failed to resolve dist tag {} of package {}",
                    &dependency.version_or_dist_tag, &dependency.real_name
                )))?;

            if package_metadata.versions.contains_key(version) {
                Ok(VersionReq::parse(version).unwrap())
            } else {
                Err(JmCoreError::new(format!(
                    "{}@{} points to version {}, which does not exist",
                    &dependency.real_name, &dependency.version_or_dist_tag, &version
                )))
            }
        }
    }
}

pub fn version_matches(requested_version: &VersionReq, version: &str) -> bool {
    requested_version.matches(&Version::from_str(version).unwrap())
}

pub fn resolve_version(
    parent: &str,
    requested_version: &VersionReq,
    package_metadata: &PackageMetadata,
) -> Result<Version, JmCoreError> {
    let mut matching_versions: Vec<Version> = package_metadata
        .versions
        .keys()
        .filter(|version| version_matches(requested_version, version))
        .map(|version| Version::from_str(version).unwrap())
        .collect();

    if matching_versions.is_empty() {
        return Err(JmCoreError::new(format!(
            "No matching versions for {}->{} (requested {})",
            parent, package_metadata.package_name, requested_version
        )));
    }

    matching_versions.sort();

    Ok(matching_versions.into_iter().last().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npm::VersionMetadata;
    use maplit::hashmap;
    use std::collections::HashMap;

    #[test]
    fn extract_dependency_version_req_dist_tag_ok() {
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

        let result = extract_dependency_version_req(&dependency, &metadata);

        assert_eq!(result, Ok(VersionReq::from_str(&version).unwrap()));
    }

    #[test]
    fn extract_dependency_version_req_dist_tag_not_found() {
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

        let result = extract_dependency_version_req(&dependency, &metadata);

        assert_eq!(
            result,
            Err(JmCoreError::new(format!(
                "Failed to resolve dist tag {} of package {}",
                dist_tag, package_name
            )))
        );
    }

    #[test]
    fn extract_dependency_version_req_dist_tag_no_matching_version() {
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

        let result = extract_dependency_version_req(&dependency, &metadata);

        assert_eq!(
            result,
            Err(JmCoreError::new(format!(
                "{}@{} points to version 1.0.0, which does not exist",
                package_name, dist_tag
            )))
        );
    }

    #[test]
    fn test_version_matches() {
        let version_req = VersionReq::parse("~1.0.0").unwrap();

        assert!(version_matches(&version_req, "1.0.0"));
        assert!(!version_matches(&version_req, "2.0.0"));
    }

    #[test]
    fn resolve_version_finds_the_best_match() {
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

        let result = resolve_version("never-mind", &version_req, &metadata);

        assert_eq!(result, Ok(Version::from_str("1.0.1").unwrap()));
    }

    #[test]
    fn resolve_version_error_when_no_matching_versions_exist() {
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

        let result = resolve_version(&parent, &version_req, &metadata);

        assert_eq!(
            result,
            Err(JmCoreError::new(format!(
                "No matching versions for {}->{} (requested {})",
                parent, package_name, version_req
            )))
        );
    }
}
