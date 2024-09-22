//! Write human-readable and colored output the the console.
use crate::valgrind;
use colored::Colorize as _;

/// Nicely format the errors in the valgrind output, if there are any.
pub fn display_errors(errors: &[valgrind::xml::Error]) {
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
        .for_each(|frame| eprintln!("             at {frame}"));
}
