pub use memory_layout_codegen::memory_layout;

#[cfg(test)]
mod tests {
  use core::mem::size_of;

  use crate::memory_layout;

  #[test]
  fn test_size() {
    #[memory_layout(0x38)]
    pub struct Foo {
      #[field_offset(0x10)]
      pub a: i32,

      #[field_offset(0x20)]
      pub b: i32,

      #[field_offset(0x30)]
      pub c: i32
    }

    assert_eq!(size_of::<Foo>(), 0x38, "`Foo` should be 0x38 bytes long")
  }
}
