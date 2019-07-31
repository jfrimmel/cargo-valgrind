//! A module for deserializing the cargo metadata output.
#[cfg(test)]
mod tests;

use serde::Deserialize;
use std::path::PathBuf;

/// The metadata of the crate to compile and run valgrind on.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Metadata {
    pub packages: Vec<Package>,
    pub target_directory: PathBuf,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Package {
    pub id: String,
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Target {
    pub kind: Vec<Kind>,
    pub crate_types: Vec<CrateType>,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Kind {
    #[serde(rename = "bin")]
    Binary,
    #[serde(rename = "example")]
    Example,
    #[serde(rename = "bench")]
    Bench,
    #[serde(rename = "lib")]
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum CrateType {
    #[serde(rename = "bin")]
    Binary,
    #[serde(rename = "lib")]
    Library,
}
