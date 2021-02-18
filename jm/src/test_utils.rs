use tempdir::TempDir;

pub fn create_tmp_dir() -> TempDir {
  let tmp_dir = TempDir::new("jm_workspaces_fixtures").unwrap();

  tmp_dir
}
