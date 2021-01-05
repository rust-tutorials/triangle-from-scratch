
(Note: Thanks to [Plecra](https://github.com/Plecra) for helping with the code for this article.)

# UTF-16 Literals

Rust literals are textual content encoded as UTF-8.

Sometimes we want our textual content in other encodings instead.

This can be done at runtime,
but why do at runtime what you could be doing at compile time?

## What is Unicode?

Unicode is a huge pile of nonsense, that's what it is.

There's an old article from 2003 called
[The Absolute Minimum Every Software Developer Absolutely, Positively Must Know About Unicode and Character Sets (No Excuses!)](https://www.joelonsoftware.com/2003/10/08/the-absolute-minimum-every-software-developer-absolutely-positively-must-know-about-unicode-and-character-sets-no-excuses/),
and the title of that article is correct.

You should stop and read it if you haven't,
and from here on I'm going to assume you've read it.

## `const fn`

In Rust, if you want to do something at compile time you must use a `const fn`.
However, Rust `const fn` is only *partly* implemented within the language.
There is much that is not yet done,
and most importantly `const fn` doesn't support traits.

Not having trait support means that we have to do things a little weird.
For one, we can't use normal iterators.
Another, we can't use string slice indexing.

So we'll have to write some `const fn` stuff without using traits.

## `break_off_code_point`

First, we want a `const fn` way to break a code point off the front of a utf-8 byte slice.
Normally, we'd use [str::chars](https://doc.rust-lang.org/std/primitive.str.html#method.chars),
but remember that there's no iterators here.
We'll have to just do it ourselves.

It's not too difficult.
We just follow the spec, as described on the [the wikipedia for utf-8](https://en.wikipedia.org/wiki/UTF-8#Encoding).

First we need a function that will take in some utf8 bytes,
break off one code point worth of bytes,
and then return what code point it found and the rest of the bytes.
All of this might fail (such as if the input is an empty slice), so the output is an `Option`

```rust
pub const fn break_off_code_point(utf8: &[u8]) -> Option<(u32, &[u8])> {
  None
}
```

As I mentioned above, we can't sub-slice a `&str` value in a const context in current rust,
so we'll have to deal directly in `&[u8]` for now.

### Determine the next code point's byte count

The first thing we do is decide how many bytes we're going to use from the input.

As a "default" case, if we can't find a code point in the input we'll give none.

```rust
pub const fn break_off_code_point(utf8: &[u8]) -> Option<(u32, &[u8])> {
  match utf8 {
    [..] => None
  }
}
```

To determine how many bytes we'll use up to get our code point we look at the bits of the leading byte.
For any multi-byte sequence the number of initial 1 bits is the number of *total* bytes in the sequence.

* If the initial bit is 0, then that's a one byte sequence.
* If the initial bits are 110, then that's a two byte sequence.
* If the initial bits are 1110, then that's a three byte sequence.
* If the initial bits are 11110, then that's a four byte sequence.

We can look at the initial byte and see what case we're in using a
[slice pattern](https://doc.rust-lang.org/reference/patterns.html#slice-patterns).
```rust
pub const fn break_off_code_point(utf8: &[u8]) -> Option<(u32, &[u8])> {
  match utf8 {
    [0b00000000..=0b01111111, ..] => None, /* one */
    [0b11000000..=0b11011111, ..] => None, /* two */
    [0b11100000..=0b11101111, ..] => None, /* three */
    [0b11110000..=0b11110111, ..] => None, /* four */
    [..] => None,                          /* default */
  }
}
```

Except that after checking the bit pattern we need to store that byte,
because we'll need it to determine the output.
Also we'll need to use the "rest of the bytes" too.
For this we can add an [identifier pattern](https://doc.rust-lang.org/reference/patterns.html#identifier-patterns)
to the matching segments.
```rust
pub const fn break_off_code_point(utf8: &[u8]) -> Option<(u32, &[u8])> {
  match utf8 {
    [a @ 0b00000000..=0b01111111, rest @ ..] => None, /* one */
    [a @ 0b11000000..=0b11011111, rest @ ..] => None, /* two */
    [a @ 0b11100000..=0b11101111, rest @ ..] => None, /* three */
    [a @ 0b11110000..=0b11110111, rest @ ..] => None, /* four */
    [..] => None,                                     /* default */
  }
}
```

Slice patterns and identifier patterns are not very common in Rust,
so if you're unfamiliar with them please go glance at the reference.

Okay, next adjustment is that the two, three, and four cases aren't actually pulling the right number of bytes off the slice.

```rust
pub const fn break_off_code_point(utf8: &[u8]) -> Option<(u32, &[u8])> {
  match utf8 {
    [a @ 0b00000000..=0b01111111, rest @ ..] => None, /* one */
    [a @ 0b11000000..=0b11011111, b, rest @ ..] => None, /* two */
    [a @ 0b11100000..=0b11101111, b, c, rest @ ..] => None, /* three */
    [a @ 0b11110000..=0b11110111, b, c, d, rest @ ..] => None, /* four */
    [..] => None,                                     /* default */
  }
}
```

What's up with those comments? That's really how rustfmt puts it. Whatever.

Also note that *technically* the trailing bytes have their own limits on what's valid and what's not.
We're going to take a page from Rust's book and ignore that.
We'll simply *assume* that the trailing bytes in a multi-byte sequence are valid.
If a caller gives us bad input, we might give them bad output back.
There's not an actual safety concern with it, so it's not a big deal.

### Compute the output code point

So now we need to fill in the output side of the four cases.

The one byte case is simple.
We just return the value directly.

```rust
pub const fn break_off_code_point(utf8: &[u8]) -> Option<(u32, &[u8])> {
  match utf8 {
    [a @ 0b00000000..=0b01111111, rest @ ..] => {
      // one byte
      Some((*a as u32, rest))
    }
    [a @ 0b11000000..=0b11011111, b, rest @ ..] => None, /* two */
    [a @ 0b11100000..=0b11101111, b, c, rest @ ..] => None, /* three */
    [a @ 0b11110000..=0b11110111, b, c, d, rest @ ..] => None, /* four */
    [..] => None,                                        /* default */
  }
}
```

The two byte case is where we start having to combine bits across different bytes.
From the leading byte we take the lowest 5 bits,
and from the trailing byte we take the lowest 6 bits.

```rust
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
    [a @ 0b11100000..=0b11101111, b, c, rest @ ..] => None, /* three */
    [a @ 0b11110000..=0b11110111, b, c, d, rest @ ..] => None, /* four */
    [..] => None,                                           /* default */
  }
}
```

The three and four byte cases are the same idea as the two byte case.
The number of bits to use from the leading byte changes each time,
but the number of bits to use from trailing bytes stays the same.

```rust
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
    [..] => None, /* default */
  }
}
```

We can also write a small unit test for this:
```rust
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
```
A passing test doesn't conclusively prove that our function works,
but it at least shows that the function does what we expect (as far as we tested).

### Invalid Input

One thing we *don't* handle quite right is invalid input.
Right now, our input is assumed to be correct.
If the input doesn't match a case we expect, then we just give back `None`.
If we're expecting to only process string literals with this, that's fine.
However, we might want to process *any* input at some point,
so let's do a little tweak to allow for lossy conversion of bad inputs.

All we have to do is break up the final `[..] => None` case into two cases.
* An empty string goes to `None`
* Our new "default" case gives the [Unicode Replacement Character](https://en.wikipedia.org/wiki/Specials_(Unicode_block)#Replacement_character)
  as the code point and consumes 1 byte if the current leading character is disallowed.

```rust
// in the `break_off_code_point` match
    [] => None,
    [_unknown, rest @ ..] => {
      // If we can't match anything above, we just pull off one byte and give
      // the unicode replacement code point.
      Some(('�' as u32, rest))
    }
```

This allows us to handle garbage in the middle of the input a little better.

It's still not perfectly conformant,
because we've decided to skip on checking the trailing bytes for validity,
but it's good enough in most cases that we'll make that trade.

## `count_utf16_code_units`

Alright, so our goal was to re-encode utf-8 as utf-16.
Now that we can iterate the code points of a utf-8 sequence,
how are we going to build a utf-16 sequence?

First, we need to get an output buffer. To put our output.
Since this is all in a const context, our output buffer is going to be an array.
How big of an array do we need?
Glad you asked.

Let's make another function for this:
```rust
pub const fn count_utf16_code_units(s: &str) -> usize {
  0
}
```

Off to a good start.

So we're going to walk the input string,
and then for each code *point* we determine if it needs 1 or 2 code *units* to be stored.
This will give us the capacity for how many `u16` values our array will need to be.

The rule to count code units in utf-16 is simple:
If the unicode code point value is less than or equal to 0xFFFF then it's 1 code unit in utf-16,
otherwise it's 2 code units in utf-16.

We can write this with a simply `while let` loop,
and we'll throw a unit test at it too.
```rust
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
```

Cool, so now we can const count the number of code units we'll need to have.

## Making A Macro

Okay, so now we want to input a string literal to something and get out a utf-16 literal.

Sadly, we can't do this with a const fn.
The output type would depend on the input value.
Rust doesn't like that idea at all.

Instead, we'll write a macro.

```rust
#[macro_export]
macro_rules! utf16 {
  // pass
}
```

Our macro has one match case it can do: you give it an expression and it'll process the text.

```rust
#[macro_export]
macro_rules! utf16 {
  ($text:expr) => {{
    todo!()
  }};
}
```

So when we get this `$text` value we want to assign it to a local const with a weird name.
This helps avoid some const-eval issues you can *potentially* get like if
the macro's caller has got an identifier in scope next to their macro usage that clashes with an identifier we're about to make in our macro.
It sounds unlikely, but it did come up in real code when developing the crate that this article is based on.
It doesn't really hurt, and it prevents a very cryptic error message from hitting the macro's user, so we'll do it.

```rust
#[macro_export]
macro_rules! utf16 {
  ($text:expr) => {{
    // Here we pick a name highly unlikely to exist in the scope
    // that $text came from, which prevents a potential const eval cycle error.
    const __A1B2C3D4_CONST_EVAL_LOOP_BREAK: &str = $text;
    const UTF8: &str = __A1B2C3D4_CONST_EVAL_LOOP_BREAK;
    todo!()
  }};
}
```

Next we'll make a const for the size of the output buffer.

```rust
#[macro_export]
macro_rules! utf16 {
  ($text:expr) => {{
    // Here we pick a name highly unlikely to exist in the scope
    // that $text came from, which prevents a potential const eval cycle error.
    const __A1B2C3D4_CONST_EVAL_LOOP_BREAK: &str = $text;
    const UTF8: &str = __A1B2C3D4_CONST_EVAL_LOOP_BREAK;
    const OUT_BUFFER_LEN: usize = $crate::util::count_utf16_code_units(UTF8);
    todo!()
  }};
}
```

Now we make a const for the output itself.
It's an array with a length equal to `OUT_BUFFER_LEN`,
but we need to mutate and fill it all in as we iterate the code points of the input,
so we'll compute it using an inner block.

We start with a zeroed buffer of the right size,
then we walk the input and write in each value.
Because the normal encoding utilities in the core library aren't `const fn`,
we have to do our own encoding math right here.

```rust
// ...
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
// ...
```

Finally, we just return the whole array.
In an initial version of this I had the output be just the data slice (`&[u16]`),
but in some select situations you do need the data in an owned form,
so the macro was adjusted to return the array directly.
If you want it to be a slice, just prefix the call with `&`.

```rust
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
```

And some handy tests wouldn't be out of place:
```rust
#[test]
fn test_utf16() {
  const HELLO16: [u16; 5] = utf16!("hello");
  assert_eq!(&HELLO16[..], &"hello".encode_utf16().collect::<Vec<u16>>());

  const WORDS8: &str = "$¢ह€한𐍈, 漢字, ひらがな / 平仮名, カタカナ / 片仮名";
  const WORDS16: &[u16] = &utf16!(WORDS8);
  assert_eq!(WORDS16, &WORDS8.encode_utf16().collect::<Vec<u16>>());
}
```

Ah, but often we want to have a *null terminated* string of utf-16.
That's also no trouble at all:
```rust
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
```

All very good, I hope you had fun with this one.
