//! Library for the [Triangle From Scratch][tfs] project.
//!
//! [tfs]: https://rust-tutorials.github.io/triangle-from-scratch/

mod macros;

pub mod util;

#[cfg(windows)]
pub mod win32;
#[cfg(windows)]
use win32::*;

pub mod gl;

pub mod vk;

/// Gathers up the bytes from a pointer.
///
/// The byte sequence must be valid and null-terminated.
///
/// The output excludes the null byte.
pub unsafe fn gather_null_terminated_bytes(mut p: *const u8) -> Vec<u8> {
  let mut v = vec![];
  while *p != 0 {
    v.push(*p);
    p = p.add(1);
  }
  v
}

/// Converts a `Vec<u8>` into a `String` using the minimum amount of
/// re-allocation.
pub fn min_alloc_lossy_into_string(bytes: Vec<u8>) -> String {
  match String::from_utf8(bytes) {
    Ok(s) => s,
    Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
  }
}

/// Makes a `&str` from all bytes before the first `0` within the byte slice.
///
/// If the slice has no `0` then the entire slice is used.
pub fn str_from_null_terminated_byte_slice(
  bytes: &[u8],
) -> Result<&str, core::str::Utf8Error> {
  let terminal_position =
    bytes.iter().copied().position(|u| u == 0).unwrap_or(bytes.len());
  core::str::from_utf8(&bytes[..terminal_position])
}
