#![allow(dead_code)]
#![allow(unused_variables)]

fn ex1() {
  let mut num = 6;

  let r1 = &num as *const i32;
  let r2 = &mut num as *mut i32;
  unsafe {
    println!("r1: {:?}", *r1);
    println!("r2: {:?}", *r2);
  }
}
fn ex2() {
  let address = 0x_012345_usize;
  let r = address as *const i32;
  unsafe {
    // error?
    println!("r={}", *r);
  }
}
fn ex3() {
  unsafe fn dangerous() {}
  unsafe { dangerous() }
}
fn ex4() {
  let mut v: Vec<_> = (1..7).collect();
  let r = &mut v[..];
  let (a, b)
      = split_at_mut(r, 4);

  println!("a={a:?} (should be [1, 2, 3, 4]");
  drop(a);
  println!("b={b:?} (should be [5, 6])");
  // assert_eq!(a, &mut[1, 2, 3, 4]);
  // assert_eq!(b, &mut[5, 6]);
}
fn split_at_mut(values: &mut[i32], ind: usize) 
-> (&mut[i32], &mut[i32]) {
  
  use std::slice::from_raw_parts_mut as part;

  assert!(ind < values.len());

  let len = values.len();
  let ptr = values.as_mut_ptr();

  unsafe {(
    part(ptr, ind), 
    // bug!
    part(ptr.add(ind + 1), len - ind)
  )}
}
extern "C" {
  fn abs(input: i32) -> i32;
}
static mut COUNTER: u32 = 0;
fn add_to_cound(inc: u32) {
  unsafe { COUNTER += inc }
}
fn ex5() {
  add_to_cound(3);
  unsafe { println!("{}", COUNTER) }
}
fn main() {
  ex4()
}
unsafe trait Doo {}
unsafe impl Doo for i32 {}