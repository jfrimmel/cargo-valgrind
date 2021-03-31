//! Provides the custom panic hook.

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

        old_hook(panic)
    }));
}
