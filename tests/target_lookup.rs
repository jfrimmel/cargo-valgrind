use cargo_valgrind::{Build, Target};

#[test]
fn single_binary_debug() {
    assert_eq!(
        cargo_valgrind::targets("tests/binary/Cargo.toml", Build::Debug).unwrap(),
        vec![Target::Binary("tests/binary/target/debug/binary".into())]
    );
}

#[test]
fn single_binary_release() {
    assert_eq!(
        cargo_valgrind::targets("tests/binary/Cargo.toml", Build::Release).unwrap(),
        vec![Target::Binary("tests/binary/target/release/binary".into())]
    );
}
