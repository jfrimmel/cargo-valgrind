use cargo_valgrind::{binaries, Build};

fn main() {
    for binary in binaries("Cargo.toml", Build::Debug).unwrap() {
        println!("{}", binary.to_str().unwrap());
    }
}
