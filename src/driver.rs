//! A module providing the wrapping driver for a custom runner.

use std::env;
use std::ffi::OsString;
use std::io;
use std::process::Command;

/// The prefix line for the target host output.
const HOST_PREFIX: &[u8] = b"host: ";

/// Act as a driver for `cargo run`/`cargo test`, but with special runner.
///
/// This function returns `Ok(true)` if all subprograms were successfully
/// executed, or `Ok(false)` if there was a non-successful subcommand.
///
/// # Errors
/// This function returns an I/O error, if a subprocess could not be spawned or
/// executed.
pub fn driver() -> io::Result<bool> {
    let cargo = env::var_os("CARGO").expect("CARGO environment variable is not set");

    /* get the output of `cargo version -v` */
    let rustc_info = Command::new(&cargo).args(["version", "-v"]).output()?.stdout;

    /* get the host information (all after the "host: ..." line) */
    let host = rustc_info
        .windows(HOST_PREFIX.len())
        .position(|window| window == HOST_PREFIX)
        .expect("Host information not present in `cargo version -v`");
    let host: String = rustc_info
        .into_iter()
        .skip(host)
        .skip(HOST_PREFIX.len())
        .take_while(|&x| x != b'\n')
        .map(char::from)
        .collect();

    /* convert to runner env variable */
    let host = host.replace(['-', '.'], "_").to_uppercase();
    let runner = format!("CARGO_TARGET_{host}_RUNNER");

    /* cargo run with a custom runner */
    let cargo_valgrind = env::args_os()
        .next()
        .unwrap_or_else(|| OsString::from("cargo-valgrind"));

    Ok(Command::new(cargo)
        .args(env::args_os().skip(2))
        .envs(env::vars_os())
        .env(runner, cargo_valgrind)
        .spawn()?
        .wait()?
        .success())
}
