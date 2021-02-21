use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
}
