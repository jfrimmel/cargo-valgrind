use assert_cmd::Command;

fn cargo_valgrind() -> Command {
    let mut cmd = Command::cargo_bin("cargo-valgrind").unwrap();
    cmd.arg("valgrind");
    cmd
}

const TARGET_CRATE: &[&str] = &["--manifest-path", "tests/corpus/Cargo.toml"];

/// Issues: [#55], [#68], [#74].
///
/// [#55]: https://github.com/jfrimmel/cargo-valgrind/issues/55
/// [#68]: https://github.com/jfrimmel/cargo-valgrind/issues/68
/// [#74]: https://github.com/jfrimmel/cargo-valgrind/issues/74
#[test]
fn duplicate_stack_fields() {
    cargo_valgrind()
        .arg("run")
        .args(TARGET_CRATE)
        .arg("--bin=issue-74")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error Invalid read of size 4"))
        .stderr(predicates::str::contains(
            "Summary Leaked 0 B total (1 other errors)",
        ));
}

/// Issue: [#70]
///
/// [#70]: https://github.com/jfrimmel/cargo-valgrind/issues/70
#[test]
fn environment_variables_are_passed_to_program_under_test() {
    cargo_valgrind()
        .arg("run")
        .args(TARGET_CRATE)
        .arg("--bin=issue-70")
        .env("RUST_LOG", "debug")
        .assert()
        .stdout(predicates::str::contains("RUST_LOG=debug"));
}
