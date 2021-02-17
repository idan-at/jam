use crate::npm::PackageMetadata;
use semver::{Compat, Version, VersionReq};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

fn translate_dist_tag_to_version(
    package_name: &str,
    requested_dist_tag: &str,
    package_metadata: &PackageMetadata,
) -> VersionReq {
    let default_dist_tags = HashMap::new();
    let dist_tags = package_metadata
        .dist_tags
        .as_ref()
        .unwrap_or(&default_dist_tags);

    let dist_tag = dist_tags
        .keys()
        .find(|dist_tag| **dist_tag == requested_dist_tag.to_string())
        .expect(&format!(
            "Failed to resolve dist tag {} of package {}",
            requested_dist_tag, package_name
        ));

    VersionReq::parse(dist_tags.get(dist_tag).unwrap()).unwrap()
}

fn filter_compatible_versions(
    requested_version: &VersionReq,
    package_metadata: &PackageMetadata,
) -> HashSet<VersionReq> {
    package_metadata
        .versions
        .keys()
        .filter(|version| requested_version.matches(&Version::from_str(version).unwrap()))
        .map(|version| VersionReq::parse_compat(version, Compat::Npm).unwrap())
        .collect()
}

fn get_best_matching_version(versions: &HashSet<VersionReq>) -> String {
    println!("choosing from {:?}", versions);

    let mut sorted_versions = versions.into_iter().collect::<Vec<&VersionReq>>();
    sorted_versions.sort();

    println!("sorted {:?}", versions);

    println!("chose {}", sorted_versions.last().unwrap().to_string());

    sorted_versions.last().unwrap().to_string()
}

pub fn get_minimal_package_versions(
    packages_requested_versions: HashMap<String, HashSet<String>>,
    packages_metadata: &Vec<PackageMetadata>,
) -> HashMap<String, HashSet<String>> {
    let mut results: HashMap<String, HashSet<String>> = HashMap::new();

    for (package_name, package_requested_versions) in packages_requested_versions {
        for version_or_dist_tag in package_requested_versions {
            let package_metadata = packages_metadata
                .iter()
                .find(|package_metadata| package_metadata.package_name == package_name)
                .unwrap();

            let package_requested_version =
                match VersionReq::parse_compat(&version_or_dist_tag, Compat::Npm) {
                    Ok(version) => version,
                    Err(_) => translate_dist_tag_to_version(
                        &package_name,
                        &version_or_dist_tag,
                        &package_metadata,
                    ),
                };

            let compatible_versions =
                filter_compatible_versions(&package_requested_version, &package_metadata);

            println!("for package {}", package_name);

            let best_matching_version = get_best_matching_version(&compatible_versions);

            if let Some(versions) = results.get_mut(&package_name) {
                versions.insert(best_matching_version);
            } else {
                let versions_set: HashSet<String> =
                    vec![best_matching_version].iter().cloned().collect();

                results.insert(package_name.to_string(), versions_set);
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npm::{DistMetadata, PackageMetadata, VersionMetadata};

    fn with_package_metadata() -> PackageMetadata {
        PackageMetadata {
            package_name: "some-package".to_string(),
            dist_tags: Some(
                vec![("latest".to_string(), "2.0.0".to_string())]
                    .into_iter()
                    .collect(),
            ),
            versions: vec![
                (
                    "1.0.1".to_string(),
                    VersionMetadata {
                        dist: DistMetadata {
                            shasum: "some-shasum".to_string(),
                            tarball: "some-tarball".to_string(),
                        },
                    },
                ),
                (
                    "1.1.0".to_string(),
                    VersionMetadata {
                        dist: DistMetadata {
                            shasum: "some-shasum".to_string(),
                            tarball: "some-tarball".to_string(),
                        },
                    },
                ),
                (
                    "2.0.0".to_string(),
                    VersionMetadata {
                        dist: DistMetadata {
                            shasum: "some-shasum".to_string(),
                            tarball: "some-tarball".to_string(),
                        },
                    },
                ),
            ]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn filter_compatible_versions_tilde() {
        let requested_version = VersionReq::parse_compat("~1.0.0", Compat::Npm).unwrap();
        let packages_metadata = with_package_metadata();

        let result = filter_compatible_versions(&requested_version, &packages_metadata);
        let expected = vec![VersionReq::parse_compat("1.0.1", Compat::Npm).unwrap()]
            .into_iter()
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn filter_compatible_versions_carrot() {
        let requested_version = VersionReq::parse_compat("^1.0.0", Compat::Npm).unwrap();
        let packages_metadata = with_package_metadata();

        let result = filter_compatible_versions(&requested_version, &packages_metadata);
        let expected = vec![
            VersionReq::parse_compat("1.0.1", Compat::Npm).unwrap(),
            VersionReq::parse_compat("1.1.0", Compat::Npm).unwrap(),
        ]
        .into_iter()
        .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn filter_compatible_versions_asterisks() {
        let requested_version = VersionReq::parse_compat("*", Compat::Npm).unwrap();
        let packages_metadata = with_package_metadata();

        let result = filter_compatible_versions(&requested_version, &packages_metadata);
        let expected = vec![
            VersionReq::parse_compat("2.0.0", Compat::Npm).unwrap(),
            VersionReq::parse_compat("1.1.0", Compat::Npm).unwrap(),
            VersionReq::parse_compat("1.0.1", Compat::Npm).unwrap(),
        ]
        .into_iter()
        .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn filter_compatible_versions_gt() {
        let requested_version = VersionReq::parse_compat(">1.0.1", Compat::Npm).unwrap();
        let packages_metadata = with_package_metadata();

        let result = filter_compatible_versions(&requested_version, &packages_metadata);
        let expected = vec![
            VersionReq::parse_compat("2.0.0", Compat::Npm).unwrap(),
            VersionReq::parse_compat("1.1.0", Compat::Npm).unwrap(),
        ]
        .into_iter()
        .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn filter_compatible_versions_ge() {
        let requested_version = VersionReq::parse_compat(">=1.0.1", Compat::Npm).unwrap();
        let packages_metadata = with_package_metadata();

        let result = filter_compatible_versions(&requested_version, &packages_metadata);
        let expected = vec![
            VersionReq::parse_compat("2.0.0", Compat::Npm).unwrap(),
            VersionReq::parse_compat("1.1.0", Compat::Npm).unwrap(),
            VersionReq::parse_compat("1.0.1", Compat::Npm).unwrap(),
        ]
        .into_iter()
        .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn filter_compatible_versions_lt() {
        let requested_version = VersionReq::parse_compat("<2.0.0", Compat::Npm).unwrap();
        let packages_metadata = with_package_metadata();

        let result = filter_compatible_versions(&requested_version, &packages_metadata);
        let expected = vec![
            VersionReq::parse_compat("1.0.1", Compat::Npm).unwrap(),
            VersionReq::parse_compat("1.1.0", Compat::Npm).unwrap(),
        ]
        .into_iter()
        .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn filter_compatible_versions_le() {
        let requested_version = VersionReq::parse_compat("<=2.0.0", Compat::Npm).unwrap();
        let packages_metadata = with_package_metadata();

        let result = filter_compatible_versions(&requested_version, &packages_metadata);
        let expected = vec![
            VersionReq::parse_compat("2.0.0", Compat::Npm).unwrap(),
            VersionReq::parse_compat("1.1.0", Compat::Npm).unwrap(),
            VersionReq::parse_compat("1.0.1", Compat::Npm).unwrap(),
        ]
        .into_iter()
        .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn filter_compatible_versions_exact() {
        let requested_version = VersionReq::parse_compat("1.0.1", Compat::Npm).unwrap();
        let packages_metadata = with_package_metadata();

        let result = filter_compatible_versions(&requested_version, &packages_metadata);
        let expected = vec![VersionReq::parse_compat("1.0.1", Compat::Npm).unwrap()]
            .into_iter()
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn filter_compatible_versions_no_match() {
        let requested_version = VersionReq::parse_compat("1.0.3", Compat::Npm).unwrap();
        let packages_metadata = with_package_metadata();

        let result = filter_compatible_versions(&requested_version, &packages_metadata);
        let expected = HashSet::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn translate_dist_tag_to_version_exists() {
        let package_metadata = with_package_metadata();

        let result = translate_dist_tag_to_version(
            &package_metadata.package_name,
            "latest",
            &package_metadata,
        );

        assert_eq!(result, VersionReq::from_str("2.0.0").unwrap());
    }

    #[test]
    #[should_panic(
        expected = "Failed to resolve dist tag non-existing-dist-tag of package some-package"
    )]
    fn translate_dist_tag_to_version_does_not_exist() {
        let package_metadata = with_package_metadata();

        translate_dist_tag_to_version(
            &package_metadata.package_name,
            "non-existing-dist-tag",
            &package_metadata,
        );
    }
}
