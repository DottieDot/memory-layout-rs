pub use memory_layout_derive::*;

#[memory_layout]
/// Docs!!!
pub struct Yeet {
  #[field_offset(0x10)]
  pub a: i32,

  #[field_offset(0x20)]
  pub b: i32,

  #[field_offset(0x30)]
  #[field_offset(0x40)]
  /// Docs!!!
  pub c: i32
}

fn main() {
  println!("{}", std::mem::size_of::<Yeet>());
}
