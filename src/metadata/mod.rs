//! A module for deserializing the cargo metadata output.
#![allow(clippy::missing_docs_in_private_items)]

#[cfg(test)]
mod tests;
mod version;

use serde::Deserialize;
use std::path::PathBuf;

/// The metadata of the crate to compile and run valgrind on.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Metadata {
    pub packages: Vec<Package>,
    pub target_directory: PathBuf,
    #[serde(deserialize_with = "version::deserialize")]
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Package {
    pub id: String,
    pub targets: Vec<Target>,
    pub manifest_path: PathBuf,
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
    #[serde(rename = "dylib")]
    DyLib,
    #[serde(rename = "cdylib")]
    CDyLib,
    #[serde(rename = "staticlib")]
    StaticLib,
    #[serde(rename = "rlib")]
    RLib,
    #[serde(rename = "test")]
    Test,
    #[serde(rename = "proc-macro")]
    ProcMacro,
    #[serde(rename = "custom-build")]
    CustomBuild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum CrateType {
    #[serde(rename = "bin")]
    Binary,
    #[serde(rename = "lib")]
    Library,
    #[serde(rename = "dylib")]
    DyLib,
    #[serde(rename = "cdylib")]
    CDyLib,
    #[serde(rename = "staticlib")]
    StaticLib,
    #[serde(rename = "rlib")]
    RLib,
    #[serde(rename = "proc-macro")]
    ProcMacro,
}
