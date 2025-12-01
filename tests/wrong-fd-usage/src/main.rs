fn write_to_closed_fd() {
    unsafe { libc::close(2) };

    unsafe { libc::write(2, b"hello, world!".as_ptr().cast(), 13) };
}

fn main() {
    write_to_closed_fd();
}
