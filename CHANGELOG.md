# Changelog

## Unreleased
- Better error message if valgrind is not found
- support multiple feature flags, similar to normal `cargo`
- support comma separation of features, similar to normal `cargo`

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
