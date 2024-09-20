fn main() {
    for n in 1..=10 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("waited {n} seconds");
    }
}
