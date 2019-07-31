//! The core library of the `cargo-valgrind` command.
mod metadata;
#[cfg(test)]
mod tests;

use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// The possible build types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Build {
    /// This is a debug build.
    Debug,
    /// This is a release build.
    Release,
}
impl Default for Build {
    fn default() -> Self {
        Build::Debug
    }
}
impl AsRef<Path> for Build {
    fn as_ref(&self) -> &Path {
        match self {
            Build::Debug => Path::new("debug"),
            Build::Release => Path::new("release"),
        }
    }
}

/// The possible targets to build and run within valgrind.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Target {
    /// A normal binary with the given name.
    Binary(OsString),
    /// An example with the given name.
    Example(OsString),
    /// A benchmark with the given name.
    Benchmark(OsString),
    /// A test with the given name.
    Test(OsString),
}

/// Invoke `cargo` and build the specified target.
///
/// The crate is specified by the path to the `Cargo.toml` using the `manifest`
/// parameter. The kind of build (debug or release) is selected via the `build`
/// parameter. The binary to build is specified via the `target` parameter.
///
/// # Errors
/// This function returns an error, if the `cargo command returned an error`.
pub fn build_target<P: AsRef<Path>>(
    manifest: P,
    build: Build,
    target: Target,
) -> Result<(), io::Error> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if let Build::Release = build {
        cmd.arg("--release");
    }
    cmd.arg("--manifest-path");
    cmd.arg(manifest.as_ref());
    match target {
        Target::Binary(_) => cmd.arg("--bin"),
        Target::Example(_) => cmd.arg("--example"),
        Target::Benchmark(_) => cmd.arg("--bench"),
        Target::Test(_) => cmd.arg("--test"),
    };
    match target {
        Target::Binary(name)
        | Target::Example(name)
        | Target::Benchmark(name)
        | Target::Test(name) => cmd.arg(name),
    };
    cmd.spawn()?.wait_with_output().and_then(|output| {
        if output.status.success() {
            Ok(())
        } else {
            Err(cargo_error(output))
        }
    })
}

/// Query all binaries of the crate denoted by the given `Cargo.toml`.
///
/// This function returns the paths to each executable in the given crate. Those
/// are all the examples, benches as the actual crate binaries. This is based on
/// the crate metadata obtained by [`metadata()`](fn.metadata.html).
///
/// Only binaries of the specified manifest are returned. This means, that other
/// crates in the same workspace may have binaries, but they are ignored.
///
/// Note, that plain tests and `custom-build` kinds currently are not supported.
///
/// # Errors
/// This function fails for the same reasons as the `metadata()` function.
///
/// # Panics
/// This function currently panics, if a test or custom build binary is
/// encountered.
pub fn binaries<P: AsRef<Path>>(path: P, build: Build) -> Result<Vec<PathBuf>, io::Error> {
    let package = metadata(&path)?;
    let path = path.as_ref().canonicalize()?;
    binaries_from(package, path, build)
}

/// Query all binaries of given metadata.
///
/// See [`binaries()`](fn.binaries.html) for details.
///
/// This is the real implementation of the `binaries()` function. It was added
/// in order to be able to test this function without actual `Cargo.toml`s and
/// by giving prepared metadata.
///
/// Note, that the path denoted by `requested` has to be canonicalized before.
fn binaries_from<P: AsRef<Path>>(
    package: metadata::Metadata,
    requested: P,
    build: Build,
) -> Result<Vec<PathBuf>, io::Error> {
    let target_dir = package.target_directory.join(build);
    Ok(package
        .packages
        .into_iter()
        .filter(|package| package.manifest_path == requested.as_ref())
        .flat_map(|package| {
            package
                .targets
                .into_iter()
                .filter(|target| target.crate_types.contains(&metadata::CrateType::Binary))
                .map(|target| {
                    target_dir
                        .join(match target.kind[0] {
                            metadata::Kind::Binary => "",
                            metadata::Kind::Example => "examples",
                            metadata::Kind::Bench => "benches",
                            metadata::Kind::Test | metadata::Kind::CustomBuild => unimplemented!(),
                            metadata::Kind::Library
                            | metadata::Kind::ProcMacro
                            | metadata::Kind::DyLib
                            | metadata::Kind::CDyLib
                            | metadata::Kind::StaticLib
                            | metadata::Kind::RLib => unreachable!("Non-binaries are filtered out"),
                        })
                        .join(target.name)
                })
        })
        .collect())
}

/// Query the crate metadata of the given `Cargo.toml`.
///
/// This collects the metadata of the crate denoted by the `path` using the
/// [`cargo_metadata()`](fn.cargo_metadata.html) function. Its output is then
/// parsed into the `Metadata` structure.
///
/// # Errors
/// This function either fails because of an error of the `cargo_metadata()`
/// function or due to an invalid output by it, that could not successfully be
/// parsed.
fn metadata<P: AsRef<Path>>(path: P) -> Result<metadata::Metadata, io::Error> {
    let metadata = cargo_metadata(path)?;
    serde_json::from_str(&metadata)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Invalid metadata: {}", e)))
}

/// Run the `cargo metadata` command and collect its output.
///
/// The `path` has to point to the `Cargo.toml` of which the metadata should be
/// collected. Metadata of the dependencies is omitted on purpose. The output is
/// then converted into a `String`.
///
/// # Errors
/// This function can fail either because the `cargo metadata` command could not
/// be spawned, the command failed (i.e. it was executed but returned a non-zero
/// exit code) or the string printed to stdout was not valid UTF-8.
fn cargo_metadata<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(path.as_ref())
        .output()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Non-UTF-8 string"))
    } else {
        Err(cargo_error(output))
    }
}

/// Build an `io::Error` from the stderr text outputted by `cargo`.
fn cargo_error(output: Output) -> io::Error {
    let msg = String::from_utf8_lossy(&output.stderr);
    let msg = msg.trim_start_matches("error: ").trim_end();
    io::Error::new(
        io::ErrorKind::Other,
        format!("cargo command failed: {}", msg),
    )
}
