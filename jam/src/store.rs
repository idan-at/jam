use jam_common::sanitize_package_name;
use jam_core::package::NpmPackage;
use crate::JamError;
use std::path::Path;
use std::path::PathBuf;
use std::fs;

pub struct Store {
  store_path: PathBuf,
}

impl Store {
  pub fn new(data_dir: &Path) -> Result<Store, JamError> {
    let store_path = data_dir.join("store");

    fs::create_dir_all(&store_path)?;

    Ok(Store { store_path })
  }

  pub fn package_path_in_store(&self, package: &NpmPackage) -> PathBuf {
    let package_dir_name = format!(
        "{}@{}",
        sanitize_package_name(&package.name),
        package.version
    );

    self.store_path.join(package_dir_name)
  }
}

#[cfg(test)]
mod tests {
  use jam_test_utils::sync_helpers::with_tmp_dir;
use super::*;

  #[test]
  fn creates_store_on_initialization() {
    with_tmp_dir(|path| {
      let result = Store::new(&path);

      assert!(result.is_ok());
      assert!(path.join("store").exists());
    })
  }

  #[test]
  fn returns_package_path_in_store() {
    with_tmp_dir(|path| {
      let store = Store::new(&path).unwrap();

      let npm_package = NpmPackage::new("package_name".to_string(), "1.0.0".to_string(), None, "shasum".to_string(), "tarball".to_string());

      let package_path = store.package_path_in_store(&npm_package);

      assert_eq!(package_path, path.join("store").join("package_name@1.0.0"));
    })
  }

  #[test]
  fn returns_scoped_package_path_in_store() {
    with_tmp_dir(|path| {
      let store = Store::new(&path).unwrap();

      let npm_package = NpmPackage::new("@scope/package_name".to_string(), "1.0.0".to_string(), None, "shasum".to_string(), "tarball".to_string());

      let package_path = store.package_path_in_store(&npm_package);

      assert_eq!(package_path, path.join("store").join("@scope_package_name@1.0.0"));
    })
  }
}
