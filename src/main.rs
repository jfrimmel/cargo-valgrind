//! The `cargo-valgrind` executable.
#![forbid(unsafe_code)]
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
    clippy::unwrap_used,
    clippy::cargo_common_metadata,
    clippy::used_underscore_binding
)]

mod driver;
mod panic;
mod valgrind;

use colored::Colorize as _;
use std::env;
use std::process;

/// Nicely format the errors in the valgrind output, if there are any.
fn display_errors(errors: &[valgrind::xml::Error]) {
    // format the output in a helpful manner
    for error in errors {
        if error.kind.is_leak() {
            display_leak(error);
        } else {
            display_generic_error(error);
        }
    }

    let total: usize = errors.iter().map(|error| error.resources.bytes).sum();
    eprintln!(
        "{:>12} Leaked {} total ({} other errors)",
        "Summary".red().bold(),
        bytesize::to_string(total as _, true),
        errors.iter().filter(|e| !e.kind.is_leak()).count()
    );
}

/// Nicely format a single memory leak error.
fn display_leak(error: &valgrind::xml::Error) {
    eprintln!(
        "{:>12} leaked {} in {} block{}",
        "Error".red().bold(),
        bytesize::to_string(error.resources.bytes as _, true),
        error.resources.blocks,
        if error.resources.blocks == 1 { "" } else { "s" }
    );

    let stack = &error.stack_trace[0]; // always available
    display_stack_trace("stack trace (user code at the bottom)", stack);
}

/// Nicely format a non-memory-leak error.
fn display_generic_error(error: &valgrind::xml::Error) {
    eprintln!(
        "{:>12} {}",
        "Error".red().bold(),
        error.main_info.as_ref().map_or("unknown", String::as_str)
    );

    let stack = &error.stack_trace[0]; // always available
    display_stack_trace("main stack trace (user code at the bottom)", stack);
    error
        .stack_trace
        .iter()
        .skip(1)
        .enumerate()
        .map(|(index, stack)| (error.auxiliary_info.get(index), stack))
        .for_each(|(msg, stack)| {
            display_stack_trace(
                msg.map_or_else(|| "additional stack trace", String::as_str),
                stack,
            );
        });
}

/// Write out the full stack trace (indented to match other messages).
fn display_stack_trace(msg: &str, stack: &valgrind::xml::Stack) {
    eprintln!("{:>12} {}", "Info".cyan().bold(), msg);
    stack
        .frames
        .iter()
        .for_each(|frame| eprintln!("             at {}", frame));
}

fn main() {
    panic::replace_hook();

    let number_of_arguments = || env::args_os().skip(1).count();
    let help_requested = || env::args_os().any(|arg| arg == "--help" || arg == "-h");
    let is_cargo_subcommand = || env::args_os().nth(1).map_or(false, |arg| arg == "valgrind");
    if number_of_arguments() == 0 || help_requested() {
        let text = format!(
            "cargo valgrind {version}\n\
            {authors}\n\
            Analyze your Rust binary for memory errors\n\
            \n\
            This program is a cargo subcommand, i.e. it integrates with the \
            normal cargo workflow. You specify this subcommand and another \
            \"target\", what valgrind should do. For example: `cargo valgrind \
            run` will do the same thing as `cargo run` (i.e. compile and run \
            your binary), but the execution will be done using valgrind. \
            Similarly to execute the tests, simply use `cargo valgrind test`.",
            version = env!("CARGO_PKG_VERSION"),
            authors = env!("CARGO_PKG_AUTHORS").replace(':', ", "),
        );
        #[cfg(feature = "textwrap")]
        let text = textwrap::wrap(
            textwrap::dedent(&text).trim_start(),
            textwrap::Options::with_termwidth(),
        )
        .join("\n");
        println!("{text}");
    } else if is_cargo_subcommand() {
        if !driver::driver().expect("Could not execute subcommand") {
            process::exit(200);
        }
    } else {
        // we are running as the cargo runner, therefore everything except the
        // first argument is the command to execute.
        let command = env::args_os().skip(1);

        let exit_code = match valgrind::execute(command) {
            Ok(valgrind::xml::Output {
                errors: Some(errors),
                ..
            }) => {
                display_errors(&errors);
                127
            }
            Ok(_) => 0,
            Err(e @ valgrind::Error::MalformedOutput(..)) => {
                panic_with!(e);
            }
            Err(valgrind::Error::ValgrindFailure(output))
                if output.contains("main thread stack using the --main-stacksize= flag") =>
            {
                let error = "Error".red().bold();
                let info = "Info".cyan().bold();
                eprintln!("{:>12}: looks like the program overflowed its stack", error);
                eprintln!("{:>12}: valgrind says:", info);
                output
                    .lines()
                    .for_each(|line| eprintln!("              {}", line));
                134 // default exit code for stack overflows
            }
            Err(e) => {
                eprintln!("{}: {}", "error".red().bold(), e);
                1
            }
        };
        process::exit(exit_code);
    }
}
