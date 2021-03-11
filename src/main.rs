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

use colored::Colorize as _;
use std::env;
use std::ffi::OsString;
use std::net::{SocketAddr, TcpListener};
use std::process::{self, Command};

fn main() {
    if env::args_os().nth(1) == Some(OsString::from("valgrind")) {
        if !driver::driver().expect("Could not execute subcommand") {
            process::exit(200);
        }
    } else {
        // we are running as the cargo runner, therefore everything except the
        // first argument is the command to execute.
        let command = env::args_os().skip(1);

        // port selected by OS
        let address: SocketAddr = ([127, 0, 0, 1], 0).into();
        let listener = TcpListener::bind(address).unwrap();
        let address = listener.local_addr().unwrap();

        let mut cargo = Command::new("valgrind")
            .arg("--xml=yes")
            .arg(format!("--xml-socket={}:{}", address.ip(), address.port()))
            .args(command)
            .spawn()
            .expect("Valgrind is not installed or cannot be started");

        // collect the output of valgrind
        let (listener, _socket) = listener.accept().unwrap();
        let xml: valgrind_xml::Output =
            serde_xml_rs::from_reader(listener).expect("Cannot parse valgrind output");
        let success = cargo.wait().unwrap().success();
        if !success {
            process::exit(100);
        } else if let Some(errors) = xml.errors {
            // format the output in a helpful manner
            for error in &errors {
                eprintln!(
                    "{:>12} leaked {} in {} block{}",
                    "Error".red().bold(),
                    bytesize::to_string(error.resources.bytes as _, true),
                    error.resources.blocks,
                    if error.resources.blocks == 1 { "" } else { "s" }
                );
                let mut info = Some("Info".cyan().bold());
                error.stack_trace.frames.iter().for_each(|frame| {
                    eprintln!("{:>12} at {}", info.take().unwrap_or_default(), frame)
                });
            }

            let total: usize = errors.iter().map(|error| error.resources.bytes).sum();
            eprintln!(
                "{:>12} Leaked {} total",
                "Summary".red().bold(),
                bytesize::to_string(total as _, true)
            );

            process::exit(127);
        }

        // TODO: use drop guard, that waits on child in order to prevent printing to stdout of the child
    }
}
