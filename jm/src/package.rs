use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct PackageNode {
    pub name: String,
    pub version: String,
}
