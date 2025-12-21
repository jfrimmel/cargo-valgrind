use assert_cmd::Command;

fn cargo_valgrind() -> Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("cargo-valgrind");
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
    let _delete_all_vg_core_files_on_exit = DeleteVgCoreFiles;

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

/// Issue: [#126]
///
/// [#126]: https://github.com/jfrimmel/cargo-valgrind/issues/126
#[test]
fn empty_tests_not_leak_in_release_mode() {
    const FFI_TARGET_CRATE: &[&str] = &["--manifest-path", "tests/ffi-bug/Cargo.toml"];
    cargo_valgrind()
        .arg("test")
        .arg("--release")
        .args(FFI_TARGET_CRATE)
        .assert()
        .success();
}

/// Issue: [#135] (unhelpful output when program under test aborts)
///
/// [#126]: https://github.com/jfrimmel/cargo-valgrind/issues/135
#[test]
fn program_under_test_aborts_without_leaks() {
    let _delete_all_vg_core_files_on_exit = DeleteVgCoreFiles;
    const FFI_TARGET_CRATE: &[&str] = &["--manifest-path", "tests/program-aborts/Cargo.toml"];
    cargo_valgrind()
        .arg("run")
        .args(FFI_TARGET_CRATE)
        .arg("--bin=no_leak")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "info: no memory error was detected, but the program was terminated by signal 6",
        ))
        .code(128 + 6);
}

/// Issue: [#135] (unhelpful output when program under test aborts)
///
/// [#126]: https://github.com/jfrimmel/cargo-valgrind/issues/135
#[test]
fn program_under_test_aborts_with_leaks() {
    let _delete_all_vg_core_files_on_exit = DeleteVgCoreFiles;
    const FFI_TARGET_CRATE: &[&str] = &["--manifest-path", "tests/program-aborts/Cargo.toml"];
    cargo_valgrind()
        .arg("run")
        .args(FFI_TARGET_CRATE)
        .arg("--bin=with_leak")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Summary Leaked 4 B total"))
        .stderr(predicates::str::contains(
            "info: the program was terminated by signal 6",
        ))
        .code(128 + 6);
}

/// If a program crashes within running it in Valgrind, a `vgcore.<pid>`-file
/// might be created in the current working directory. In order to not clutter
/// the main project directory, this type can be used as a drop-guard to delete
/// all `vgcore.*`-files within a test by using it like this:
/// ```
/// let _delete_all_vg_core_files_on_exit = DeleteVgCoreFiles;
/// ```
/// Note, that this deletes all found `vgcore.*`-files, not only the one created
/// by this specific test. One should add this drop-guard to each crashing test
/// nevertheless, as else the files will be left over, if only that specific
/// test is run (e.g. due to test filtering).
struct DeleteVgCoreFiles;
impl Drop for DeleteVgCoreFiles {
    fn drop(&mut self) {
        std::fs::read_dir(".")
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map_or(false, |type_| type_.is_file()))
            .filter(|file| match file.file_name().into_string() {
                Ok(name) if name.starts_with("vgcore.") => true,
                _ => false,
            })
            .for_each(|vg_core| {
                std::fs::remove_file(vg_core.path()).ok();
            });
    }
}
