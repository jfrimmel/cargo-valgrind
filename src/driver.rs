//! A module providing the wrapping driver for a custom runner.

use std::env;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus};

/// The prefix line for the target host output.
const HOST_PREFIX: &str = "host: ";

/// Search for [`HOST_PREFIX`] inside the command output and extract its value.
fn search_for_host(command: &mut Command) -> Option<String> {
    let output = command.output().ok()?.stdout;
    let output = String::from_utf8(output).ok()?;

    output
        .lines()
        .find(|line| line.starts_with(HOST_PREFIX))
        .map(|host_line| host_line.trim_start_matches(HOST_PREFIX).to_string())
}

/// Act as a driver for `cargo run`/`cargo test`, but with special runner.
///
/// This function returns `Ok(true)` if all subprograms were successfully
/// executed, or `Ok(false)` if there was a non-successful subcommand.
///
/// # Errors
/// This function returns an I/O error, if a subprocess could not be spawned or
/// executed.
pub fn driver() -> io::Result<ExitStatus> {
    let cargo = env::var_os("CARGO").expect("CARGO environment variable is not set");
    let rustc = Path::new(&cargo).with_file_name("rustc");

    // Search for the host currently running to be able to override the runner.
    // The host field is extracted from `cargo version -v` if possible, since
    // this relies entirely on the used `cargo` binary. Older versions of cargo
    // don't provide the host in that output, though, so there is a fallback to
    // `rustc -vV` in that case.
    let host = search_for_host(Command::new(&cargo).arg("version").arg("-v"))
        .or_else(|| search_for_host(Command::new(rustc).arg("rustc").arg("-vV")))
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "could not determine host"))?;

    /* convert to runner env variable */
    let host = host.replace(['-', '.'], "_").to_uppercase();
    let runner = format!("CARGO_TARGET_{host}_RUNNER");

    /* cargo run with a custom runner */
    let cargo_valgrind = env::args_os()
        .next()
        .unwrap_or_else(|| OsString::from("cargo-valgrind"));

    Command::new(cargo)
        .args(env::args_os().skip(2))
        .envs(env::vars_os())
        .env(runner, cargo_valgrind)
        .spawn()?
        .wait()
}
