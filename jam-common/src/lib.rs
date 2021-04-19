use jam_npm_metadata::NpmBinMetadata;
use std::collections::HashMap;

pub fn sanitize_package_name(package_name: &str) -> String {
    package_name.replace("/", "_")
}

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
    use maplit::hashmap;

    #[test]
    fn sanitized_package_names() {
        let package_name = "@scope/name";

        assert_eq!(
            sanitize_package_name(package_name),
            String::from("@scope_name")
        );
    }

    #[test]
    fn extract_binaries_none() {
        let package_name = "name";
        let bin = None;

        assert_eq!(
            extract_binaries(&package_name, &bin),
            hashmap! {}
        );
    }

    #[test]
    fn extract_binaries_string() {
        let package_name = "name";
        let bin = Some(NpmBinMetadata::String("./a.js".to_string()));

        assert_eq!(
            extract_binaries(&package_name, &bin),
            hashmap! {
                package_name.to_string() => "./a.js".to_string(),
            }
        );
    }

    #[test]
    fn extract_binaries_object() {
        let package_name = "name";
        let bin_object = hashmap! {
            "script".to_string() => "./a.js".to_string(),
        };
        let bin = Some(NpmBinMetadata::Object(bin_object.clone()));

        assert_eq!(
            extract_binaries(&package_name, &bin),
            bin_object
        );
    }
}
