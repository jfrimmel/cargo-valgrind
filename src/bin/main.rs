use cargo_valgrind::{binaries, Build};

fn main() {
    match binaries("Cargo.toml", Build::Debug) {
        Ok(binaries) => {
            for binary in binaries {
                println!("{}", binary.to_str().unwrap());
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
