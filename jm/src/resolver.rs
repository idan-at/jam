use crate::npm::PackageMetadata;
use semver::{Compat, Version, VersionReq};
use std::collections::HashSet;
use std::str::FromStr;

fn translate_dist_tag_to_version(
    package_name: &str,
    requested_dist_tag: &str,
    package_metadata: &PackageMetadata,
) -> VersionReq {
    let dist_tag = package_metadata
        .dist_tags
        .keys()
        .find(|dist_tag| **dist_tag == requested_dist_tag.to_string())
        .expect(&format!(
            "Failed to resolve dist tag {} of package {}",
            requested_dist_tag, package_name
        ));

    VersionReq::parse(package_metadata.dist_tags.get(dist_tag).unwrap()).unwrap()
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
    let mut sorted_versions = versions.into_iter().collect::<Vec<&VersionReq>>();
    sorted_versions.sort();

    sorted_versions.last().unwrap().to_string()
}

pub fn get_package_exact_version(
    package_name: &str,
    version_or_dist_tag: &str,
    package_metadata: &PackageMetadata,
) -> String {
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
    // TODO: handle the case where no compatible_versions were found

    let version = get_best_matching_version(&compatible_versions);
    let without_equal_prefix = &version[1..];

    String::from(without_equal_prefix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npm::{PackageMetadata, VersionMetadata};
    use maplit::hashmap;
    use std::collections::HashMap;

    fn with_package_metadata() -> PackageMetadata {
        PackageMetadata {
            package_name: "some-package".to_string(),
            dist_tags: hashmap! {
                "latest".to_string() => "2.0.0".to_string(),
            },
            versions: hashmap! {
                "1.0.1".to_string() => VersionMetadata {
                    shasum: "some-shasum".to_string(),
                    tarball: "some-tarball".to_string(),
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                },
                "1.1.0".to_string() => VersionMetadata {
                    shasum: "some-shasum".to_string(),
                    tarball: "some-tarball".to_string(),
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                },
                "2.0.0".to_string() => VersionMetadata {
                    shasum: "some-shasum".to_string(),
                    tarball: "some-tarball".to_string(),
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                },
            },
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
