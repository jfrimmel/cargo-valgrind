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

fn main() {
    if !driver::driver().expect("Could not execute subcommand") {
        std::process::exit(1);
    }
}
