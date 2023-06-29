# memory-layout

[![crates.io](https://img.shields.io/crates/v/memory-layout.svg)](https://crates.io/memory-layout)
[![docs.rs](https://docs.rs/memory-layout/badge.svg)](https://docs.rs/memory-layout)
[![CI](https://github.com/DottieDot/memory-layout-rs/actions/workflows/CI.yml/badge.svg)](https://github.com/DottieDot/memory-layout-rs/actions/workflows/CI.yml)

`memory-layout` is a minimal `no_std` compatible crate that allows you to specify the memory layout of a struct, similarly to C#'s [`[StructLayout(LayoutKind.Explicit)]`](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.layoutkind).

## Docs
https://docs.rs/memory-layout/

## Features
* Specify the offset a field should have in a struct.
* Offsets are checked to be valid at compile time.
* `no_std` compatible.

## Example
```rust
use memory_layout::memory_layout;

#[memory_layout]
pub struct Example {
  #[field_offset(0x00)]
  a: i32,
  #[field_offset(0x10)]
  b: u64,
  #[field_offset(0x20)]
  c: f32
}
```
Will expand to:
```rust
pub struct Example {
  #[doc(hidden)]
  __pad0: [u8; 0usize],
  a:      i32,
  #[doc(hidden)]
  __pad1: [u8; 16usize - ::core::mem::size_of::<i32>()],
  b:      u64,
  #[doc(hidden)]
  __pad2: [u8; 8usize - ::core::mem::size_of::<u64>()],
  c:      f32
}
```

## Caveats
* Fields have to be defined in ascending order by the specified offset.
* `#[memory_layout]` attribute has to be defined before any `derive` attributes.

## Comparable projects
### [struct_layout](https://crates.io/crates/struct_layout)
This project has a similar goal to this crate, replicating C#'s `[StructLayout(LayoutKind.Explicit)]`. The key difference is that `struct_layout` uses an internal array that can be accessed using methods like `get_<field_name>` and `set_<field_name>` while this crate aligns the fields themselves.
