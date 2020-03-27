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

mod driver;
mod valgrind_xml;

use std::env;
use std::ffi::OsString;
use std::process;

fn main() {
    if env::args_os().nth(1) == Some(OsString::from("valgrind")) {
        if !driver::driver().expect("Could not execute subcommand") {
            process::exit(1);
        }
    } else {
        let x: Vec<_> = env::args().skip(1).collect();
        println!("valgrind [valgrind options] {}", x.join(" "));
    }
}
