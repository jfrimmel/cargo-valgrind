fn main() {
    for (key, value) in std::env::vars_os() {
        println!("{}={}", key.to_string_lossy(), value.to_string_lossy())
    }
}
