//! Utility functions for the crate.

/// Removes a code point from the head of a utf-8 byte slice.
///
/// If the input isn't utf-8 bytes it will attempt to use the unicode
/// replacement character as appropriate.
///
/// Unfortunately, because of Rust's current `const fn` limitations, this
/// function cannot accept and return `&str` values directly.
pub const fn break_off_code_point(utf8: &[u8]) -> Option<(u32, &[u8])> {
  match utf8 {
    [a @ 0b00000000..=0b01111111, rest @ ..] => {
      // one byte
      Some((*a as u32, rest))
    }
    [a @ 0b11000000..=0b11011111, b, rest @ ..] => {
      // two bytes
      let lead = (*a & 0b11111) as u32;
      let trail = (*b & 0b111111) as u32;
      Some((lead << 6 | trail, rest))
    }
    [a @ 0b11100000..=0b11101111, b, c, rest @ ..] => {
      // three bytes
      let lead = (*a & 0b1111) as u32;
      let trail1 = (*b & 0b111111) as u32;
      let trail2 = (*c & 0b111111) as u32;
      let out = lead << 12 | trail1 << 6 | trail2;
      Some((out, rest))
    }
    [a @ 0b11110000..=0b11110111, b, c, d, rest @ ..] => {
      // four bytes
      let lead = (*a & 0b111) as u32;
      let trail1 = (*b & 0b111111) as u32;
      let trail2 = (*c & 0b111111) as u32;
      let trail3 = (*d & 0b111111) as u32;
      let out = lead << 18 | trail1 << 12 | trail2 << 6 | trail3;
      Some((out, rest))
    }
    [] => None,
    [_unknown, rest @ ..] => {
      // If we can't match anything above, we just pull off one byte and give
      // the unicode replacement code point.
      Some(('�' as u32, rest))
    }
  }
}

#[test]
fn test_break_off_code_point() {
  // code points of 1, 2, 3, and 4 byte size
  for ch in &['$', '¢', 'ह', '€', '한', '𐍈'] {
    let s = format!("{}", ch);
    assert_eq!(break_off_code_point(s.as_bytes()), Some((*ch as u32, &[][..])));
  }

  // empty string works properly
  assert!(break_off_code_point("".as_bytes()).is_none());
}

/// Counts the number of code units that the input would require in UTF-16.
///
/// This is **not** the number of code *points*, this is the number of code
/// *units*. Just like with utf-8, utf-16 is variable width. Not all code points
/// are just one `u16`, some are two. This counts the number of `u16` values
/// you'd have if you re-encoded the input as utf-16.
pub const fn count_utf16_code_units(s: &str) -> usize {
  let mut bytes = s.as_bytes();
  let mut len = 0;
  while let Some((u, rest)) = break_off_code_point(bytes) {
    len += if u <= 0xFFFF { 1 } else { 2 };
    bytes = rest;
  }
  len
}

#[test]
fn test_count_utf16_code_units() {
  let s = "hello from the unit test";
  let normal_style: usize = s.chars().map(|ch| ch.len_utf16()).sum();
  assert_eq!(normal_style, count_utf16_code_units(s));

  let s = "$¢ह€한𐍈, 漢字, ひらがな / 平仮名, カタカナ / 片仮名";
  let normal_style: usize = s.chars().map(|ch| ch.len_utf16()).sum();
  assert_eq!(normal_style, count_utf16_code_units(s));
}
