# `cargo-valgrind`
> A cargo subcommand, that runs valgrind and collects its output in a helpful manner.

[![Latest version](https://img.shields.io/crates/v/cargo-valgrind.svg)](https://crates.io/crates/cargo-valgrind)
[![Latest GitHub release](https://img.shields.io/github/v/release/jfrimmel/cargo-valgrind)](https://github.com/jfrimmel/cargo-valgrind/releases/latest)
![License](https://img.shields.io/crates/l/cargo-valgrind)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/cargo-valgrind)](https://crates.io/crates/cargo-valgrind)


This command extends cargo with the capability to directly run `valgrind` on any crate executable.
The output of valgrind is then used to mark the binary as pass/fail.

This command should not be necessary for ordinary Rust programs, especially if you are only using safe Rust code.
But if you do FFI-related stuff (either by simply using a FFI-binding crate or because you are developing a safe wrapper for such FFI bindings) it may be really helpful to check, whether the memory usages across the FFI borders are correct.

## Usage
A typical mistake would be:
```rust
use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    fn puts(s: *const c_char);
}

fn main() {
    let string = CString::new("Test").unwrap();

    let ptr = string.into_raw();
    unsafe { puts(ptr) };

    // unsafe { CString::from_raw(ptr) };
}
```
The memory of the variable `string` will never be freed.
If you run `cargo valgrind run` in your shell, it detects the leak:
```bash
$ cargo valgrind run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/cstring`
Test
       Error leaked 5 B in 1 block
        Info stack trace (user code at the bottom)
             at malloc (vg_replace_malloc.c:446)
             at alloc (alloc.rs:100)
             at alloc_impl (alloc.rs:183)
             at allocate (alloc.rs:243)
             at try_allocate_in<u8, alloc::alloc::Global> (raw_vec.rs:230)
             at with_capacity_in<u8, alloc::alloc::Global> (raw_vec.rs:158)
             at with_capacity_in<u8, alloc::alloc::Global> (mod.rs:699)
             at with_capacity<u8> (mod.rs:481)
             at spec_new_impl_bytes (c_str.rs:290)
             at <&str as alloc::ffi::c_str::CString::new::SpecNewImpl>::spec_new_impl (c_str.rs:309)
             at alloc::ffi::c_str::CString::new (c_str.rs:319)
             at ffi_bug::main (main.rs:9)
             at core::ops::function::FnOnce::call_once (function.rs:250)
             at std::sys::backtrace::__rust_begin_short_backtrace (backtrace.rs:152)
             at std::rt::lang_start::{{closure}} (rt.rs:162)
             at call_once<(), (dyn core::ops::function::Fn<(), Output=i32> + core::marker::Sync + core::panic::unwind_safe::RefUnwindSafe)> (function.rs:284)
             at do_call<&(dyn core::ops::function::Fn<(), Output=i32> + core::marker::Sync + core::panic::unwind_safe::RefUnwindSafe), i32> (panicking.rs:557)
             at try<i32, &(dyn core::ops::function::Fn<(), Output=i32> + core::marker::Sync + core::panic::unwind_safe::RefUnwindSafe)> (panicking.rs:521)
             at catch_unwind<&(dyn core::ops::function::Fn<(), Output=i32> + core::marker::Sync + core::panic::unwind_safe::RefUnwindSafe), i32> (panic.rs:350)
             at {closure#2} (rt.rs:141)
             at do_call<std::rt::lang_start_internal::{closure_env#2}, isize> (panicking.rs:557)
             at try<isize, std::rt::lang_start_internal::{closure_env#2}> (panicking.rs:521)
             at catch_unwind<std::rt::lang_start_internal::{closure_env#2}, isize> (panic.rs:350)
             at std::rt::lang_start_internal (rt.rs:141)
             at std::rt::lang_start (rt.rs:161)
             at main
     Summary Leaked 5 B total (0 other errors)
```
Un-commenting the `unsafe { CString::from_raw(ptr) };` re-takes the memory and frees it correctly.
`cargo valgrind run` will compile the binary for you and won't detect a leak, since there is no leak anymore.

If you would like to pass flags to valgrind (for example to run an alternate subtool), you can set the `VALGRINDFLAGS` environment variable to a space-delimited list of valid Valgrind options.

_Note_: users of `cargo-valgrind` version 1.x should mind the changed command line.
Previously there was a `cargo valgrind` subcommand, that replaced the `cargo run` or `cargo test` commands.
Now the command line is `cargo valgrind <command>`, where `<command>` can be any normal cargo subcommand.

# Installation
## Requirements
You need to have `valgrind` installed and in the `PATH` (you can test this by running `valgrind --help` in your shell).

You'll also need to have `cargo` installed and in the `PATH`, but since this is a cargo subcommand, you will almost certainly have it already installed.
Note, that this tool is only supported on platforms, that have `valgrind` available.
The code is built on MacOS and Linux on x86_64, but the tests are only run under Linux.

## Install the binary
### Use pre-built binaries
Head over to the [latest release] and download the artifact for your platform.
The binary has to be extracted into the `.cargo`-directory, typically under `$HOME/.cargo`.
Note, that it is not possible to directly run the program itself, as it must be invoked via `cargo valgrind`, so it must be located in a directory, `cargo` searches its subcommands in.

[latest release]: https://github.com/jfrimmel/cargo-valgrind/releases/latest

### From Source
Run the following command to install from [crates.io](https://crates.io/crates/cargo-valgrind):
```bash
$ cargo install cargo-valgrind
```
This will install the latest official released version.

If you want to use the latest changes, that were not yet published to `crates.io`, you can install the binary from the git-repository like this:
```bash
$ cargo install --git https://github.com/jfrimmel/cargo-valgrind
```

# License
Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `cargo-valgrind` by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
