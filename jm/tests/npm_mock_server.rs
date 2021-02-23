use httpmock::Method::GET;
use httpmock::MockServer;
use jm::npm::NpmPackageMetadata;
use reqwest::blocking::Client;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use jm::npm::NpmDistMetadata;
    use jm::npm::NpmVersionMetadata;
    use maplit::hashmap;

    #[test]
    fn works_for_package() {
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
    fn works_for_scoped_package() {
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
}
