use assert_cmd::Command;

fn cargo_valgrind() -> Command {
    let mut cmd = Command::cargo_bin("cargo-valgrind").unwrap();
    cmd.arg("valgrind");
    cmd
}

const TARGET_CRATE: &[&str] = &["--manifest-path", "tests/corpus/Cargo.toml"];

#[test]
fn issue74() {
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

#[test]
fn issue70() {
    cargo_valgrind()
        .arg("run")
        .args(TARGET_CRATE)
        .arg("--bin=issue-70")
        .env("RUST_LOG", "debug")
        .assert()
        .stdout(predicates::str::contains("RUST_LOG=debug"));
}
