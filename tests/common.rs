use std::fs;
use tempdir::TempDir;

fn create_tmp_dir() -> TempDir {
    let tmp_dir = TempDir::new("jm_fixtures").unwrap();

    tmp_dir
}

fn with_manifest_file(dir: &TempDir, content: &str) {
    let file_path = dir.path().join("package.json");

    fs::write(file_path, content).unwrap();
}

pub fn given_manifest_file_does_not_exist() -> TempDir {
    create_tmp_dir()
}

pub fn given_valid_manifest_file() -> TempDir {
    let tmp_dir = create_tmp_dir();
    with_manifest_file(&tmp_dir, r#"{ "workspaces": ["**/*"] }"#);

    tmp_dir
}
