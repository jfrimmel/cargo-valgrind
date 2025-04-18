//! Build-script to automatically generate the contents of a suppression file
//! ready for inclusion into the binary and to be passed to Valgrind.
//!
//! The [`SUPPRESSIONS_DIR`] is searched for files any their entire content is
//! written as a string constant to a source file, so that their contents is
//! embedded into the resulting binary.

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
        .filter(|file| file.file_name() != "README.md")
        .map(|file| file.path())
        .filter_map(|path| fs::read_to_string(path).ok())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create file `$OUT_DIR/suppressions.rs`, which will be written to later
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets $OUT_DIR"));
    let out_file = fs::File::create(out_dir.join("suppressions.rs"))?;
    let mut out_file = BufWriter::new(out_file);

    // Create the file contents of the suppression file given to valgrind. This
    // file contains all the individual suppressions joined together into a long
    // string, so that the application can use that single string.
    // Normally, this should be stored as a fixed `OsStr`, since it is not re-
    // quired to be UTF-8 (the interpretation is done by Valgrind alone), but
    // this is not possible in Rust at the moment. Therefore it is actually
    // stored as a string (which makes handling the file contents in this build
    // script easier as well). So, this will generate a Rust string constant
    // (`const SUPPRESSIONS: &str = "...";`) with all the file contents joined
    // together of each file in order of the iterations.
    out_file.write_all(b"/// Rust-std suppression file contents generated by build script\n")?;
    out_file.write_all(b"const SUPPRESSIONS: &str = \"")?;
    for file in search_suppressions() {
        out_file.write_all(file.as_bytes())?;
    }
    out_file.write_all(b"\";")?;

    // Cargo should monitor the whole directory for changes/new files, so that
    // this build script is run on new/changed suppression files.
    println!("cargo:rerun-if-changed={SUPPRESSIONS_DIR}");
    Ok(())
}
