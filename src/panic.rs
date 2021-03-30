//! Provides the custom panic hook.

use std::panic;

/// Replaces any previous hook with the custom hook of this application.
///
/// This custom hook points the user to the project repository and asks them to
/// open a bug report.
pub fn replace_hook() {
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        eprintln!(
            "Oooops. cargo valgrind unexpectedly crashed. This is a bug!\n\
            \n\
            This is an error in this program, which should be fixed. If you \
            can, please submit a bug report at\n\
            https://github.com/jfrimmel/cargo-valgrind/issues/new/choose\n\
            \n\
            To make fixing the error more easy, please provide the information \
            below as well as additional information on which project the error \
            occurred or how to reproduce it.\n"
        );
        old_hook(panic)
    }));
}
