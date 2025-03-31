//! Build-script to automatically generate a list of suppression files ready for
//! inclusion into the binary. The [`SUPPRESSIONS_DIR`] is searched for files
//! any their entire content is written in a structured way to a source file, so
//! that their contents is embedded into the resulting binary.

/// The directory containing the suppression files.
const SUPPRESSIONS_DIR: &str = "suppressions";

use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::{env, fs};

/// Search for suppression files inside the [`SUPPRESSIONS_DIR`].
///
/// This will return an iterator over the contents of the files, that may be
/// suitable for Valgrind suppression files. There is no recursive search.
/// Non-readable entries and other I/O errors are ignored.
fn search_suppressions() -> impl Iterator<Item = String> {
    fs::read_dir(SUPPRESSIONS_DIR)
        .expect("could not find the suppression directory")
        .filter_map(|entry| entry.ok())
        .filter(|path| path.file_type().map_or(false, |path| path.is_file()))
        .map(|file| file.path())
        .filter_map(|path| fs::read_to_string(path).ok())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create file `$OUT_DIR/suppressions.rs`, which will be written to later
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets $OUT_DIR"));
    let out_file = fs::File::create(out_dir.join("suppressions.rs"))?;
    let mut out_file = BufWriter::new(out_file);

    // Create slice containing all successfully read suppression file contents.
    // This will generate `const SUPPRESSIONS: &[&str] = &[...];` with the full
    // file contents as strings for each file in order of the iterations.
    out_file.write_all(b"const SUPPRESSIONS: &[&str] = &[")?;
    for file in search_suppressions() {
        out_file.write_all(b"r\"")?;
        out_file.write_all(file.as_bytes())?;
        out_file.write_all(b"\",")?;
    }
    out_file.write_all(b"];")?;

    // Cargo should monitor the whole directory for changes/new files, so that
    // this build script is run on new/changed suppression files.
    println!("cargo:rerun-if-changed={SUPPRESSIONS_DIR}");
    Ok(())
}
