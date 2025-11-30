fn leak_some_memory_early() {
    let leaked_memory = Box::new(1234);
    Box::leak(leaked_memory);
}

fn main() {
    leak_some_memory_early();

    std::process::abort();
}
