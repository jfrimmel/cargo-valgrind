//! Provides the panic handling neceises.
//!
//! This module allows to customize the panicking of the application. First, it
//! changes the normal panic handler to print a custom message, that guides the
//! user to the bug tracker. Secondly, the panic message presented will change
//! depending on the payload-type. If that panic is an implementation bug, some
//! addition information is printed (e.g. for [`Error::MalformedOutput`]).
//!
//! [`Error::MalformedOutput`]: crate::valgrind::Error::MalformedOutput

use std::panic;

const PANIC_HEADER: &str = "
    Oooops. cargo valgrind unexpectedly crashed. This is a bug!

    This is an error in this program, which should be fixed. If you can, please
    submit a bug report at

        https://github.com/jfrimmel/cargo-valgrind/issues/new/choose

    To make fixing the error more easy, please provide the information below as
    well as additional information on which project the error occurred or how
    to reproduce it.
    ";

/// Panic with a custom panic output.
///
/// This is helpful for printing debug information to the panic message.
#[macro_export]
macro_rules! panic_with {
    ($e:expr) => {
        std::panic::panic_any($e)
    };
}

/// Replaces any previous hook with the custom hook of this application.
///
/// This custom hook points the user to the project repository and asks them to
/// open a bug report.
pub fn replace_hook() {
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        #[cfg(not(feature = "textwrap"))]
        let text = PANIC_HEADER;
        #[cfg(feature = "textwrap")]
        let text = textwrap::wrap(
            &textwrap::dedent(PANIC_HEADER).trim_start(),
            textwrap::Options::with_termwidth(),
        )
        .join("\n");
        eprintln!("{}", text);

        if let Some(crate::valgrind::Error::MalformedOutput(e)) = panic.payload().downcast_ref() {
            eprintln!(
                "XML format mismatch between `valgrind` and `cargo valgrind`: {}",
                e
            );
        } else {
            old_hook(panic)
        }
    }));
}
