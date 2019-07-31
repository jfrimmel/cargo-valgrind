# `cargo-valgrind`
> A cargo subcommand, that runs valgrind and collects its output in a helpful manner.

This command extends cargo with the capability to directly run `valgrind` on the executable (either a bin-crate or an example).
The output of valgrind is then used to mark the binary as pass/fail.

This command should not be necessary for ordinary Rust programs, especially if you are only using safe Rust code.
But if you do FFI-related stuff (either by simply using a FFI-binding crate or because you are developing a safe wrapper for such FFI bindings) it may be really helpful to check, whether the memory usages across the FFI borders are correct.

A typical mistake would be:
```rust
use std::ffi::CString;
use std::os::raw::c_char;

// A sample FFI function
fn ffi_function(string: *const c_char) { /* ... */ }

fn main() {
    let string = CString::new("Test").unwrap();
    ffi_function(string.into_raw());
}
```
The memory of the variable `string` will never be freed.
`cargo valgrind` detects it:
```bash
$ cargo valgrind
TODO
```

# Installation
## Requirements
You need to have `valgrind` installed and in the `PATH` (you can test this by running `valgrind --help` in your shell).

## Install the binary
Run the following command:
```
cargo install --git https://github.com/jfrimmel/cargo-valgrind
```
