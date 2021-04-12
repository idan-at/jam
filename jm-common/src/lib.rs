pub fn sanitize_package_name(package_name: &str) -> String {
    package_name.replace("/", "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitized_package_names() {
        let package_name = "@scope/name";

        assert_eq!(sanitize_package_name(package_name), String::from("@scope_name"));
    }
}
