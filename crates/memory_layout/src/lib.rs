pub use memory_layout_proc::memory_layout;

#[cfg(test)]
mod tests {
  use crate::memory_layout;

  #[test]
  fn test_size() {
    #[memory_layout]
    pub struct Foo {
      #[field_offset(0x10)]
      pub a: i32,

      #[field_offset(0x20)]
      pub b: i32,

      #[field_offset(0x30)]
      pub c: i32
    }

    assert_eq!(
      std::mem::size_of::<Foo>(),
      0x34,
      "`Foo` should be 0x34 bytes long"
    )
  }
}
