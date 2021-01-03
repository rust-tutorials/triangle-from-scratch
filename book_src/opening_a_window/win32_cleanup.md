# Win32 Window Cleanup

During our introduction to Win32 I said that we'd just write all our code into `main.rs`,
and then we could sort it into the library later.

Well, later is now.

## New Files

This isn't very hard.
Cargo follows a "convention over configuration" style,
so as long as we do what it expects we won't even have to change `Cargo.toml` or anything.

First we copy `src/main.rs` into `examples/win32_window_standalone.rs`,
so that we can keep our nice small version that's all in one file.
Over time our library will build up,
and later lessons will refer to things we've put in our library already.
However, this first example can be understood in a single file,
without a person having to know what we've put into our library.
I think that's pretty valuable, so we will preserve this first draft for later viewing.

Then we make `src/lib.rs`.
This holds the top level items of the library.
Within the crate's module hierarchy, `lib.rs` is a module is *above* the other modules of the crate.
Within the filesystem on disk, `lib.rs` is a file *beside* the other files that make up the crate.
With *all other* modules, the default filesystem location for a module matches the module's logical crate location.
With `lib.rs` it's just slightly magical, so it has a different default filesystem location.
Sometimes people find this one exception confusing,
so I'm trying to be extra clear about what's going on regarding this point.

Next, since this is all Win32 specific stuff we'll be putting in the library right now,
which obviously doesn't work on all targets,
we'll make a `src/win32.rs` file,
and then declare it as a conditional module within the library.

```rust
// in lib.rs

#[cfg(windows)]
pub mod win32;
```

Now, if we're on windows, we'll be able to use our nice `win32` module.

## Put The Declarations In The Library

Okay first we put every single type, struct, const, and extern declaration into `win32.rs`.
We also have to make sure that all the fields and types are marked as `pub`.

Then we put a `use triangle_from_scratch::win32::*;` in our program.

The `main.rs` should only be left with `main` and `window_procedure`,
but the program should build and run exactly as before.

Also, I'm going to remove the `allow` attributes at the top of the `main.rs`,
and then convert the file to standard Rust naming for all the variables.
This doesn't make the program do better things, but it helps people read it.

## We Got Too Much Unsafe

All this `unsafe` code isn't great.
Having `unsafe` code around means that if we aren't careful we don't get "just" a wrong output or an unexpected panic,
instead we get some sort of *Undefined Behavior* (UB).
UB *might* do what you expected, or it *might* segfault, or it *might* be a security vulnerability.
I'd rather not have a security vulnerability in my program,
so I'd like to reduce the amount of `unsafe` code in the program as much as I can.

`/rant start`

Let's be very plain: You cannot fully eliminate `unsafe`.

Fundamentally, interacting with the outside world is `unsafe`.
If your program doesn't have any `unsafe` *anywhere* in the call stack,
then it's not actually interacting with the world at all,
and that's a pretty useless program to be running.

Operating systems and device drivers aren't designed to be free of UB.
They are designed for you to pay attention, and ask for the right thing at the right time.
We have to run `unsafe` code to get *anything* useful done.

However, `unsafe` code is **not** automatically the end of everything.
It's code that *can* go wrong, but that doesn't means it *must* go wrong.

Fire can burn down your house, but you also need it to forge metal.

`/rant end`

Our job, every time we use an `unsafe` block, is to make sure,
either with compile time checks or runtime checks,
that we don't call any `unsafe` functions improperly.

It's "that simple".
You know, "just" get good.

Every time we have an `unsafe` block, that block needs to be audited for correctness.
The less `unsafe` blocks we have, the less we need to audit.

Our strategy is that we want to put our `unsafe` blocks inside of safe functions,
and then the safe function performs whatever checks it needs to before actually making the `unsafe` function call.
That might mean a runtime check,
or the type system might even allow for static correctness to be known without a runtime check.
Either way, we get it right *once* in the safe wrapper function,
and then all other code only calls the safe function from there on.
Then we only have to audit the `unsafe` block in one location,
not everywhere all over the codebase.

I know, "put it in a function" is a very basic concept.
You could have thought of that too, I'm sure.
Still, sometimes it helps just to put the plan into words, even if it seems obvious.

## Safe Wrapping `GetModuleHandleW`

Alright so what can we *actually* make into safe functions?

Well the first thing the program does is call `GetModuleHandleW(null())`.
Is that legal?
I mean we can at least say that it seems intentional, that doesn't mean it's correct.
Let's checks the docs for [GetModuleHandleW](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew).

> lpModuleName: ... If this parameter is NULL, GetModuleHandle returns a handle to the file used to create the calling process (.exe file).

Okay, not only are we allowed to pass null, but if we do then we get a special sort of default-ish return value.

In terms of an interface for a wrapper function for this,
I think it'd be a little awkward to try and accept both null and non-null arguments and have it be ergonomic and stuff.
Thankfully, *we don't actually have to do that*.
We're just using the null argument style,
and if we use the non-null argument variant later we can just make it a totally separate function later.
So let's write a function for calling `GetModuleHandleW(null())`:

```rust
/// Returns a handle to the file used to create the calling process (.exe file)
///
/// See [`GetModuleHandleW`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew)
pub fn get_process_handle() -> HMODULE {
  // Safety: as per the MSDN docs.
  unsafe { GetModuleHandleW(null()) }
}
```

And now we can call that at the start of our `fn main()` in `main.rs`:
```rust
fn main() {
  let hinstance = get_process_handle();
  // ...
```
No `unsafe` block! One less thing to audit!

## Safe Wrapping `LoadCursorW`

So we go down a bit, the next `unsafe` function is `LoadCursorW`.
That one gave us some trouble for quite a while.
I was passing the wrong argument in the first position,
and then getting a null back without realizing it.
It wasn't actually UB, it was an allowed function call to make,
but I should have been checking the function's output and handling the error.
Forgetting to handle the error case is what was causing my downfall.

Let's have a look at [LoadCursorW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadcursorw).

We see that the `HINSTANCE` we pass has to be an executable file with a cursor in it.
That's a thing, did you know that?
You can put cursors and icons *inside* of executables and then programs can pick them out and use them.
I knew you could do that with icons for desktop shortcuts,
but I guess it works with cursors too.
Neat.

Right now the program says `LoadCursorW(null_mut(), IDC_ARROW)`, so can we pass null?
It doesn't say anything about null in the description for `hInstance`,
but lower down in the description for `lpCursorName` it says if you pass an `IDC_` value from the list as the `lpCursorName`,
then in that case you should set `hInstance` to null.

Like with how we wrapped `GetModuleHandleW`,
we don't need to make a *single* wrapper function that handles every possible case that `LoadCursorW` does.
Here, lets just make a function for loading the `IDC_` cursors.
If we want to load cursors out of non-null instances later on that can be a separate function.

And let's not forget to check that Return Value error case information:

> If the function succeeds, the return value is the handle to the newly loaded cursor.
> If the function fails, the return value is NULL. To get extended error information, call `GetLastError`.

Simple enough. What do the Remarks have to say? Ah, this part sounds important:

> This function returns a valid cursor handle only if the lpCursorName parameter is a pointer to a cursor resource.
> If lpCursorName is a pointer to any type of resource other than a cursor (such as an icon),
> the return value is not NULL, even though it is not a valid cursor handle.

Yikes, so we want to be very sure that we're passing a value from that list of allowed values.

Okay, so what if we make an enum of what you're allowed to pass in.
On the Rust side, you can only have an enum of a real variant.
Constructing an improper enum is already UB, so we don't even have to think about that case.
If we define our enum properly then we know people will only be allowed to pass correct values.

And since things can error, let's use a `Result` type for the output.
For now, the error will just be the `()` type.
We'll come back here once we've looked at `GetLastError`,
and made ourselves a useful error code type.

```rust
/// The predefined cursor styles.
pub enum IDCursor {
  /// Standard arrow and small hourglass
  AppStarting = 32650,
  /// Standard arrow
  Arrow = 32512,
  /// Crosshair
  Cross = 32515,
  /// Hand
  Hand = 32649,
  /// Arrow and question mark
  Help = 32651,
  /// I-beam
  IBeam = 32513,
  /// Slashed circle
  No = 32648,
  /// Four-pointed arrow pointing north, south, east, and west
  SizeAll = 32646,
  /// Double-pointed arrow pointing northeast and southwest
  SizeNeSw = 32643,
  /// Double-pointed arrow pointing north and south
  SizeNS = 32645,
  /// Double-pointed arrow pointing northwest and southeast
  SizeNwSe = 32642,
  /// Double-pointed arrow pointing west and east
  SizeWE = 32644,
  /// Vertical arrow
  UpArrow = 32516,
  /// Hourglass
  Wait = 32514,
}

/// Load one of the predefined cursors.
///
/// See [`LoadCursorW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadcursorw)
pub fn load_predefined_cursor(cursor: IDCursor) -> Result<HCURSOR, ()> {
  // Safety: The enum only allows values from the approved list. See MSDN.
  let hcursor =
    unsafe { LoadCursorW(null_mut(), MAKEINTRESOURCEW(cursor as WORD)) };
  if hcursor.is_null() {
    Err(())
  } else {
    Ok(hcursor)
  }
}
```

That's a lot of variants, but the wrapper function is still very simple.
The tag values of the enum are each set to the `MAKEINTRESOURCEW` input shown in the documentation.
When an enum is passed in, `cursor as WORD` will give us the tag value.
We pass that value to `MAKEINTRESOURCEW`, then it goes off to `LoadCursorW`.

Also, remember that `MAKEINTRESOURCEW` is just some type casting stuff,
it's not actually making any resources we have to free up later.

Let's check our `fn main` with this update:
```rust
  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(window_procedure);
  wc.hInstance = hinstance;
  wc.lpszClassName = sample_window_class_wn.as_ptr();
  wc.hCursor = load_predefined_cursor(IDCursor::Arrow).unwrap();
```
Ah ha! Now it's clear that we're going for a predefined cursor,
*and* it's clear that the call could fail.
Of course, using `unwrap` isn't a very robust way to solve problems.
It's *absolutely* not allowed in a good library (always pass the error back up!),
but in a binary it's "sorta okay, I guess", particularly since this is a demo.

## Partial Wrapping `RegisterClassW`

*Partial* wrapping?
I can hear you asking.

Yeah, you can't always make it safe.
But you can at least *almost* always make it better typed.

If we have a look at the docs of our next `unsafe` function call,
[RegisterClassW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassw),
It says "You must fill the structure with the appropriate class attributes before passing it to the function."

But the [WNDCLASSW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassw)
type is full of pointers to strings and stuff.
There's like three things that *aren't* pointers,
and then all the rest is a pile of pointers.
We don't have any easy way to track the validity of all the fields.
I'm sure it's possible to do something here to make sure that all fields are valid all the time,
but I'm also sure that the amount of effort that it would take would exceed the effort to just use an `unsafe` block and audit that code every so often.

So we're going to make a wrapper function,
but we'll leave it as an `unsafe` function.
Even then, we can give a better input and output type.

```rust
/// Registers a window class struct.
///
/// ## Safety
///
/// All pointer fields of the struct must be valid.
///
/// See [`RegisterClassW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassw)
pub unsafe fn register_class(window_class: &WNDCLASSW) -> Result<ATOM, ()> {
  let atom = RegisterClassW(window_class);
  if atom == 0 {
    Err(())
  } else {
    Ok(atom)
  }
}
```

Okay, and then the usage code:
```rust
  let atom = unsafe { register_class(&wc) }.unwrap_or_else(|()| {
    let last_error = unsafe { GetLastError() };
    panic!("Could not register the window class, error code: {}", last_error);
  });
```
Hmm.
At first glance, things didn't improve as much as we might have wanted.
Ah, but here's an interesting thing.
Now `atom` is marked as a *totally unused* variable.
We can't even forget to check for an error any more,
someone else already did that for us.

Still, the error case is very wonky.
That needs a fix.

## Safe Wrapping `GetLastError`

I'm pretty sure I remember how [GetLastError](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror)
worked.
It was super simple, right?
Yeah, it just gives you the thread-local last-error value.

```rust
/// Gets the thread-local last-error code value.
///
/// See [`GetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror)
pub fn get_last_error() -> DWORD {
  unsafe { GetLastError() }
}
```
Done.
Right?

Naw, of course not.
Keep reading.

> To obtain an error string for system error codes, use the [FormatMessage](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessage) function.
> For a complete list of error codes provided by the operating system, see [System Error Codes](https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes).

So we want to be able to print out the error string.
Really, the least we can do for our users.
Let's just have a quick look at `FormatMessage` and... oh... oh my...
You see that function signature?
There's flags, there's pointers, there's even a `va_list` thing we don't know about.
Oof.

## A Newtype For The Error Code

Okay, okay.
When there's a lot to do, one step at a time is usually the best way to do it.

```rust
#[repr(transparent)]
pub struct Win32Error(pub DWORD);
```

But, unlike the other types so far, this type is really intended to be shown to people.
For this, there's two main traits that Rust supports:
* `Debug` is for when you want to show the value *to a Rust programmer*.
* `Display` is for when you want to show the value *to the general public*.

So for our Debug impl, it can just show "Win32Error(12)" or similar.
This is exactly what a derived `Debug` impl will do, so we'll use the derive:

```rust
#[derive(Debug)]
#[repr(transparent)]
pub struct Win32Error(pub DWORD);
```

For Display we can't derive an implementation.
I don't mean that we *shouldn't* derive an implementation,
but that we literally cannot.
The standard library literally doesn't offer a derive for the `Display` trait.
That's because the standard library is managed by people who are very silly.
They have a silly concern that a derived Display impl "might" not show the right sort of info.
Instead of saying "if the derive doesn't do the right thing for you, write the impl by hand",
they just completely refuse to offer a derive at all.
Like I said, completely silly.

But we won't dwell on that too much,
because even if the derive was there, we wouldn't be able to use it in this case.

Instead... we get to use everyone's favorite function.... [FormatMessageW](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessagew).

```rust
impl core::fmt::Display for Win32Error {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    todo!("call FormatMessageW")
  }
}
```

First we define the extern we need:
```rust
pub type LPCVOID = *const core::ffi::c_void;
pub type va_list = *mut c_char;
pub type c_char = i8;
#[link(name = "Kernel32")]
extern "system" {
  /// [`FormatMessageW`](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessagew)
  pub fn FormatMessageW(
    dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD,
    lpBuffer: LPWSTR, nSize: DWORD, Arguments: va_list,
  ) -> DWORD;
}
```

Now let's go through each argument one by one.
```rust
impl core::fmt::Display for Win32Error {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    let dwFlags = todo!();
    let lpSource = todo!();
    let dwMessageId = todo!();
    let dwLanguageId = todo!();
    let lpBuffer = todo!();
    let nSize = todo!();
    let Arguments = todo!();
    let dword = unsafe {
      FormatMessageW(
        dwFlags,
        lpSource,
        dwMessageId,
        dwLanguageId,
        lpBuffer,
        nSize,
        Arguments,
      )
    };
    todo!("call FormatMessageW")
  }
}
```

* `dwFlags` lets us control a lot of options.
  Looking carefully, it seems like we want `FORMAT_MESSAGE_ALLOCATE_BUFFER`,
  which makes `FormatMessageW` perform the allocation of a large enough buffer.
  But then... if we use this `lpBuffer` gets a little odd.
  More on that in a moment.
  We also want `FORMAT_MESSAGE_FROM_SYSTEM`, since these are system errors.
  All done? Not quite.
  If we skip ahead down the page to "Security Remarks" then we see that we need `FORMAT_MESSAGE_IGNORE_INSERTS` too.
  *Now* are we done?
  Hmm, if we set the low-order byte we can fiddle the line length, but we don't need that.
  We'll leave it as 0.
* `lpSource` is the location of the message definition.
  It's only used if our message is from an hmodule or a string.
  Since our message is from the system the argument is ignored,
  so we'll leave this as null.
* `dwMessageId` is the identifier of the requested message.
  That means the error code, so we'll set `self.0` here.
* `dwLanguageId` is the language identifier of the message.
  Happily, if we just pass 0, then it'll basically look up the best message it can,
  and then format that. So we'll just pass 0.
* `lpBuffer` is... hey we had to remember something about this!
  Okay so because we're using `FORMAT_MESSAGE_ALLOCATE_BUFFER`...
  well *normally* this would be interpreted as `LPWSTR` (pointer to a null-terminated wide string).
  However, since we're using `FORMAT_MESSAGE_ALLOCATE_BUFFER`,
  instead the function will use our pointer as a pointer to the start of a buffer.
  The `lpBuffer` is written with the buffer start info,
  and then we read it back after the function completes,
  and we get our allocation that way.
  So, in our use case, the `lpBuffer` arg is a pointer *to a pointer*.
  We have to be careful about this point.
* `nSize` is the size of the output buffer, if you're providing the output buffer,
  or it's the minimum output buffer size you want if you're using `FORMAT_MESSAGE_ALLOCATE_BUFFER`.
  We don't have any minimum needs, so we'll give 0.
* `Arguments` is the insert arguments for the formatting.
  However, we're using `FORMAT_MESSAGE_IGNORE_INSERTS`, so we'll pass null.

**Returns:** what we get back is the number of `TCHAR` values stored in the buffer,
*excluding* the final null character.
A `TCHAR` is either an `i8` (for `A` functions) or a `u16` (for `W` functions).

Okay, so, let's review what we've got so far, because this is a lot of little things:
```rust
impl core::fmt::Display for Win32Error {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    let dwFlags = FORMAT_MESSAGE_ALLOCATE_BUFFER
      | FORMAT_MESSAGE_FROM_SYSTEM
      | FORMAT_MESSAGE_IGNORE_INSERTS;
    let lpSource = null_mut();
    let dwMessageId = self.0;
    let dwLanguageId = 0;
    let mut buffer: *mut u16 = null_mut();
    let lpBuffer = &mut buffer as *mut *mut u16 as *mut u16;
    let nSize = 0;
    let Arguments = null_mut();
    let tchar_count_excluding_null = unsafe {
      FormatMessageW(
        dwFlags,
        lpSource,
        dwMessageId,
        dwLanguageId,
        lpBuffer,
        nSize,
        Arguments,
      )
    };
    todo!("read the buffer")
  }
}
```

So if we got a count of 0, of if the buffer is still null,
then there was some sort of problem.

```rust
if tchar_count_excluding_null == 0 || buffer.is_null() {
  // some sort of problem happened. we can't usefully get_last_error since
  // Display formatting doesn't let you give an error value.
  return Err(core::fmt::Error);
} else {
  todo!()
}
```

If there was no problem then we need to access the buffer.
The simplest way is to turn it into a slice:
```rust
    } else {
      let buffer_slice: &[u16] = unsafe {
        core::slice::from_raw_parts(buffer, tchar_count_excluding_null as usize)
      };
      todo!()
    }
```

Now we can decode the data with [decode_utf16](https://doc.rust-lang.org/core/char/fn.decode_utf16.html).
This iterates over the `u16` values, producing `Result<char, DecodeUtf16Error>` as it goes.
If there was any decoding error, let's just use the standard [Unicode Replacement Character](https://en.wikipedia.org/wiki/Specials_(Unicode_block)#Replacement_character) instead.
Then we put whatever character we've got into the output.
```rust
for decode_result in
  core::char::decode_utf16(buffer_slice.iter().copied())
{
  let ch = decode_result.unwrap_or('�');
  write!(f, "{}", ch)?;
}
```

Cool.
All done?
Ah, not quite.

Remember how we had `FormatMessageW` allocate the buffer for us?
We need to free that buffer or we'll have a memory leak.
A memory leak is *safe*, but it's still *bad*.

There's more than one allocation system within Windows.
To free this memory, `FormatMessageW` says that we need to use [LocalFree](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-localfree).
```rust
pub type HLOCAL = HANDLE;
#[link(name = "Kernel32")]
extern "system" {
  /// [`LocalFree`](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-localfree)
  pub fn LocalFree(hMem: HLOCAL) -> HLOCAL;
}
```

So where's our call to `FreeLocal` go?
At the end, right?
Except we also have all those `?` operators on the writing.
Any of those can early return from the function.

Let's use the magic of `Drop` to solve our problem.
```rust
struct OnDropLocalFree(HLOCAL);
impl Drop for OnDropLocalFree {
  fn drop(&mut self) {
    unsafe { LocalFree(self.0) };
  }
}
let _on_drop = OnDropLocalFree(buffer as HLOCAL);
let buffer_slice: &[u16] = unsafe {
  core::slice::from_raw_parts(buffer, tchar_count_excluding_null as usize)
};
for decode_result in
  core::char::decode_utf16(buffer_slice.iter().copied())
{
  let ch = decode_result.unwrap_or('�');
  write!(f, "{}", ch)?;
}
Ok(())
```
Isn't that cool?
I think it's pretty cool.

One small note: we have to be sure to bind it to a local variable.
If we didn't bind it to a local variable, or if we bound it to the special `_` variable,
then the struct would drop *immediately* (before we read the buffer),
and then things would go very wrong.

If we test it out with error code 0, we can see "The operation completed successfully.\r\n".
Hmm, let's eat up those newline characters though. We didn't want those.
```rust
match decode_result {
  Ok('\r') | Ok('\n') => write!(f, " ")?,
  Ok(ch) => write!(f, "{}", ch)?,
  Err(_) => write!(f, "�")?,
}
```
that's better.

One other note: if the 29th bit is set, then it's an application error.
The system doesn't know how to format those, so we won't even ask it.
Instead, we'll just show display that and return early.
```rust
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    if self.0 & (1 << 29) > 0 {
      return write!(f, "Win32ApplicationError({})", self.0);
    }
```

We want our error getting function to use this great new type we worked on:
```rust
/// Gets the thread-local last-error code value.
///
/// See [`GetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror)
pub fn get_last_error() -> Win32Error {
  Win32Error(unsafe { GetLastError() })
}
```

And then we can update our stuff that returns `Result` types to use this error type.

```rust
pub fn load_predefined_cursor(cursor: IDCursor) -> Result<HCURSOR, Win32Error> {
  // ...
}

pub unsafe fn register_class(
  window_class: &WNDCLASSW,
) -> Result<ATOM, Win32Error> {
  // ...
}
```

There's also the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html) trait.
It's a bit of a mess right now, but there's a Working Group trying to develop things to be better in the future.
At the moment, we might as well implement `std::error::Error` for our error type,
just to be potentially more compatible with the rest of the Rust ecosystem.
It's not like we even have to do anything, it's all default methods:
```rust
impl std::error::Error for Win32Error {}
```

## Window Creation

The next `unsafe` function that our `main` calls is `CreateWindowExW`.
This one is one heck of a swiss-army-chainsaw of a function.
See, it turns out that, in Win32, not only are the things we think of as windows "windows",
but *tons* of the GUI elements are "windows" too.
It's all windows, everywhere, all over the place.

So `CreateWindowExW` has like a million options it can do.
It also has a ton of arguments that can't be easily verified.
It's just as bad as `register_class`,
the only difference is that the arguments are passed as arguments,
instead of being stuffed into a struct and then passed as a single struct.

Like we did with with `register_class`,
we're gonna basically skip on the verification and leave it as `unsafe`.
What we will do is give it a `Result` for the output,
so that we enforce the error handling on ourselves.

```rust
/// Creates a window.
///
/// See [`CreateWindowExW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw)
pub unsafe fn create_window_ex_w(
  ex_style: DWORD, class_name: LPCWSTR, window_name: LPCWSTR, style: DWORD,
  x: c_int, y: c_int, width: c_int, height: c_int, parent: HWND, menu: HMENU,
  instance: HINSTANCE, param: LPVOID,
) -> Result<HWND, Win32Error> {
  let hwnd = CreateWindowExW(
    ex_style,
    class_name,
    window_name,
    style,
    x,
    y,
    width,
    height,
    parent,
    menu,
    instance,
    param,
  );
  if hwnd.is_null() {
    Err(get_last_error())
  } else {
    Ok(hwnd)
  }
}
```

Ugh.
Are we really adding value here?
Isn't the point to like, you know, cut down on accidents?
Let's simplify this.
* First of all, we'll just accept `&str` and then make wide strings ourselves.
  This lets us use string literals, and the extra allocation isn't a huge deal.
  We're already calling the OS to make a window, so this isn't a "hot path" function.
* Next, we won't accept `ex_style` or `style` values.
  We'll just pick some "good default" values to use.
  Since a user can always just bypass our decision if they really want to
  (by calling `CreateWindowExW` themselves), it's fine.
* Instead of accepting `x` and `y`, we'll just take an `Option<[i32;2]>` as the position.
  If you give a `Some` then it uses the two array values as the `x` and `y`.
  If you give a `None` then both `x` and `y` will be `CW_USEDEFAULT`, which gives a default position.
  This is *much* simpler than the normal rules for how `CW_USEDEFAULT` works.
  The normal rules seriously take up about two paragraphs of the `CreateWindowExW` documentation.
* Also, the window size can be `[i32; 2]`.
  It doesn't seem particularly useful to keep the ability to have a default size.
  It's not a huge burden to have the caller always pick a size.
* We don't need to specify a parent window.
  We'll always pass null, so that's one less thing for the caller to think about.
* We don't need to specify a custom menu to use.
  A null argument here means to use the class window,
  so if we wanna change the menu we'd change it on the window class.
  Again, one less thing for the caller to think about in the 99% case.
* The instance isn't useful to pass in,
  we can just have `create_app_window` look up the instance itself.
* We'll rename `param` to `create_param`.
  Normally, the styles used can change the meaning of this pointer.
  With the styles we're using, this will be the argument to the `WM_NCCREATE` event.

```rust
pub unsafe fn create_app_window(
  class_name: &str, window_name: &str, position: Option<[i32; 2]>,
  [width, height]: [i32; 2], create_param: LPVOID,
) -> Result<HWND, Win32Error> {
  // ...
}
```
That's a *lot* less for the caller to think about.
We can call it at moderate improvement.

## Messages

Next would be [ShowWindow](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow),
but I'm not sure we can provide much help there.
We don't have a general window abstraction where we can be sure that a `HWND` is real or not.
So even if we made an enum for the second arg, it'd be an `unsafe` function overall.
There's also no error value to help fix up into an `Option` or `Result`.
I suppose we'll skip over it for now.

Instead, let's have a look at [GetMessageW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew).
Here's a function where I think we can make some improvements.

The basic output of `GetMessageW` is 0 for a quit event,
or non-zero for anything else,
and if the "anything else" was an error, then it's specifically -1.
That's because it's *kinda* intended to be used with C's looping constructs,
where test expressions evaluating to 0 will cancel the loop.
Except, it doesn't work well with C loops because you end up missing the error when you get -1
(which isn't 0, so you'd continue the loop).
In fact MSDN *specifically* tells you to not write `while (GetMessage(lpMsg, hWnd, 0, 0)) {`,
because it does the wrong thing,
and presumably enough people wrote that and asked why it went wrong that they put it on the docs to not do that.
So I think we can easily say that they picked the wrong sentinel values for `GetMessageW` to use.
Still, they are what they are,
we'll just adapt a bit.
Instead, let's focus on if we got a message or not,
and then we can worry about if it was a quit event in the calling code.
What we want is something like this:
```rust
/// Gets a message from the thread's message queue.
///
/// The message can be for any window from this thread,
/// or it can be a non-window message as well.
///
/// See [`GetMessageW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew)
#[inline(always)]
pub fn get_any_message() -> Result<MSG, Win32Error> {
  let mut msg = MSG::default();
  let output = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
  if output == -1 {
    Err(get_last_error())
  } else {
    Ok(msg)
  }
}
```

And then in `main` adjust how we call it just a bit:
```rust
loop {
  match get_any_message() {
    Ok(msg) => {
      if msg.message == WM_QUIT {
        break;
      }
      unsafe {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      }
    }
    Err(e) => panic!("Error when getting from the message queue: {}", e),
  }
}
```

Next, we can make [TranslateMessage](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translatemessage) safe too.
```rust
/// Translates virtual-key messages into character messages.
///
/// The character messages go into your thread's message queue,
/// and you'll see them if you continue to consume messages.
///
/// **Returns:**
/// * `true` if the message was `WM_KEYDOWN`, `WM_KEYUP`, `WM_SYSKEYDOWN`, or
///   `WM_SYSKEYUP`.
/// * `true` for any other message type that generated a character message.
/// * otherwise `false`
///
/// See [`TranslateMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translatemessage)
pub fn translate_message(msg: &MSG) -> bool {
  0 != unsafe { TranslateMessage(msg) }
}
```

Can we make `DispatchMessageW` safe just as easily?
Sadly, no.
Using `DispatchMessageW` causes the window procedure to be called,
*or* it can cause a timer callback to be called.
Since a call to `DispatchMessageW` with a funky `MSG` value could make arbitrary functions get called,
and with arbitrary arguments,
then we cannot wrap `DispatchMessageW` in a safe way.
In the case of `main`, we can see that we're not messing with the fields of the message,
everything in the message is what the operating system said,
so we know the message content is "real" content.
However, if we put a safe version of `DispatchMessageW` into our library,
that library code wouldn't actually be correct for all possible message inputs.

## Get/Set Window Long Pointer

When we're using [SetWindowLongPtrW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw),
and also the `Get` version,
there's a lot of options going on.
Also, we're also not checking the error values properly at the moment.

What's supposed to happen with the *setter* is that you set a value,
and the return value is the previous value.
If there's an error, then you get 0 back (and you call `GetLastError`).
Except, if the previous value was 0, then you can't tell if things are wrong or not.
So what you do is you first call [SetLastError](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-setlasterror),
which we haven't used yet,
and you set the error code to 0.
Then you do `SetWindowLongPtrW` and if you do get a 0,
then you can check the error code.
If the error code is still the 0 that you set it to,
then actually you had a "successful" 0.
The `GetWindowLongPtrW` behaves basically the same.

For now, we'll *only* support getting/setting the userdata pointer.
This simplifies the problem immensely.

First we need to declare that we'll be using `SetLastError`
```rust
#[link(name = "Kernel32")]
extern "system" {
  /// [`SetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-setlasterror)
  pub fn SetLastError(dwErrCode: DWORD);
}
```

And we'll make this callable as a safe operation,
```rust
/// Sets the thread-local last-error code value.
///
/// See [`SetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-setlasterror)
pub fn set_last_error(e: Win32Error) {
  unsafe { SetLastError(e.0) }
}
```

Now we can make an unsafe function for setting the userdata pointer:
```rust
/// Sets the "userdata" pointer of the window (`GWLP_USERDATA`).
///
/// **Returns:** The previous userdata pointer.
///
/// [`SetWindowLongPtrW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw)
pub unsafe fn set_window_userdata(
  hwnd: HWND, ptr: *mut c_void,
) -> Result<*mut c_void, Win32Error> {
  set_last_error(Win32Error(0));
  let out = SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as LONG_PTR);
  if out == 0 {
    // if output is 0, it's only a "real" error if the last_error is non-zero
    let last_error = get_last_error();
    if last_error.0 != 0 {
      Err(last_error)
    } else {
      Ok(out as *mut c_void)
    }
  } else {
    Ok(out as *mut c_void)
  }
}
```

And this lets us upgrade our window creation process a bit:
```rust
// in `window_procedure`
    WM_NCCREATE => {
      println!("NC Create");
      let createstruct: *mut CREATESTRUCTW = lparam as *mut _;
      if createstruct.is_null() {
        return 0;
      }
      let boxed_i32_ptr = (*createstruct).lpCreateParams;
      return set_window_userdata(hwnd, boxed_i32_ptr).is_ok() as LRESULT;
    }
```

The getter for the userdata pointer is basically the same deal:
```rust
/// Gets the "userdata" pointer of the window (`GWLP_USERDATA`).
///
/// **Returns:** The userdata pointer.
///
/// [`GetWindowLongPtrW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowlongptrw)
pub unsafe fn get_window_userdata(
  hwnd: HWND,
) -> Result<*mut c_void, Win32Error> {
  set_last_error(Win32Error(0));
  let out = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
  if out == 0 {
    // if output is 0, it's only a "real" error if the last_error is non-zero
    let last_error = get_last_error();
    if last_error.0 != 0 {
      Err(last_error)
    } else {
      Ok(out as *mut c_void)
    }
  } else {
    Ok(out as *mut c_void)
  }
}
```

Now we can adjust how WM_DESTROY and WM_PAINT are handled.
```rust
// in `window_procedure`
    WM_DESTROY => {
      match get_window_userdata(hwnd) {
        Ok(ptr) if !ptr.is_null() => {
          Box::from_raw(ptr);
          println!("Cleaned up the box.");
        }
        Ok(_) => {
          println!("userdata ptr is null, no cleanup")
        }
        Err(e) => {
          println!("Error while getting the userdata ptr to clean it up: {}", e)
        }
      }
      PostQuitMessage(0);
    }
    WM_PAINT => {
      match get_window_userdata(hwnd) {
        Ok(ptr) if !ptr.is_null() => {
          let ptr = ptr as *mut i32;
          println!("Current ptr: {}", *ptr);
          *ptr += 1;
        }
        Ok(_) => {
          println!("userdata ptr is null")
        }
        Err(e) => {
          println!("Error while getting the userdata ptr: {}", e)
        }
      }
      let mut ps = PAINTSTRUCT::default();
      let hdc = BeginPaint(hwnd, &mut ps);
      let _success = FillRect(hdc, &ps.rcPaint, (COLOR_WINDOW + 1) as HBRUSH);
      EndPaint(hwnd, &ps);
    }
```

## PostQuitMessage

This one is easy to make safe:
you give it an exit code, and that exit code goes with the WM_QUIT message you get back later on.

There's nothing that can go wrong, so we just wrap it.
```rust
/// Indicates to the system that a thread has made a request to terminate
/// (quit).
///
/// The exit code becomes the `wparam` of the [`WM_QUIT`] message your message
/// loop eventually gets.
///
/// [`PostQuitMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage)
pub fn post_quit_message(exit_code: c_int) {
  unsafe { PostQuitMessage(exit_code) }
}
```

And then we just put that as the last line of the `WM_DESTROY` branch.

## BeginPaint

Our next target is [BeginPaint](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-beginpaint),
which is another thing that's simple to make easier to use when you've got Rust types available.

```rust
/// Prepares the specified window for painting.
///
/// On success: you get back both the [`HDC`] and [`PAINTSTRUCT`]
/// that you'll need for future painting calls (including [`EndPaint`]).
///
/// [`BeginPaint`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-beginpaint)
pub unsafe fn begin_paint(
  hwnd: HWND,
) -> Result<(HDC, PAINTSTRUCT), Win32Error> {
  let mut ps = PAINTSTRUCT::default();
  let hdc = BeginPaint(hwnd, &mut ps);
  if hdc.is_null() {
    Err(get_last_error())
  } else {
    Ok((hdc, ps))
  }
}
```

## FillRect

Using [FillRect](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-fillrect)
you can paint using an HBRUSH *or* a system color.

We only want to support the system color path.
First we make an enum for all the system colors.
This is a little fiddly because some values are named more than once,
and so we have to pick just a single canonical name for each value,
but it's not too bad:
```rust
/// See [`GetSysColor`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getsyscolor)
pub enum SysColor {
  _3dDarkShadow = 21,
  _3dLight = 22,
  ActiveBorder = 10,
  ActiveCaption = 2,
  AppWorkspace = 12,
  /// Button face, also "3D face" color.
  ButtonFace = 15,
  /// Button highlight, also "3D highlight" color.
  ButtonHighlight = 20,
  /// Button shadow, also "3D shadow" color.
  ButtonShadow = 16,
  ButtonText = 18,
  CaptionText = 9,
  /// Desktop background color
  Desktop = 1,
  GradientActiveCaption = 27,
  GradientInactiveCaption = 28,
  GrayText = 17,
  Highlight = 13,
  HighlightText = 14,
  HotLight = 26,
  InactiveBorder = 11,
  InactiveCaption = 3,
  InactiveCaptionText = 19,
  InfoBackground = 24,
  InfoText = 23,
  Menu = 4,
  MenuHighlight = 29,
  MenuBar = 30,
  MenuText = 7,
  ScrollBar = 0,
  Window = 5,
  WindowFrame = 6,
  WindowText = 8,
}
```

and then we make a function to fill in with a system color:
```rust
/// Fills a rectangle with the given system color.
///
/// When filling the specified rectangle, this does **not** include the
/// rectangle's right and bottom sides. GDI fills a rectangle up to, but not
/// including, the right column and bottom row, regardless of the current
/// mapping mode.
///
/// [`FillRect`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-fillrect)
pub unsafe fn fill_rect_with_sys_color(
  hdc: HDC, rect: &RECT, color: SysColor,
) -> Result<(), ()> {
  if FillRect(hdc, rect, (color as u32 + 1) as HBRUSH) != 0 {
    Ok(())
  } else {
    Err(())
  }
}
```

## EndPaint

You might think that [EndPaint](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-endpaint)
has some sort of error code we're ignoring.
It returns a BOOL right?
Actually when you check the docs, "The return value is always nonzero".
In other words, the function might as well return nothing.

```rust
/// See [`EndPaint`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-endpaint)
pub unsafe fn end_paint(hwnd: HWND, ps: &PAINTSTRUCT) {
  EndPaint(hwnd, ps);
}
```
Not a big gain in terms of API quality.
However, this way the caller can at least see they're supposed to pass a real paint struct,
and not possibly a null pointer.
Also, now it's clear that there's no output value.
We'll call it a small win.

```rust
// in `window_procedure`
      match begin_paint(hwnd) {
        Ok((hdc, ps)) => {
          let _ = fill_rect_with_sys_color(hdc, &ps.rcPaint, SysColor::Window);
          end_paint(hwnd, &ps);
        }
        Err(e) => {
          println!("Couldn't begin painting: {}", e)
        }
      }
```

## A Painting Closure

Is it easy to mess up the whole begin/end painting thing?
Yeah, I could see that going wrong.
One thing we might want to *try* is having a function that takes closure to do painting.

The function signature for this is pretty gnarly,
because *anything* with a closure is gnarly.

The closure is going to get three details it needs to know from the `PAINTSTRUCT`:
* The HDC.
* If the background needs to be erased or not.
* The target rectangle for painting.
Everything else in the `PAINTSTRUCT` is just system reserved info that we don't even care about.

Our library function will get the HWND and the closure.
It starts the painting,
runs the closure,
and then ends the painting.
Remember that we want the painting to be ended *regardless* of success/failure of the closure.
```rust
/// Performs [`begin_paint`] / [`end_paint`] around your closure.
pub unsafe fn do_some_painting<F, T>(hwnd: HWND, f: F) -> Result<T, Win32Error>
where
  F: FnOnce(HDC, bool, RECT) -> Result<T, Win32Error>,
{
  let (hdc, ps) = begin_paint(hwnd)?;
  let output = f(hdc, ps.fErase != 0, ps.rcPaint);
  end_paint(hwnd, &ps);
  output
}
```

Neat!

Note that, to write this, we needed to make `RECT` a `Copy` type.
Most all the C structs we're declaring should be Debug, Clone, Copy, etc.
We just didn't add all the impls at the time.

What's it look like in practice?
Not too bad at all:
```rust
// in `window_procedure`
    WM_PAINT => {
      match get_window_userdata(hwnd) {
        Ok(ptr) if !ptr.is_null() => {
          let ptr = ptr as *mut i32;
          println!("Current ptr: {}", *ptr);
          *ptr += 1;
        }
        Ok(_) => {
          println!("userdata ptr is null")
        }
        Err(e) => {
          println!("Error while getting the userdata ptr: {}", e)
        }
      }
      do_some_painting(hwnd, |hdc, _erase_bg, target_rect| {
        let _ = fill_rect_with_sys_color(hdc, &target_rect, SysColor::Window);
        Ok(())
      })
      .unwrap_or_else(|e| println!("error during painting: {}", e));
    }
```

What I like the most about this is that the user can still call `begin_paint` and `end_paint` on their own if they want.
Because maybe we make some abstraction workflow thing that doesn't work for them,
and they can just skip around our thing if that's the case.

## Using The Exit Code

One thing we don't do is pass along the `wParam` from the `MSG` struct when we see `WM_QUIT`.
We're *supposed* to pass that as the exit code of our process.
For this, we can use `std::process:exit`, and then pass the value, instead of just breaking the loop.

```rust
// in main
  loop {
    match get_any_message() {
      Ok(msg) => {
        if msg.message == WM_QUIT {
          std::process::exit(msg.wParam as i32);
        }
        translate_message(&msg);
        unsafe {
          DispatchMessageW(&msg);
        }
      }
      Err(e) => panic!("Error when getting from the message queue: {}", e),
    }
  }
```

## Done

Is our program perfect?
Naw, but I think it's good enough for now.
