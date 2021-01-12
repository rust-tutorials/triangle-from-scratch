//! Module for the library's macros.
//!
//! Due to some rust weirdness, it's better to have macros in their own module.

/// Changes a rust &str expression into an array literal of utf-16 data.
///
/// The output is a sized array literal.
/// If you want to use the output as a `&[u16]` just prefix the call with `&`.
#[macro_export]
macro_rules! utf16 {
  ($text:expr) => {{
    // Here we pick a name highly unlikely to exist in the scope
    // that $text came from, which prevents a potential const eval cycle error.
    const __A1B2C3D4_CONST_EVAL_LOOP_BREAK: &str = $text;
    const UTF8: &str = __A1B2C3D4_CONST_EVAL_LOOP_BREAK;
    const OUT_BUFFER_LEN: usize = $crate::util::count_utf16_code_units(UTF8);
    const UTF16: [u16; OUT_BUFFER_LEN] = {
      let mut buffer = [0u16; OUT_BUFFER_LEN];
      let mut bytes = UTF8.as_bytes();
      let mut i = 0;
      while let Some((u, rest)) = $crate::util::break_off_code_point(bytes) {
        if u <= 0xFFFF {
          buffer[i] = u as u16;
          i += 1;
        } else {
          let code = u - 0x1_0000;
          buffer[i] = 0xD800 | ((code >> 10) as u16);
          buffer[i + 1] = 0xDC00 | ((code & 0x3FF) as u16);
          i += 2;
        }
        bytes = rest;
      }
      buffer
    };
    UTF16
  }};
}

#[test]
fn test_utf16() {
  const HELLO16: [u16; 5] = utf16!("hello");
  assert_eq!(&HELLO16[..], &"hello".encode_utf16().collect::<Vec<u16>>());

  const WORDS8: &str = "$¢ह€한𐍈, 漢字, ひらがな / 平仮名, カタカナ / 片仮名";
  const WORDS16: &[u16] = &utf16!(WORDS8);
  assert_eq!(WORDS16, &WORDS8.encode_utf16().collect::<Vec<u16>>());
}

/// As per [`utf16`], but places a null-terminator on the end.
#[macro_export]
macro_rules! utf16_null {
  ($text:expr) => {{
    const TEXT_NULL___A1B2C3D4: &str = concat!($text, '\0');
    $crate::utf16!(TEXT_NULL___A1B2C3D4)
  }};
}

#[test]
fn test_utf16_null() {
  const HELLO: &[u16] = &utf16_null!("hello");
  assert_eq!(HELLO, &"hello\0".encode_utf16().collect::<Vec<u16>>());
}

/// Turns a rust string literal into a null-terminated `&[u8]`.
#[macro_export]
macro_rules! c_str {
  ($text:expr) => {{
    concat!($text, '\0').as_bytes()
  }};
}
