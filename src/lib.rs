//! The core library of the `cargo-valgrind` command.
mod metadata;

use std::{
    io,
    path::{Path, PathBuf},
    process::Command,
};

/// Query all binaries of the crate denoted by the given `Cargo.toml`.
///
/// This function returns the paths to each executable in the given crate. Those
/// are all the examples, benches as the actual crate binaries. This is based on
/// the crate metadata obtained by [`metadata()`](fn.metadata.html).
///
/// # Errors
/// This function fails for the same reasons as the `metadata()` function.
pub fn binaries<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, io::Error> {
    let package = metadata(path)?;

    let target_dir = package.target_directory.join("debug");
    Ok(package
        .packages
        .into_iter()
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
                            metadata::Kind::Library => unreachable!(),
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
        let msg = String::from_utf8_lossy(&output.stderr);
        let msg = msg.trim_start_matches("error: ").trim_end();
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("cargo command failed: {}", msg),
        ))
    }
}
