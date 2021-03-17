use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    fn puts(s: *const c_char);
}

fn main() {
    let string = CString::new("Test").unwrap();

    let ptr = string.into_raw();
    unsafe { puts(ptr) };

    // unsafe { CString::from_raw(ptr) };
}

#[cfg(test)]
mod tests {
    #[test]
    fn t1() {}
    #[test]
    fn t2() {}
    #[test]
    fn t3() {}
    #[test]
    fn t4() {}
    #[test]
    fn t5() {}
}
