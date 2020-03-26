//! The `cargo-valgrind` executable.
#![deny(clippy::correctness)]
#![warn(
    clippy::perf,
    clippy::complexity,
    clippy::style,
    clippy::nursery,
    clippy::pedantic,
    clippy::clone_on_ref_ptr,
    clippy::decimal_literal_representation,
    clippy::float_cmp_const,
    clippy::missing_docs_in_private_items,
    clippy::multiple_inherent_impl,
    clippy::option_unwrap_used,
    clippy::print_stdout,
    clippy::result_unwrap_used
)]

mod valgrind_xml;

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    /* get path of `rustc` */
    let cargo = env::var_os("CARGO").expect("CARGO environment variable is not set");
    let rustc = Path::new(&cargo).with_file_name("rustc");

    /* get the output of `rustc -vV` */
    let rustc_info = Command::new(rustc).arg("-vV").output().unwrap().stdout;

    /* get the host information (all after the "host: ..." line) */
    let host = rustc_info
        .windows(5)
        .position(|window| window == b"host:")
        .expect("Host information not present in `rustc -vV`");
    let host: String = rustc_info
        .into_iter()
        .skip(host)
        .skip(b"host: ".len())
        .take_while(|&x| x != b'\n')
        .map(char::from)
        .collect();

    /* convert to runner env variable */
    let runner = format!(
        "CARGO_TARGET_{}_RUNNER",
        host.replace('-', "_").replace('.', "_").to_uppercase()
    );

    /* cargo run with a custom runner */
    Command::new(cargo)
        .args(env::args_os().skip(2))
        .envs(env::vars_os())
        .env(runner, "valgrind") // TODO: call the own binary with a special parameter
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
