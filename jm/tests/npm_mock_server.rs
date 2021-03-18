use flate2::write::GzEncoder;
use flate2::Compression;
use httpmock::Method::GET;
use httpmock::MockServer;
use jm::npm::NpmPackageMetadata;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use tempdir::TempDir;
use urlencoding::encode;

pub struct NpmMockServer {
    server: MockServer,
}

impl NpmMockServer {
    pub fn new() -> NpmMockServer {
        let server = MockServer::start();

        NpmMockServer { server }
    }

    pub fn url(&self) -> String {
        self.server.base_url()
    }

    pub fn with_metadata(&mut self, package_name: &str, package_metadata: &NpmPackageMetadata) {
        let expected_path = format!("/{}", encode(package_name));

        self.server.mock(|when, then| {
            when.method(GET).path(expected_path);
            then.status(200)
                .body(serde_json::to_string(package_metadata).unwrap());
        });
    }

    pub fn with_tarball_data(&mut self, package_name: &str, files: HashMap<String, String>) {
        let tmp_dir = TempDir::new("jm-tarballs").unwrap();

        self.write_files(&files, tmp_dir.path().to_path_buf());

        let tar_gz_path = self.write_tarball(package_name, tmp_dir.path().to_str().unwrap());

        let expected_path = format!("/tarball/{}", encode(package_name));

        self.server.mock(|when, then| {
            when.method(GET).path(expected_path);
            then.status(200)
                .header("content-encoding", "gzip")
                .body_from_file(tar_gz_path.to_str().unwrap());
        });
    }

    fn write_files(&self, files: &HashMap<String, String>, to: PathBuf) {
        for (name, content) in files {
            fs::write(to.join(name), content).unwrap();
        }
    }

    fn write_tarball(&self, package_name: &str, files_path: &str) -> PathBuf {
        let tar_gz_path = env::temp_dir().join(package_name);
        fs::create_dir_all(tar_gz_path.parent().unwrap()).unwrap();

        let tar_gz = File::create(&tar_gz_path).unwrap();
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);

        tar.append_dir_all("package", files_path).unwrap();

        tar_gz_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jm::npm::NpmDistMetadata;
    use jm::npm::NpmVersionMetadata;
    use maplit::hashmap;

    #[test]
    fn metadata_works_for_package() {
        let mut server = NpmMockServer::new();
        let client = Client::new();

        let expected = NpmPackageMetadata {
            dist_tags: None,
            versions: hashmap! {
              "1.0.0".to_string() => NpmVersionMetadata {
                dist: NpmDistMetadata {
                  shasum: String::from("some-shasum"),
                  tarball: String::from("some-tarball"),
                },
                dependencies: None,
              },
            },
        };

        server.with_metadata("lodash", &expected);

        let url = format!("{}/lodash", server.url());
        let response = client.get(&url).send().unwrap();

        let status = response.status();
        let body = response.json::<NpmPackageMetadata>().unwrap();

        assert_eq!(status, 200);
        assert_eq!(body, expected);
    }

    #[test]
    fn metadata_works_for_scoped_package() {
        let mut server = NpmMockServer::new();
        let client = Client::new();

        let expected = NpmPackageMetadata {
            dist_tags: None,
            versions: hashmap! {
              "2.0.0".to_string() => NpmVersionMetadata {
                dist: NpmDistMetadata {
                  shasum: String::from("some-shasum"),
                  tarball: String::from("some-tarball"),
                },
                dependencies: None,
              },
            },
        };

        server.with_metadata("@types/lodash", &expected);

        let url = format!("{}/%40types%2Flodash", server.url());
        let response = client.get(&url).send().unwrap();

        let status = response.status();
        let body = response.json::<NpmPackageMetadata>().unwrap();

        assert_eq!(status, 200);
        assert_eq!(body, expected);
    }

    #[test]
    fn download_tarball_works_for_package() {
        let mut server = NpmMockServer::new();
        let client = Client::new();

        let files = hashmap! {
          "file1".to_string() => "hello".to_string()
        };

        server.with_tarball_data("some-lib", files);

        let url = format!("{}/tarball/{}", server.url(), "some-lib");
        let response = client.get(&url).send().unwrap();

        let status = response.status();
        let body = response.text().unwrap();

        assert_eq!(status, 200);
        assert!(body.len() > 0);
    }

    #[test]
    fn download_tarball_works_for_scoped_package() {
        let mut server = NpmMockServer::new();
        let client = Client::new();

        let files = hashmap! {
          "file1".to_string() => "hello".to_string()
        };

        server.with_tarball_data("@types/lodash", files);

        let url = format!("{}/tarball/%40types%2Flodash", server.url());
        let response = client.get(&url).send().unwrap();

        let status = response.status();
        let body = response.text().unwrap();

        assert_eq!(status, 200);
        assert!(body.len() > 0);
    }
}
