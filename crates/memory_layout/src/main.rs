
pub use memory_layout_derive::*;

pub trait MemoryLayout {}

#[derive(MemoryLayout)]
pub struct Yeet {
  #[field_offset(0x00)]
  pub a: i32,

  #[field_offset(0x08)]
  pub b: i32,

  #[field_offset(0x10)]
  pub c: i32
}

fn main() {
  println!("{}", std::mem::size_of::<Yeet>());
  println!("{}", std::mem::size_of::<Test>());
}
