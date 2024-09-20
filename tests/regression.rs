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

/// Issue: [#36]: make sure, that an interrupted `cargo valgrind` invocation
/// kills the running program, so that it does not run in the background.
///
/// [#36]: https://github.com/jfrimmel/cargo-valgrind/issues/36
#[test]
fn interrupted_program_execution() {
    use assert_cmd::cargo::CommandCargoExt;
    use std::{io::Read, process, thread, time::Duration};

    // pre-build the crate to run, so that it does not need to be built later on
    cargo_valgrind()
        .arg("build")
        .args(TARGET_CRATE)
        .arg("--bin=issue-36");

    // We need the raw `std::process::Command` in order to send the kill signal
    // to it. Therefore this does not use the `cargo_valgrind()` helper as all
    // other tests.
    let mut cargo_valgrind = process::Command::cargo_bin("cargo-valgrind")
        .unwrap()
        .arg("valgrind")
        .arg("run")
        .arg("-q") // silence cargo output
        .args(TARGET_CRATE)
        .arg("--bin=issue-36")
        .stderr(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap();

    // wait until program is certainly started
    thread::sleep(Duration::from_millis(500));

    // kill `cargo valgrind`, which should kill the run program as well.
    cargo_valgrind.kill().unwrap();
    cargo_valgrind.wait().unwrap();

    // Check, what the helper program printed. The run program prints one line
    // every second. Since this test should have killed the program before the
    // first second is elapsed, there should be no output.
    let mut stdout = String::new();
    cargo_valgrind
        .stdout
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    assert_eq!("", stdout, "Program must end before the first print");
}
