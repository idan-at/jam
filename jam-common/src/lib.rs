use jam_npm_metadata::NpmBinMetadata;
use std::collections::HashMap;

pub fn sanitize_package_name(package_name: &str) -> String {
    package_name.replace("/", "_")
}

// TODO: test
pub fn extract_binaries(
    package_name: &str,
    bin: &Option<NpmBinMetadata>,
) -> HashMap<String, String> {
    let mut binaries = HashMap::new();

    if let Some(bin) = bin {
        match bin {
            NpmBinMetadata::String(path) => {
                binaries.insert(package_name.to_string(), path.clone());
            }
            NpmBinMetadata::Object(object) => binaries.extend(object.clone()),
        };
    }

    binaries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitized_package_names() {
        let package_name = "@scope/name";

        assert_eq!(
            sanitize_package_name(package_name),
            String::from("@scope_name")
        );
    }
}
