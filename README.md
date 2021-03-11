# `cargo-valgrind`
> A cargo subcommand, that runs valgrind and collects its output in a helpful manner.

[![Latest version](https://img.shields.io/crates/v/cargo-valgrind.svg)](https://crates.io/crates/cargo-valgrind)
[![Documentation](https://docs.rs/cargo-valgrind/badge.svg)](https://docs.rs/cargo-valgrind)

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
If you run `cargo valgrind run` it your shell, it detects the leak:
```bash
$ cargo valgrind run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/cstring`
Test
       Error Leaked 5 bytes
        Info at realloc (vg_replace_malloc.c:826)
             at realloc (alloc.rs:125)
             at realloc (alloc.rs:184)
             at reserve_internal<u8,alloc::alloc::Global> (raw_vec.rs:666)
             at reserve_exact<u8,alloc::alloc::Global> (raw_vec.rs:411)
             at reserve_exact<u8> (vec.rs:482)
             at std::ffi::c_str::CString::from_vec_unchecked (c_str.rs:355)
             at std::ffi::c_str::CString::_new (c_str.rs:330)
             at std::ffi::c_str::CString::new (c_str.rs:324)
             at cstring::main (main.rs:9)
             at std::rt::lang_start::{{closure}} (rt.rs:64)
             at {{closure}} (rt.rs:49)
             at std::panicking::try::do_call (panicking.rs:293)
             at __rust_maybe_catch_panic (lib.rs:85)
             at try<i32,closure> (panicking.rs:272)
             at catch_unwind<closure,i32> (panic.rs:394)
             at std::rt::lang_start_internal (rt.rs:48)
             at std::rt::lang_start (rt.rs:64)
             at main
     Summary Leaked 5 B total
```
Un-commenting the `unsafe { CString::from_raw(ptr) };` re-takes the memory and frees it correctly.
`cargo valgrind run` will compile the binary for you and won't detect a leak, since there is no leak anymore.

_Note_: users of `cargo-valgrind` version 1.x should mind the changed command line.
Previously there was a `cargo valgrind` subcommand, that replaced the `cargo run` or `cargo test` commands.
Now the command line is `cargo valgrind <command>`, where `<command>` can be any normal cargo subcommand.

# Installation
## Requirements
You need to have `valgrind` installed and in the `PATH` (you can test this by running `valgrind --help` in your shell).

You'll also need to have `cargo` installed and in the `PATH`, but since this is a cargo subcommand, you will almost certainly have it already installed.

## Install the binary
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
