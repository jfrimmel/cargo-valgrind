# Changelog

## Unreleased
- Check, if valgrind is installed and print a helpful error if not
- Handle other valgrind errors more gracefully
- Add a custom panic hook, that points the user to the bug-tracker
- Format help and panic messages depending on the terminal size

## Version 2.0.0
Breaking API and CLI change!
- Support running _every_ `cargo` executable (binary, unit tests, doctests, ...)
- changed command line
    - `cargo valgrind` -> `cargo valgrind run`
    - `cargo valgrind --tests` -> `cargo valgrind test`
    - `cargo valgrind --example asdf` -> `cargo valgrind run --example asdf`
    - etc.
- currently no valgrind parameter support

## Version 1.3.0
- Support user flags for the analyzed binary.

## Version 1.2.3
- Updated dependencies

## Version 1.2.2
- Better error message if valgrind is not found
- support multiple feature flags, similar to normal `cargo`
- support comma separation of features, similar to normal `cargo`
- Bugfix: replace `-` by `_` in integration test target names

## Version 1.2.1
- Support running of integration tests (normal tests are not yet supported)
- Fixed panic if the crate under test contains a build script
- Print an error if there are no runnable targets available

## Version 1.2.0
- Support the valgrind parameter `--show-leak-kinds=<set>`
- Support the valgrind parameter `--leak-check=<summary|full>`

## Version 1.1.2
- Manually implement `Hash` for `Target`.
  This was previously derived, which was wrong due to the custom `PartialEq`-implementation (refer to the [`Hash` documentation](https://doc.rust-lang.org/std/hash/trait.Hash.html#hash-and-eq)).

## Version 1.1.1
- Print the total number of leaked bytes as a summary

## Version 1.1.0
- Added `--features` flag.
  This flag is the `cargo valgrind` analog to the same flag on other `cargo` subcommands.
- deprecated `cargo_valgrind::build_target()` in favor of the more flexible `cargo_valgrind::Cargo` type.

## Version 1.0.0
- Initial release
