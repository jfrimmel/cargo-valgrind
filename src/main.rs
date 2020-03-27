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
use std::io::Read;
use std::net::{SocketAddr, TcpListener};
use std::process;

fn main() {
    if env::args_os().nth(1) == Some(OsString::from("valgrind")) {
        if !driver::driver().expect("Could not execute subcommand") {
            process::exit(200);
        }
    } else {
        // port selected by OS
        let address: SocketAddr = ([127, 0, 0, 1], 0).into();
        let listener = TcpListener::bind(address).unwrap();
        let address = listener.local_addr().unwrap();

        let mut cargo = std::process::Command::new("valgrind")
            .arg("--xml=yes")
            .arg(format!("--xml-socket={}:{}", address.ip(), address.port()))
            .args(env::args_os().skip(1))
            .spawn()
            .unwrap();

        let (mut listener, _socket) = listener.accept().unwrap();

        let success = cargo.wait().unwrap().success();
        let mut xml = String::new();
        listener.read_to_string(&mut xml).unwrap();
        println!("{}", xml);
        if !success {
            process::exit(100);
        }
    }
}
