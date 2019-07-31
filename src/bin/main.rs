use cargo_valgrind::binaries;

fn main() {
    for binary in binaries("Cargo.toml").unwrap() {
        println!("{}", binary.to_str().unwrap());
    }
}
