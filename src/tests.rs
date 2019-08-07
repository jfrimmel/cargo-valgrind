use super::{
    binaries_from,
    metadata::{CrateType, Kind, Metadata, Package, Target},
    Build,
};
use std::path::PathBuf;

#[test]
fn multiple_binaries_are_supported() {
    let manifest = PathBuf::from("a/test/Cargo.toml");
    let metadata = Metadata {
        target_directory: "test-dir".into(),
        version: 1,
        packages: vec![Package {
            id: "a test crate".into(),
            manifest_path: manifest.clone(),
            targets: vec![
                Target {
                    crate_types: vec![CrateType::Binary],
                    kind: vec![Kind::Binary],
                    name: "a binary".into(),
                },
                Target {
                    crate_types: vec![CrateType::Binary],
                    kind: vec![Kind::Binary],
                    name: "another binary".into(),
                },
            ],
        }],
    };
    assert_eq!(
        binaries_from(metadata, manifest, Build::Debug).unwrap(),
        vec![
            crate::Target::Binary(PathBuf::from("test-dir/debug/a binary")),
            crate::Target::Binary(PathBuf::from("test-dir/debug/another binary")),
        ]
    );
}

#[test]
fn examples_are_supported() {
    let manifest = PathBuf::from("a/test/Cargo.toml");
    let metadata = Metadata {
        target_directory: "test-dir".into(),
        version: 1,
        packages: vec![Package {
            id: "a test crate".into(),
            manifest_path: manifest.clone(),
            targets: vec![Target {
                crate_types: vec![CrateType::Binary],
                kind: vec![Kind::Example],
                name: "an example".into(),
            }],
        }],
    };
    assert_eq!(
        binaries_from(metadata, manifest, Build::Debug).unwrap(),
        vec![crate::Target::Example(PathBuf::from(
            "test-dir/debug/examples/an example"
        ))]
    );
}

#[test]
fn benches_are_supported() {
    let manifest = PathBuf::from("a/test/Cargo.toml");
    let metadata = Metadata {
        target_directory: "test-dir".into(),
        version: 1,
        packages: vec![Package {
            id: "a test crate".into(),
            manifest_path: manifest.clone(),
            targets: vec![Target {
                crate_types: vec![CrateType::Binary],
                kind: vec![Kind::Bench],
                name: "a benchmark".into(),
            }],
        }],
    };
    assert_eq!(
        binaries_from(metadata, manifest, Build::Debug).unwrap(),
        vec![crate::Target::Benchmark(PathBuf::from(
            "test-dir/debug/benches/a benchmark"
        ))]
    );
}

#[test]
fn libraries_are_ignored() {
    let manifest = PathBuf::from("a/test/Cargo.toml");
    let metadata = Metadata {
        target_directory: "test-dir".into(),
        version: 1,
        packages: vec![Package {
            id: "a test crate".into(),
            manifest_path: manifest.clone(),
            targets: vec![
                Target {
                    crate_types: vec![CrateType::Library],
                    kind: vec![Kind::Library],
                    name: "a lib".into(),
                },
                Target {
                    crate_types: vec![CrateType::DyLib],
                    kind: vec![Kind::DyLib],
                    name: "a dylib".into(),
                },
                Target {
                    crate_types: vec![CrateType::CDyLib],
                    kind: vec![Kind::CDyLib],
                    name: "a cdylib".into(),
                },
                Target {
                    crate_types: vec![CrateType::StaticLib],
                    kind: vec![Kind::StaticLib],
                    name: "a static lib".into(),
                },
                Target {
                    crate_types: vec![CrateType::RLib],
                    kind: vec![Kind::RLib],
                    name: "an rlib".into(),
                },
            ],
        }],
    };
    assert_eq!(
        binaries_from(metadata, manifest, Build::Debug).unwrap(),
        Vec::<_>::new()
    );
}

#[test]
fn proc_macros_are_ignored() {
    let manifest = PathBuf::from("a/test/Cargo.toml");
    let metadata = Metadata {
        target_directory: "test-dir".into(),
        version: 1,
        packages: vec![Package {
            id: "a test crate".into(),
            manifest_path: manifest.clone(),
            targets: vec![Target {
                crate_types: vec![CrateType::ProcMacro],
                kind: vec![Kind::ProcMacro],
                name: "a proc macro".into(),
            }],
        }],
    };
    assert_eq!(
        binaries_from(metadata, manifest, Build::Debug).unwrap(),
        Vec::<_>::new()
    );
}

#[test]
fn only_binaries_of_manifest_are_returned() {
    let manifest = PathBuf::from("a/test/Cargo.toml");
    let metadata = Metadata {
        target_directory: "test-dir".into(),
        version: 1,
        packages: vec![
            Package {
                id: "a test crate".into(),
                manifest_path: manifest.clone(),
                targets: vec![
                    Target {
                        crate_types: vec![CrateType::Binary],
                        kind: vec![Kind::Binary],
                        name: "a binary".into(),
                    },
                    Target {
                        crate_types: vec![CrateType::Binary],
                        kind: vec![Kind::Binary],
                        name: "another binary".into(),
                    },
                ],
            },
            Package {
                id: "another crate of the workspace".into(),
                manifest_path: manifest.clone().join("sub-crate"),
                targets: vec![Target {
                    crate_types: vec![CrateType::Binary],
                    kind: vec![Kind::Binary],
                    name: "wrong binary".into(),
                }],
            },
        ],
    };
    assert_eq!(
        binaries_from(metadata, manifest, Build::Debug).unwrap(),
        vec![
            crate::Target::Binary(PathBuf::from("test-dir/debug/a binary")),
            crate::Target::Binary(PathBuf::from("test-dir/debug/another binary")),
        ]
    );
}
