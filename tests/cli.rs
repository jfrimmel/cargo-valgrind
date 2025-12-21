use assert_cmd::Command;

fn cargo_valgrind() -> Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("cargo-valgrind");
    cmd.arg("valgrind");
    cmd
}

const TARGET_CRATE: &[&str] = &["--manifest-path", "tests/ffi-bug/Cargo.toml"];

#[test]
fn leak_detected() {
    cargo_valgrind()
        .arg("run")
        .args(TARGET_CRATE)
        .assert()
        .failure();
}

#[test]
fn examples_are_runnable() {
    cargo_valgrind()
        .args(["run", "--example", "no-leak"])
        .args(TARGET_CRATE)
        .assert()
        .success()
        .stdout("Hello, world!\n");
}

#[test]
fn tests_are_runnable() {
    cargo_valgrind()
        .arg("test")
        .args(TARGET_CRATE)
        .assert()
        .success();
}

#[test]
fn help_is_supported() {
    cargo_valgrind()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("cargo valgrind"));
}
