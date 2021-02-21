use tempdir::TempDir;

pub fn create_tmp_dir() -> TempDir {
    TempDir::new("jm_fixtures").unwrap()
}

pub fn get_manifest_file_content(workspaces: Vec<&str>) -> String {
    let workspaces = workspaces
        .iter()
        .map(|workspace| format!(r#""{}""#, workspace))
        .collect::<Vec<String>>()
        .join(", ");

    format!(r#"{{ "workspaces": [{}] }}"#, workspaces)
}
