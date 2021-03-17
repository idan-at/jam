use semver::{Compat, VersionReq};

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct Dependency {
    pub name: String,
    pub real_name: String,
    pub version_or_dist_tag: String,
}

impl Dependency {
    pub fn from_entry(key: &str, value: &str) -> Dependency {
        match VersionReq::parse_compat(&value, Compat::Npm) {
            Ok(_) => Dependency {
                name: key.to_string(),
                real_name: key.to_string(),
                version_or_dist_tag: value.to_string(),
            },
            Err(_) => {
                if value.starts_with("npm:") {
                    let segments: Vec<&str> = value
                        .split("npm:")
                        .collect::<Vec<&str>>()
                        .get(1)
                        .unwrap()
                        .split("@")
                        .collect();

                    if segments.len() == 2 {
                        Dependency {
                            name: key.to_string(),
                            real_name: segments[0].to_string(),
                            version_or_dist_tag: segments[1].to_string(),
                        }
                    } else {
                        Dependency {
                            name: key.to_string(),
                            real_name: format!("@{}", segments[1]),
                            version_or_dist_tag: segments[2].to_string(),
                        }
                    }
                } else {
                    Dependency {
                        name: key.to_string(),
                        real_name: key.to_string(),
                        version_or_dist_tag: value.to_string(),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_semver_version() {
        let key = "lodash";
        let version = "~1.0.0";

        let expected = Dependency {
            real_name: key.to_string(),
            name: key.to_string(),
            version_or_dist_tag: version.to_string(),
        };

        let result = Dependency::from_entry(key, version);

        assert_eq!(result, expected);
    }

    #[test]
    fn with_dist_tag_version() {
        let key = "lodash";
        let version = "latest";

        let expected = Dependency {
            real_name: key.to_string(),
            name: key.to_string(),
            version_or_dist_tag: version.to_string(),
        };

        let result = Dependency::from_entry(key, version);

        assert_eq!(result, expected);
    }

    #[test]
    fn with_alias_version() {
        let key = "lol";
        let version = "npm:lodash@latest";

        let expected = Dependency {
            real_name: "lodash".to_string(),
            name: key.to_string(),
            version_or_dist_tag: "latest".to_string(),
        };

        let result = Dependency::from_entry(key, version);

        assert_eq!(result, expected);
    }

    #[test]
    fn with_scoped_alias_version() {
        let key = "lol-types";
        let version = "npm:@types/lodash@latest";

        let expected = Dependency {
            real_name: "@types/lodash".to_string(),
            name: key.to_string(),
            version_or_dist_tag: "latest".to_string(),
        };

        let result = Dependency::from_entry(key, version);

        assert_eq!(result, expected);
    }
}
