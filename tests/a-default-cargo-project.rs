//! Tests against an almost empty crate generated by `cargo new --lib`.
use assert_cmd::Command;

fn cargo_valgrind() -> Command {
    let mut cmd = Command::cargo_bin("cargo-valgrind").unwrap();
    cmd.arg("valgrind");
    cmd
}

/// Test, that running `cargo valgrind test` does not result in any reported
/// leaks. This is the "minimal" crate (in the sense, that people might start
/// with such a minimal crate to experiment with `cargo valgrind`), where
/// `cargo valgrind` must never report any issues. If that test fails, then
/// there is a leak in the Rust standard library, which is reported to be
/// [allowed][leak-issue]:
///
/// > Based on the consensus in rust-lang/rust#133574, non-default full
/// > leakcheck is "convenience if we can, but no hard guarantees", [...].
///
/// [leak-issue]: https://github.com/rust-lang/rust/issues/135608#issuecomment-2597205627
#[test]
fn default_cargo_project_reports_no_violations() {
    cargo_valgrind()
        .arg("test")
        .args(["--manifest-path", "tests/default-new-project/Cargo.toml"])
        .assert()
        .success();
}
