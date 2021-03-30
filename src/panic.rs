//! Provides the custom panic hook.

use std::panic::{self, PanicInfo};

/// Replaces any previous hook with the custom hook of this application.
///
/// This custom hook points the user to the project repository and asks them to
/// open a bug report.
pub fn replace_hook() {
    panic::set_hook(Box::new(|panic: &PanicInfo| {
        let payload = panic.payload();
        let location = panic.location();
        eprintln!(
            "Oooops. cargo valgrind unexpectedly crashed. This is a bug!\n\
            \n\
            This is an error in this program, which should be fixed. If you \
            can, please submit a bug report at\n\
            https://github.com/jfrimmel/cargo-valgrind/issues/new/choose\n\
            \n\
            To make fixing the error more easy, please provide the information \
            below as well as additional information on which project the error \
            occurred or how to reproduce it.\n\
            \n\
            Panic '{}' in {}:{}:{}",
            if let Some(s) = payload.downcast_ref::<String>() {
                s
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s
            } else {
                fn type_name<T: ?Sized>(_val: &T) -> &str {
                    std::any::type_name::<T>()
                }
                type_name(payload)
            },
            location.map_or("<unknown>", |l| l.file()),
            location.map_or_else(|| String::from("??"), |l| l.line().to_string()),
            location.map_or_else(|| String::from("??"), |l| l.column().to_string()),
        );
    }));
}
