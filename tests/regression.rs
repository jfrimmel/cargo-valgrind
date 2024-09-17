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

/// Issue: [#13]
///
/// [#13]: https://github.com/jfrimmel/cargo-valgrind/issues/13
#[test]
fn stack_overflow_in_program_under_test() {
    cargo_valgrind()
        .arg("run")
        .args(TARGET_CRATE)
        .arg("--bin=issue-13")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "looks like the program overflowed its stack",
        ));
}

/// Issue: [#20]
///
/// [#20]: https://github.com/jfrimmel/cargo-valgrind/issues/20
#[test]
fn invalid_free() {
    cargo_valgrind()
        .arg("run")
        .args(TARGET_CRATE)
        .arg("--bin=issue-20")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error Invalid free"))
        .stderr(predicates::str::contains(
            "is 0 bytes inside a block of size 8 free'd",
        ))
        .stderr(predicates::str::contains("Info Block was alloc'd at"));
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
