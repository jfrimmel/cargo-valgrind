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
        .success();
}
