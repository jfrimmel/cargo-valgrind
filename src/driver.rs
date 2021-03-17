//! A module providing the wrapping driver for a custom runner.

use std::env;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::Command;

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
    /* get path of `rustc` */
    let cargo = env::var_os("CARGO").expect("CARGO environment variable is not set");
    let rustc = Path::new(&cargo).with_file_name("rustc");

    /* get the output of `rustc -vV` */
    let rustc_info = Command::new(rustc).arg("-vV").output()?.stdout;

    /* get the host information (all after the "host: ..." line) */
    let host = rustc_info
        .windows(HOST_PREFIX.len())
        .position(|window| window == HOST_PREFIX)
        .expect("Host information not present in `rustc -vV`");
    let host: String = rustc_info
        .into_iter()
        .skip(host)
        .skip(HOST_PREFIX.len())
        .take_while(|&x| x != b'\n')
        .map(char::from)
        .collect();

    /* convert to runner env variable */
    let host = host.replace('-', "_").replace('.', "_").to_uppercase();
    let runner = format!("CARGO_TARGET_{}_RUNNER", host);

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
