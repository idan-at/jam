use tempdir::TempDir;

pub fn create_tmp_dir() -> TempDir {
    TempDir::new("jm_fixtures").unwrap()
}

pub fn with_manifest_file_content(workspaces: Vec<&str>) -> String {
    let workspaces = workspaces
        .iter()
        .map(|workspace| format!(r#""{}""#, workspace))
        .collect::<Vec<String>>()
        .join(", ");

    format!(r#"{{ "workspaces": [{}] }}"#, workspaces)
}

pub fn with_package_json_file_content(name: &str, version: &str) -> String {
    String::from(format!(
        r#"{{
        "name": "{}",
        "version": "{}"
    }}"#,
        name, version
    ))
}
