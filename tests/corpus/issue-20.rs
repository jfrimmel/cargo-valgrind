extern "C" {
    fn malloc(n: usize) -> *mut std::ffi::c_void;
    fn free(ptr: *mut std::ffi::c_void);
}

fn main() {
    let ptr = unsafe { malloc(8) };
    unsafe { free(ptr) };
    unsafe { free(ptr) };
}
