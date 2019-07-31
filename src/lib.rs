//! The core library of the `cargo-valgrind` command.
mod metadata;

use std::{path::Path, io, process::Command};

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
