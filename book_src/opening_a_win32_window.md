
# Opening a Win32 Window

Alright, if we wanna draw a triangle, we have to have a window to draw the triangle in.

Uh, how do we do that? For the sake of the lesson, let's imagine I don't know how to do that.

## Search The Web

Okay so we don't know what to do, let's ask the internet nicely.
Something like ["open a window win32"](https://duckduckgo.com/?q=open+a+window+win32) sounds right.
Hey look, that [first result](https://docs.microsoft.com/en-us/windows/win32/learnwin32/creating-a-window) is straight from Microsoft.
It's a whole little tutorial on how to open a window.
Perfect, just what we wanted.

## Starting The Win32 Windowing Tutorial

Alright, let's read the first paragraph of the [windowing tutorial](https://docs.microsoft.com/en-us/windows/win32/learnwin32/creating-a-window)
that we just found...

To summarize the opening portion:
* Every window needs a window class.
* A window class is registered with the OS at runtime.
* We need to fill in a [WNDCLASSA](https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassa) (or [WNDCLASSW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassw))

Whoa, slow down, hold on, what's this structure thing? And why are there two versions?

### ANSI and Wide

All over the Win32 API you'll find stuff where there's an `A` version and a `W` version.
This happens with functions that process textual data, as well as with structs associated with those functions.
In this case of `WNDCLASSA` / `WNDCLASSW`, a window class has, as part of it, a menu name as well as a class name.
These names are textual, and so we get both an `A` and a `W` version.

The `A` and `W` letters come from the two types of string that the windows API lets you use: ANSI strings and "wide" strings.
* ANSI strings use C's `char` type.
  They don't have a specified encoding.
  If you store anything other than ASCII data in an ANSI string, the results vary based on context.
* Wide strings use C's `wchar_t` type.
  These strings are UTF-16 encoded.
  This gives you consistent results while using all the world's writing systems.

#### What does this mean for us Rust users?

Well, Rust string literals, and Rust's normal `String` and `&str` types, are all UTF-8 encoded.
This means there's a bit of a mismatch between what Windows expects and what we've usually got.

UTF-8 is a *superset* of ASCII.
This means that any ASCII-only string can be stored compatibly inside a UTF-8 string.
So if we only want to use ASCII data the normal `String` and `&str` types will be (mostly) compatible with `A`-type operations.

On the other hand, ASCII is pretty limited.
Most languages of the world aren't representable with only ASCII text.
You get English, Latin, Esperanto, Klingon, but the list runs out quick after that.
Even English doesn't get all of its fancy typographical marks in an ASCII-only context:
ellipses (…), “angled quotes”, different length dashes (– / —), and so on.

So we really want to be using these `W`-type operations.
This means that we have to convert UTF-8 over to UTF-16.
Oh, hey, look, [that's in the standard library](https://doc.rust-lang.org/std/primitive.str.html#method.encode_utf16), isn't it neat?
The only slight problem is that we can't use that in a `const` context (yet).
It's not the worst to do a little runtime data mucking here and there, so we'll accept the overhead.
The UTF-16 conversion is kinda just an "unfortunate but accepted" part of working with Windows.

### Reading a C struct declaration

Okay, so now we've picked that we're going to use `WNDCLASSW`.
Let's look at the [MSDN definition](https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassw):

```c
typedef struct tagWNDCLASSW {
  UINT      style;
  WNDPROC   lpfnWndProc;
  int       cbClsExtra;
  int       cbWndExtra;
  HINSTANCE hInstance;
  HICON     hIcon;
  HCURSOR   hCursor;
  HBRUSH    hbrBackground;
  LPCWSTR   lpszMenuName;
  LPCWSTR   lpszClassName;
} WNDCLASSW, *PWNDCLASSW, *NPWNDCLASSW, *LPWNDCLASSW;
```

Oh, gross, what the heck?
What's going on here?
Let's take it one part at a time.

* `typedef` says that we're making a "type definition".
  The way it works is that first you give a base type, and then you list one or more other names you want to have as aliases.
* `struct tagWNDCLASSW` this names the first type, that we're making the aliases for.
* `{ ... }` the part in braces lists the fields of the struct.
  Each line has the field's type, then the name of the field, then a `;`
* `WNDCLASSW,` is the first alias we're making.
  From now on, if you refer to a `WNDCLASSW`, then it's the same as if you'd referred to the whole `struct tagWNDCLASSW { ... }` declaration.
  This is really good, because writing out all the fields any time we just want to talk about the type is just a pain.
* `*PWNDCLASSW, *NPWNDCLASSW, *LPWNDCLASSW;` these are more aliases as well.
  The `*` makes these pointer types, so a `PWNDCLASSW` is the same as `struct tagWNDCLASSW { ... } *` or `WNDCLASSW*`.
  The prefixes on each name variant stand for "Pointer", "Near Pointer", and "Long Pointer".
  Long ago when computers had segmented memory there were differences in the pointer types.
  These days computers aren't set up for that, so they're all just a normal pointer.
  The different names are still around for legacy compatability.

### Starting Our Rust Code

I think we've got enough on our plate to start writing things down in Rust.

```
Microsoft Windows [Version 10.0.19041.685]
(c) 2020 Microsoft Corporation. All rights reserved.

D:\dev\triangle-from-scratch>cargo init --bin
     Created binary (application) package

D:\dev\triangle-from-scratch>cargo run
   Compiling triangle-from-scratch v0.1.0 (D:\dev\triangle-from-scratch)
    Finished dev [unoptimized + debuginfo] target(s) in 0.65s
     Running `target\debug\triangle-from-scratch.exe`
Hello, world!
```

Great.
Later on we'll put some of this into a library,
sort it into modules,
all that sort of housekeeping stuff.
For now, we'll just write into `main.rs`.

```rust
#[repr(C)]
struct WNDCLASSW {
  style: UINT,
  lpfnWndProc: WNDPROC,
  cbClsExtra: int,
  cbWndExtra: int,
  hInstance: HINSTANCE,
  hIcon: HICON,
  hCursor: HCURSOR,
  hbrBackground: HBRUSH,
  lpszMenuName: LPCWSTR,
  lpszClassName: LPCWSTR,
}
```

Oh, excellent, and we're sure to put that little `repr(C)` at the top.
This makes sure it has the right [memory layout](https://doc.rust-lang.org/reference/type-layout.html) for interacting with foreign code.

Let's give that a try:
```
D:\dev\triangle-from-scratch>cargo run
   Compiling triangle-from-scratch v0.1.0 (D:\dev\triangle-from-scratch)
error[E0412]: cannot find type `UINT` in this scope
 --> src\main.rs:9:10
  |
9 |   style: UINT,
  |          ^^^^ not found in this scope

error[E0412]: cannot find type `WNDPROC` in this scope
  --> src\main.rs:10:16
   |
10 |   lpfnWndProc: WNDPROC,
   |                ^^^^^^^ not found in this scope

...you get the idea
```

Okay, so, that should be obvious enough in terms of the error message.
We can't declare a struct to have fields with types Rust doesn't know about.
It's just not gonna fly.

### How Big Is An Int?

Okay, start with just the first field on the list of missing types.
Another web search for ["msdn uint"](https://duckduckgo.com/?q=msdn+uint),
and we find a handy page of [Windows Data Types](https://docs.microsoft.com/en-us/windows/win32/winprog/windows-data-types).
```
UINT: An unsigned INT. The range is 0 through 4294967295 decimal.

This type is declared in WinDef.h as follows:

typedef unsigned int UINT;
```

Alright, closer to an answer.
Now we just ask ["how big is an int on windows"](https://duckduckgo.com/?q=how+big+is+an+int+on+windows),
which doesn't have any pages that immediately look useful.
What if we ask ["how big is an int on windows msdn"](https://duckduckgo.com/?q=how+big+is+an+int+on+windows+msdn)?
Ah, here we go, [Data Type Ranges](https://docs.microsoft.com/en-us/cpp/cpp/data-type-ranges)
gives us all the info we need about the size of different C types.

An `unsigned int` is 4 bytes, so in Rust terms it's a `u32`.
We could call our type `unsigned_int`, but the rust convention is to give C types a `c_` prefix, and also to just say `u` for "unsigned".
In other words, `unsigned int` in C becomes [c_uint](https://doc.rust-lang.org/std/os/raw/type.c_uint.html) in the Rust convention.
There's no strong reason to *not* keep with this naming convention, so we'll go with that.

Now we can add definitions that get us up to `UINT`,
and we can do signed ints as well while we're at it:

```rust
#[repr(C)]
struct WNDCLASSW {
  style: UINT,
  lpfnWndProc: WNDPROC,
  cbClsExtra: c_int,
  cbWndExtra: c_int,
  hInstance: HINSTANCE,
  hIcon: HICON,
  hCursor: HCURSOR,
  hbrBackground: HBRUSH,
  lpszMenuName: LPCWSTR,
  lpszClassName: LPCWSTR,
}
type UINT = c_uint;
type c_uint = u32;
type c_int = i32;
```

Three of the fields aren't underlined in red already!

### Reading a C function declaration

Now we [look up WNDPROC](https://duckduckgo.com/?q=WNDPROC+msdn),
which is a [WindowProc callback function](https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms633573(v=vs.85)):
```c
LRESULT CALLBACK WindowProc(
  _In_ HWND   hwnd,
  _In_ UINT   uMsg,
  _In_ WPARAM wParam,
  _In_ LPARAM lParam
);
```

Oh, no, we're back to the weird stuff again!

Really, it's not too bad.
We do need a few hints though:
* `_In_` is just a note on the intended usage of that function argument.
  It's a C macro which gets replaced with nothing later on, so it's basically a code comment.
  These arguments move data "in" to the function.
  Sometimes there's "out" arguments,
  or even "in-out" arguments.
  We'll worry about those later.
* `CALLBACK` is a C macro that gets replaced with the "callback" ABI attribute.
  In this case, that's `__stdcall`.
  How do I know that?
  Well, I had to look directly in the windows include files.
  Unfortunate, but occasionally necessary.
  If you have visual studio installed, it should be in something like `C:\Program Files (x86)\Windows Kits\10\Include\10.0.16299.0`.
  Then I just did a grep to look for `CALLBACK` and looked around.
  Lots of false hits, but the only one where `CALLBACK` gets defined as a function attribute is
  `127:#define CALLBACK    __stdcall`, so that's our winner.
  (NOTE: later on I found that `CALLBACK` is discussed on the
  [Windows Data Types](https://docs.microsoft.com/en-us/windows/win32/winprog/windows-data-types) page,
  so it's much less mysterious than I thought at first.
  Still, it's good to have a note on where to find the headers,
  so I'm leaving this bit in here.)

Alright, get that junk out of the way and what do we see?
```c
LRESULT WindowProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam);
```
Oh, hey, *we can almost read that*.
It helps to remember that C puts the function output type to the left of the function's name,
and also the function argument types are to the left of each argument name.
When we think back to how strut fields were declared,
this is all fairly consistent.

The final *very* important thing to know is that C function pointers are nullable,
while Rust `fn` pointers are always non-null.
If we want to have a nullable value on the Rust side,
we have to use `Option<fn()>` instead of just `fn()`.

So let's finally add that `WNDPROC` definition:
```rust
type WNDPROC = Option<
  unsafe extern "system" fn(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
  ) -> LRESULT,
>;
```

VS Code says we're at 12 errors. Not so bad.

### Void Pointers

Now that we understand what we're supposed to be doing,
it's just a matter of filling in definition after definition until all the errors go away.
A lot of them are over on that [Windows Data Types](https://docs.microsoft.com/en-us/windows/win32/winprog/windows-data-types) page,
so we don't even have to look too many places.

Next up is `HINSTANCE`:
```
HINSTANCE: A handle to an instance. This is the base address of the module in memory.

HMODULE and HINSTANCE are the same today, but represented different things in 16-bit Windows.

This type is declared in WinDef.h as follows:

typedef HANDLE HINSTANCE;
```

So
```rust
type HINSTANCE = HANDLE;
```

Next, `HANDLE`:
```
HANDLE: A handle to an object.

This type is declared in WinNT.h as follows:

typedef PVOID HANDLE;
```

This is where it gets interesting, because now we need to have `PVOID`:
```
PVOID: A pointer to any type.

This type is declared in WinNT.h as follows:

typedef void *PVOID;
```

Remember that the `*` after the type makes it a pointer variant of the type.
It also has the `P` prefix we saw before.

The `void` type name in C performs a sort of double duty,
but in Rust we actually don't see it very often.
* When `void` is used as a *return type* it means that there's no return value from a function.
  In Rust we instead use the `()` type for functions that return nothing.
* When `void` is used as a *pointer target type* it means that the pointer points to just some opaque memory.
  In Rust, we don't really care for mysterious opaque memory,
  and we have generics,
  so we essentially never end up using void pointers.

Because the `void*` type (and the `const void *` type) are the special memory handling types in C,
LLVM has particular knowledge and opinions about how they work.
To ensure that Rust has the correct type mapping for void pointers,
there's a [c_void](https://doc.rust-lang.org/core/ffi/enum.c_void.html) type provided in the standard library.

```rust
type PVOID = *mut core::ffi::c_void;
```

### Pointer Sized Types

As we proceed down the list of errors,
filling them in one at a time,
things are fairly simple based on what we know to do so far,
and we get this:
```rust
type HICON = HANDLE;
type HCURSOR = HICON;
type HBRUSH = HANDLE;
type LPCWSTR = *const WCHAR;
type WCHAR = wchar_t;
type wchar_t = u16;
type HWND = HANDLE;
type WPARAM = UINT_PTR;
```

Then we get to `UINT_PTR`, which has a slightly funny description:
```
UINT_PTR: An unsigned INT_PTR.

This type is declared in BaseTsd.h as follows:

// C++
#if defined(_WIN64)
 typedef unsigned __int64 UINT_PTR;
#else
 typedef unsigned int UINT_PTR;
#endif
```

Hmm, a little confusing.
So far the types haven't cared about the architecture size.
Maybe something is up.
Let's see what `INT_PTR` says:
```
INT_PTR	
A signed integer type for pointer precision. Use when casting a pointer to an integer to perform pointer arithmetic.

This type is declared in BaseTsd.h as follows:

// C++
#if defined(_WIN64) 
 typedef __int64 INT_PTR; 
#else 
 typedef int INT_PTR;
#endif
```

Ah ha, so `INT_PTR` is the signed integer type used for *pointer arithmetic*,
and `UINT_PTR` is the unsigned version of course.
Well, if they're for pointer math, that's why they care about the size of a pointer.
If you know your Rust types then you already know what we need to use.
That's right, `isize` and `usize`.
They're naturally the size of a pointer, and there's the signed and unsigned variants and everything.

And now we can finally get no errors with our struct declaration!
```rust
type c_int = i32;
type c_uint = u32;
type HANDLE = PVOID;
type HBRUSH = HANDLE;
type HCURSOR = HICON;
type HICON = HANDLE;
type HINSTANCE = HANDLE;
type HWND = HANDLE;
type LONG_PTR = isize;
type LPARAM = LONG_PTR;
type LPCWSTR = *const WCHAR;
type LRESULT = LONG_PTR;
type PVOID = *mut core::ffi::c_void;
type UINT = c_uint;
type UINT_PTR = usize;
type WCHAR = wchar_t;
type wchar_t = u16;
type WPARAM = UINT_PTR;

type WNDPROC = Option<
  unsafe extern "system" fn(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
  ) -> LRESULT,
>;

#[repr(C)]
pub struct WNDCLASSW {
  style: UINT,
  lpfnWndProc: WNDPROC,
  cbClsExtra: c_int,
  cbWndExtra: c_int,
  hInstance: HINSTANCE,
  hIcon: HICON,
  hCursor: HCURSOR,
  hbrBackground: HBRUSH,
  lpszMenuName: LPCWSTR,
  lpszClassName: LPCWSTR,
}
```

Phew.

## Continuing The Windowing Tutorial

I don't know if you recall,
but like a decade ago when this article started we had a [windowing tutorial](https://docs.microsoft.com/en-us/windows/win32/learnwin32/creating-a-window)
that we were working on.

### Making a `WNDCLASSW` value

It says that we need to fill in the window procedure, the hinstance, and the class name.
The other stuff is optional, but those are essential.

In the sample C++ code, we see this interesting line:
```cpp
WNDCLASS wc = { };
```
That's a little odd looking, it might not be obvious what's happening.
It's declaring a variable `wc`, of type `WNDCLASS`, and then zeroing the entire struct.
Keeping in mind that `WNDCLASS` is an alias for either `WNDCLASSA` or `WNDCLASSW`,
depending on how you're building the C++ program,
and also keeping in mind that we're always going to be using the `W` versions of things,
then the equivalent Rust would be something like this:
```rust
let mut wc: WNDCLASSW = unsafe { core::mem::zeroed() };
```
We haven't even called the OS and we've already got `unsafe` stuff going on.

But... does this need to be `unsafe` that everyone thinks about?
Is this the kind of unsafe action that we need to evaluate the correctness of every type we do it?
No, not at all.
It's always safe to make a default `WNDCLASSW` by zeroing the memory.
We know that right now, and that doesn't change based on the situation.
So we'll just give a `Default` impl to our type that does this for us.

```rust
impl Default for WNDCLASSW {
  #[inline]
  #[must_use]
  fn default() -> Self {
    unsafe { core::mem::zeroed() }
  }
}
```
In fact, this is going to be true for all the foreign C structs we declare.
We'll just make a macro to handle this for us consistently.
When you're making a lot of bindings by hand, consistency is king.

```rust
macro_rules! unsafe_impl_default_zeroed {
  ($t:ty) => {
    impl Default for $t {
      #[inline]
      #[must_use]
      fn default() -> Self {
        unsafe { core::mem::zeroed() }
      }
    }
  };
}
```
"Lokathor, why did you put unsafe in that macro name? Default isn't an unsafe trait."
Good question.
It's because the macro _could_ be used improperly.
The `unsafe` block around the call to `zeroed` tells the compiler "no, hush, it's fine, I checked."
So if you were to use the macro to make a `Default` impl for a type that can't be safely zeroed,
then you'd sure have a problem on your hand.

Any time a macro hides away some sort of unsafe thing, you should put unsafe in the name.
It's a simple convention, but it keeps it obvious that the macro can go wrong if misused.

Now our rust can look like this:
```rust
let mut wc = WNDCLASSW::default();
```
And that's so much nicer, at least to my eyes.

### Writing a Window Procedure

The guide says

> We'll examine the window procedure in detail later. For now, just treat this as a forward reference.

So, for now we'll just make a dummy window procedure that panics if it's actually called.

```rust
unsafe extern "system" fn dummy_window_procedure(
  hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> LRESULT {
  unimplemented!()
}
```

And we can start filling in the `wc` value:
```rust
fn main() {
  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(dummy_window_procedure);
  wc.hInstance = todo!();
  wc.lpszClassName = todo!();
}
```

### Getting the HINSTANCE

This next part is a hair tricky to solve on your own.

What the tutorial wants us to do is pass the `hInstance` value that we were given at the start of the `WinMain` function.
Except the problem is that we're not writing a Windows C++ program so we don't have a WinMain function at all.
We're writing a Rust program, and the Rust program starts at `fn main()`, with no instance argument.

If we just ask the internet about ["msdn get my instance"](https://duckduckgo.com/?q=msdn+get+my+instance)
then there's not too much help.
However, if we phrase it more like ["msdn get my hinstance c++"](https://duckduckgo.com/?q=msdn+get+my+hinstance+c%2B%2B)
then there's a lovely [StackOverflow](https://stackoverflow.com/questions/1749972/determine-the-current-hinstance)
asking about this very situation.
If we call `GetModuleHandle(NULL)` we can get the `HINSTANCE` of our exe.

Interestingly, one of the comments on the question also says that we can just plain pass `NULL` as our instance value and it'll be fine.
However, the MSDN tutorial says to pass an `HINSTANCE`,
and this pushes us to learn a bit and try a new thing,
so we'll at least try the `GetModuleHandle` way first.

If we look up `GetModuleHandle`, we see that it has an `A`-form and `W`-form, since it takes a name, and the name is textual.
We want to use [GetModuleHandleW](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew), as discussed.

> If this parameter is NULL, GetModuleHandle returns a handle to the file used to create the calling process (.exe file).

Sounds good.

```rust
fn main() {
  let hInstance = GetModuleHandleW(core::ptr::null());

  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(dummy_window_procedure);
  wc.hInstance = hInstance;
  wc.lpszClassName = todo!();
}
```
Well, obviously this won't work, but let's check that error message for fun:
```
D:\dev\triangle-from-scratch>cargo run
   Compiling triangle-from-scratch v0.1.0 (D:\dev\triangle-from-scratch)
error[E0425]: cannot find function, tuple struct or tuple variant `GetModuleHandleW` in this scope
  --> src\main.rs:18:19
   |
18 |   let hInstance = GetModuleHandleW(core::ptr::null());
   |                   ^^^^^^^^^^^^^^^^ not found in this scope
```

Okay, so we need to declare the function before we can use it.
We do this with an [external block](https://doc.rust-lang.org/reference/items/external-blocks.html).

An external block just declares the signature of a function, like this:
```rust
extern ABI {
  fn NAME1(args) -> output;
  
  fn NAME2(args) -> output;

  // ...
}
```

The actual function is "external" to the program.
To perform compilation, all the compiler really needs is the correct function signature.
This allows it to perform type checking, and ensure the correct call ABI is used.
Later on, the linker sorts it all out.
If it turns out that a function can't be linked after all,
you get a link error rather than a compile error.

But who tells the linker what to link with to find the external functions?
Well, you can use a build script, or you can put it right on the extern block.

```rust
#[link(name = "LibraryName")]
extern ABI {
  fn NAME1(args) -> output;
}
```

If the library is some sort of common system library that the linker will already know about,
then it's perfectly fine to just use the attribute.
In other cases, like if a library name varies by operating system, you might need the build script.

Where do we find `GetModuleHandleW` though?
MSDN tells us right there on the page.
If we look in the **Requirements** section we'll see:

> DLL:	Kernel32.dll

So in our Rust we have our declaration like this:
```rust
#[link(name = "Kernel32")]
extern "system" {
  /// [`GetModuleHandleW`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew)
  pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
}
```

And now we can call `GetModuleHandleW` without error (if we put an `unsafe` block around the call):
```rust
fn main() {
  let hInstance = unsafe { GetModuleHandleW(core::ptr::null()) };

  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(dummy_window_procedure);
  wc.hInstance = hInstance;
  wc.lpszClassName = todo!();
}
```

### Wide Strings

The last thing we need is one of those fancy `LPCWSTR` things.
A "long pointer to a C-style wide string".
Well a long pointer is just a pointer.
And a wide string, to Windows, means a UTF-16 string.
The only thing we haven't mentioned yet is the C-style thing.

There's two basic ways to handle strings.
* "Null terminated", where the string is just a pointer, but it isn't allowed to contain 0.
  To determine the string's length you have to walk the string until you see a 0, and that's the end of the string.
* "ptr+len", where the string is a pointer and a length, and the string can contain any value.
  To determine the length, you just check the length value.

Rust uses the ptr+len style for strings, as well as for slices in general.
C and C++ use the null terminated style for strings.

It's not *too* difficult to convert a ptr+len string into a null terminated string,
but it's also not entirely free.
Pushing an extra 0 onto the end of the string is only cheap if there's spare capacity to do it.
In the case of string literals, for example,
you'd have to allocate a separate string, because the literal is kept in read-only memory.

The basic form of this is very simple code:
```rust
/// Turns a Rust string slice into a null-terminated utf-16 vector.
pub fn wide_null(s: &str) -> Vec<u16> {
  s.encode_utf16().chain(Some(0)).collect()
}
```
The `.encode_utf16()` makes the basic encoding iterator,
then `.chain(Some(0))` puts a 0 on the end of the iteration,
and we just `.collect()` it into a totally normal `Vec<u16>`.

Long term, if we were using a lot of UTF-16,
we might want to build a way to have these "C wide strings" computed as compile time and stored as literals.
It lets the program allocate a little less as it performs its startup stuff.
However, the code for that is a little hairy, and a bit of a side story compared to the current goal.

Soooo.... we can just write it like this, right?
```rust
fn main() {
  let hInstance = unsafe { GetModuleHandleW(core::ptr::null()) };

  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(dummy_window_procedure);
  wc.hInstance = hInstance;
  // BAD, WRONG, NO
  wc.lpszClassName = wide_null("Sample Window Class").as_ptr();
}
```
Ah, we can't do that!
This is a classic beginner's mistake, but it must be avoided.

If we wrote it like that, the vec of utf-16 would get allocated, then we'd call `as_ptr`,
assign that pointer to `wc.lpszClassName`, and then... the expression would end.
And the vector would drop, and clean up, and deallocate the memory we wanted to point to.
We'd have a dangling pointer, horrible.
Maybe it'd even sometimes work anyway.
The allocator might not re-use the memory right away, so it might still hold useful data for a while.
It's still some nasty Undefined Behavior though.

Here's the correct way to do it:
```rust
fn main() {
  let hInstance = unsafe { GetModuleHandleW(core::ptr::null()) };
  let sample_window_class_wn = wide_null("Sample Window Class");

  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(dummy_window_procedure);
  wc.hInstance = hInstance;
  wc.lpszClassName = sample_window_class_wn.as_ptr();
}
```

This way, the `sample_window_class_wn` binding holds the vector live,
and the pointer can be used for as long as that binding lasts.
In this case, to the end of the `main` function.

### Registering The Window Class

Okay, so our widow class request is all filled out, we just have to register it using [RegisterClassW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassw):
```cpp
ATOM RegisterClassW(
  const WNDCLASSW *lpWndClass
);
```

And in Rust:
```rust
type ATOM = WORD;
type WORD = c_ushort;
type c_ushort = u16;
#[link(name = "User32")]
extern "system" {
  /// [`RegisterClassW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassw)
  pub fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> ATOM;
}
```
It's a little weird sometimes to see that the `const` and `*` part are "around" the target type in C,
and then both on the same side of the type in Rust,
but that's genuinely the correct translation.

So now we can make the register call:
```rust
fn main() {
  let hInstance = unsafe { GetModuleHandleW(core::ptr::null()) };
  let sample_window_class_wn = wide_null("Sample Window Class");

  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(dummy_window_procedure);
  wc.hInstance = hInstance;
  wc.lpszClassName = sample_window_class_wn.as_ptr();

  unsafe { RegisterClassW(&wc) };
}
```

But we don't know if it worked or not.
Almost any call to the operating system can fail.
Cosmic rays and stuff.
If we check the **Return value** part of the MSDN page it says:

> If the function fails, the return value is zero. To get extended error information, call `GetLastError`.

Hmm, let's check [GetLastError](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror),
that sounds like a thing we'll want to use a lot.

yada yada... thead local error code...
yada yada... some functions set an error code and *then* succeed...
okay... "To obtain an error string for system error codes, use the [FormatMessage](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessage) function."
Oof, we'd have a whole extra layer to dive into if we went down that path.
"For a complete list of error codes provided by the operating system, see [System Error Codes](https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes)."
Okay, well that's not too bad.
For now, we can show an error code and then look it up by hand.

```rust
type DWORD = c_ulong;
type c_ulong = u32;
#[link(name = "Kernel32")]
extern "system" {
  /// [`GetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror)
  pub fn GetLastError() -> DWORD;
}
```

And now we have basic error checking / reporting:
```rust
fn main() {
  let hInstance = unsafe { GetModuleHandleW(core::ptr::null()) };
  let sample_window_class_wn = wide_null("Sample Window Class");

  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(dummy_window_procedure);
  wc.hInstance = hInstance;
  wc.lpszClassName = sample_window_class_wn.as_ptr();

  let atom = unsafe { RegisterClassW(&wc) };
  if atom == 0 {
    let last_error = unsafe { GetLastError() };
    panic!("Could not register the window class, error code: {}", last_error);
  }
}
```

## Creating The Window

VS Code says I'm at like 4500 words already, and we haven't even made our Window yet.

> To create a new instance of a window, call the [CreateWindowEx](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw) function:

Okay, sure, that'll be nice and easy, no proble--
```cpp
HWND CreateWindowExW(
  DWORD     dwExStyle,
  LPCWSTR   lpClassName,
  LPCWSTR   lpWindowName,
  DWORD     dwStyle,
  int       X,
  int       Y,
  int       nWidth,
  int       nHeight,
  HWND      hWndParent,
  HMENU     hMenu,
  HINSTANCE hInstance,
  LPVOID    lpParam
);
```
oof!

Okay, actually most of these we've seen before.
This is getting easier the more we do.

```rust
type HMENU = HANDLE;
type LPVOID = *mut core::ffi::c_void;
#[link(name = "User32")]
extern "system" {
  /// [`CreateWindowExW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw)
  pub fn CreateWindowExW(
    dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR,
    dwStyle: DWORD, X: c_int, Y: c_int, nWidth: c_int, nHeight: c_int,
    hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID,
  ) -> HWND;
}
```

> `CreateWindowEx` returns a handle to the new window, or zero if the function fails.
> To show the window—that is, make the window visible —pass the window handle to the
> [ShowWindow](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow) function

Hey, look, the MSDN docs are using some of that extended typography we mentioned before.

Apparently we want our window creation to look something like this:
```rust
let sample_window_name_wn = wide_null("Sample Window Name");
let hwnd = unsafe {
  CreateWindowExW(
    0,
    sample_window_class_wn.as_ptr(),
    sample_window_name_wn.as_ptr(),
    WS_OVERLAPPEDWINDOW,
    CW_USEDEFAULT,
    CW_USEDEFAULT,
    CW_USEDEFAULT,
    CW_USEDEFAULT,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    hInstance,
    core::ptr::null_mut(),
  )
};
```

Now we just have to define `WS_OVERLAPPEDWINDOW` and `CW_USEDEFAULT`.
These are defined in the header files as C macro values, which expand to literals.
In Rust, we *could* define them as macros, but it'd be a little silly.
We probably want to define them as `const` values instead.

```rust
const WS_OVERLAPPED: u32 = 0x00000000;
const WS_CAPTION: u32 = 0x00C00000;
const WS_SYSMENU: u32 = 0x00080000;
const WS_THICKFRAME: u32 = 0x00040000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_MAXIMIZEBOX: u32 = 0x00010000;
const WS_OVERLAPPEDWINDOW: u32 = WS_OVERLAPPED
  | WS_CAPTION
  | WS_SYSMENU
  | WS_THICKFRAME
  | WS_MINIMIZEBOX
  | WS_MAXIMIZEBOX;
const CW_USEDEFAULT: c_int = 0x80000000_u32 as c_int;
```
There's more `WS_` values you could define, but that's enough to start.

Oh, and heck, we probably want to just import `null` and `null_mut` since we'll be using them a lot.
```rust
use core::ptr::{null, null_mut};
```

For calling `ShowWindow`, we have a `HWND` already,
but the show parameter is apparently another one of those WinMain arguments.
Instead we'll just look at the list of what the [ShowWindow](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow)
docs say, and I guess we can pick `SW_SHOW`.

```rust
const SW_SHOW: c_int = 5;
type BOOL = c_int;
#[link(name = "User32")]
extern "system" {
  /// [`ShowWindow`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow)
  pub fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> BOOL;
}
```

Okay, now we can at least make the window and the program will close.
We expect it to like, flicker on screen really fast and then disappear, or something.
```
D:\dev\triangle-from-scratch>cargo run
   Compiling triangle-from-scratch v0.1.0 (D:\dev\triangle-from-scratch)
    Finished dev [unoptimized + debuginfo] target(s) in 0.60s
     Running `target\debug\triangle-from-scratch.exe`
thread 'main' panicked at 'not implemented', src\main.rs:60:3
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\debug\triangle-from-scratch.exe` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)
```
Whoops!
Haha, remember how we had that dummy window procedure?
It's actually *not* supposed to panic and unwind the stack during the callback.
Bad things end up happening.
We just did it to fill in a little bit so the compiler would be cool.

Now that we're tying to turn on the program on for real (even for a second),
we need a real window procedure.
But we don't know how to write one yet.
Never fear, there's a [DefWindowProcW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-defwindowprocw)
function that you can use to handle any messages you don't know how to handle.
Right now, for us, that's all of them.

```rust
fn main() {
  let hInstance = unsafe { GetModuleHandleW(null()) };
  let sample_window_class_wn = wide_null("Sample Window Class");

  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(DefWindowProcW);
  wc.hInstance = hInstance;
  wc.lpszClassName = sample_window_class_wn.as_ptr();
  // ...
}

#[link(name = "User32")]
extern "system" {
  /// [`DefWindowProcW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-defwindowprocw)
  pub fn DefWindowProcW(
    hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM,
  ) -> LRESULT;
}
```

And, finally, we can get a window to flicker on the screen!

## Handling Window Messages

We're on to the next page of the tutorial!

Now we get to learn all about [Window Messages](https://docs.microsoft.com/en-us/windows/win32/learnwin32/window-messages)

First we need to define this great `MSG` struct:
```cpp
typedef struct tagMSG {
  HWND   hwnd;
  UINT   message;
  WPARAM wParam;
  LPARAM lParam;
  DWORD  time;
  POINT  pt;
  DWORD  lPrivate;
} MSG, *PMSG, *NPMSG, *LPMSG;
```

In Rust:
```rust
#[repr(C)]
pub struct MSG {
  hwnd: HWND,
  message: UINT,
  wParam: WPARAM,
  lParam: LPARAM,
  time: DWORD,
  pt: POINT,
  lPrivate: DWORD,
}
unsafe_impl_default_zeroed!(MSG);
```
Hey look, we have nearly all of that defined already.

```rust
type LONG = c_long;
type c_long = i32;
#[repr(C)]
pub struct POINT {
  x: LONG,
  y: LONG,
}
unsafe_impl_default_zeroed!(POINT);
```

And now we can get our window messages.
```rust
#[link(name = "User32")]
extern "system" {
  /// [`GetMessageW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew)
  pub fn GetMessageW(
    lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT,
  ) -> BOOL;
}
```

We have to get them in a loop of course, because we'll be getting a whole lot of them.
```rust
let mut msg = MSG::default();
loop {
  let message_return = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
  if message_return == 0 {
    break;
  } else if message_return == -1 {
    let last_error = unsafe { GetLastError() };
    panic!("Error with `GetMessageW`, error code: {}", last_error);
  }
}
```

Except we're missing [TranslateMessage](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translatemessage)
and [DispatchMessageW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-dispatchmessagew).
```rust
#[link(name = "User32")]
extern "system" {
  /// [`TranslateMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translatemessage)
  pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;

  /// [`DispatchMessageW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-dispatchmessagew)
  pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
}
```

There's a lot of good info on the page about window messages,
but that's all we have to do here in terms of our code.

By now, our program can open a window.
However, we also see an eternally spinning mouse once it's open.
We also *don't* see our program close when we close the window.
It just continues to spin in the `loop`,
and we have to exit it by pressing Ctrl+C in the command line.

## Writing The Window Procedure

Next up is [Writing the Window Procedure](https://docs.microsoft.com/en-us/windows/win32/learnwin32/writing-the-window-procedure).

That default window procedure we've been using so far is fine for most events.
Usually it just ignores every event.
However, a few event types can't just be ignored.
One of them is that window closing situation.
Another is that thing with the mouse cursor.

If we look at MSDN page for the [WM_CLOSE](https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-close)
message, we can see that we'll need to be able to use [DestroyWindow](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)
and [PostQuitMessage](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage).

```rust
#[link(name = "User32")]
extern "system" {
  /// [`DestroyWindow`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)
  pub fn DestroyWindow(hWnd: HWND) -> BOOL;

  /// [`PostQuitMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage)
  pub fn PostQuitMessage(nExitCode: c_int);
}
```

And we have to write our own procedure.
This time, no panics.
```rust
pub unsafe extern "system" fn window_procedure(
  hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> LRESULT {
  0
}
```

There's a few different ways we can arrange the branching here,
and it comes down to taste in the end,
but most of the messages should return 0 when you've processed them.
We'll assume that 0 is the "normal" response and build our setup around that.

```rust
pub unsafe extern "system" fn window_procedure(
  hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> LRESULT {
  match Msg {
    WM_CLOSE => DestroyWindow(hWnd),
    WM_DESTROY => PostQuitMessage(0),
    _ => return DefWindowProcW(hWnd, Msg, wParam, lParam),
  }
  0
}
```
One little problem here is that `DestroyWindow` and `PostQuitMessage` have different return types.
Even though we're ignoring the output of `DestroyWindow`, it's a type error to have it like this.

```rust
pub unsafe extern "system" fn window_procedure(
  hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> LRESULT {
  match Msg {
    WM_CLOSE => {
      DestroyWindow(hWnd);
    }
    WM_DESTROY => PostQuitMessage(0),
    _ => return DefWindowProcW(hWnd, Msg, wParam, lParam),
  }
  0
}
```
Ehhhhhh, I'm not sure if I'm a fan of rustfmt making it look like that.

```rust
pub unsafe extern "system" fn window_procedure(
  hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> LRESULT {
  match Msg {
    WM_CLOSE => drop(DestroyWindow(hWnd)),
    WM_DESTROY => PostQuitMessage(0),
    _ => return DefWindowProcW(hWnd, Msg, wParam, lParam),
  }
  0
}
```
Oh, yeah, that's the good stuff.
We can use `drop` to throw away the `i32` value,
so we don't need the `;` and braces,
so rustfmt keeps it on a single line.
I am *all about* that compact code stuff.

Now we can open the window and click for it to close and the program actually terminates.

## Fixing The Cursor (maybe?)

The mouse cursor is still kinda funky.
It gets kinda... *stuck* with different icons.
If you move the mouse into the window area from different sides,
the little "adjust window size" cursors don't change to the normal cursor once the mouse is in the middle of the window.
That's mostly our fault, we left the cursor for our Window Class as null.

Instead, if we use [LoadCursorW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadcursorw)
we can assign a cursor to our window class,
and then the default window procedure will set the cursor to be the right image at the right time.

We're supposed to call it with something like:
```rust
wc.hCursor = unsafe { LoadCursorW(hInstance, IDC_ARROW) };
```

And the extern function is easy to do:
```rust
#[link(name = "User32")]
extern "system" {
  /// [`LoadCursorW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadcursorw)
  pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> HCURSOR;
}
```

But how do we make that `IDC_ARROW` thing?
In the docs for [LoadCursorW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadcursorw)
they're all listed as `MAKEINTRESOURCE(number)`.
Okay so we look up [MAKEINTRESOURCEW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-makeintresourcew)
and... it's a C macro. Blast.
Okay so we can't link to it and call it,
instead we'll have to grep the windows includes to see what's happening.
```
C:\Program Files (x86)\Windows Kits\10\Include>rg "#define MAKEINTRESOURCE"
10.0.16299.0\um\WinUser.h
215:#define MAKEINTRESOURCEA(i) ((LPSTR)((ULONG_PTR)((WORD)(i))))
216:#define MAKEINTRESOURCEW(i) ((LPWSTR)((ULONG_PTR)((WORD)(i))))
218:#define MAKEINTRESOURCE  MAKEINTRESOURCEW
220:#define MAKEINTRESOURCE  MAKEINTRESOURCEA

10.0.16299.0\shared\ks.h
4464:#define MAKEINTRESOURCE( res ) ((ULONG_PTR) (USHORT) res)
```
Hm, so.. the input value is cast to a `WORD`,
then cast directly to a `ULONG_PTR`,
then cast directly to a string pointer (either ansi or wide).
That's not too hard at all.

We *could* do this as a Rust macro,
but I feel like we might want to use a `const fn` instead.
I just like having the types be a little more checked when possible.
```rust
type LPWSTR = *mut WCHAR;
type ULONG_PTR = usize;
/// [`MAKEINTRESOURCEW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-makeintresourcew)
pub const fn MAKEINTRESOURCEW(i: WORD) -> LPWSTR {
  i as ULONG_PTR as LPWSTR
}
const IDC_ARROW: LPCWSTR = MAKEINTRESOURCEW(32512);
```

Did that fix it?
Huh. Nope.

Sometimes you're really sure that you know what's wrong,
it's just *so* obvious,
but you didn't know at all,
and you basically wasted your time with some dead end.
This is frustrating, but it's okay.
We all make mistakes.

In some web searches about this problem,
I've heard that the mouse cursor sometimes is heuristic in Windows,
and once Windows thinks that your app is behaving properly,
it'll make the mouse cursor so the right thing for you.
I don't have any idea if this is true or not.
If it is true, then maybe once we add more to our program the mouse cursor will start working right.
Let's move on, we can come back to this later.

## Painting The Window

The tutorial wants to tell us about [Painting The Window](https://docs.microsoft.com/en-us/windows/win32/learnwin32/painting-the-window) next.

So we have to accept a `WM_PAINT` message:
```rust
const WM_PAINT: u32 = 0x000F;
```
And then do a little dance with a `PAINTSTRUCT`,
as well as the additional types it depends on:
```rust
#[repr(C)]
pub struct PAINTSTRUCT {
  hdc: HDC,
  fErase: BOOL,
  rcPaint: RECT,
  fRestore: BOOL,
  fIncUpdate: BOOL,
  rgbReserved: [BYTE; 32],
}
unsafe_impl_default_zeroed!(PAINTSTRUCT);
type HDC = HANDLE;
type BYTE = u8;
#[repr(C)]
pub struct RECT {
  left: LONG,
  top: LONG,
  right: LONG,
  bottom: LONG,
}
unsafe_impl_default_zeroed!(RECT);
```
This is all becoming routine by now, I hope.

They want us to use [BeginPaint](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-beginpaint),
then [FillRect](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-fillrect) on the whole canvas,
then [EndPaint](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-endpaint).
Sounds easy enough to do.

```rust
#[link(name = "User32")]
extern "system" {
  /// [`BeginPaint`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-beginpaint)
  pub fn BeginPaint(hWnd: HWND, lpPaint: LPPAINTSTRUCT) -> HDC;

  /// [`FillRect`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-fillrect)
  pub fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> c_int;

  /// [`EndPaint`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-endpaint)
  pub fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
}
const COLOR_WINDOW: u32 = 5;
```
The `COLOR_WINDOW` constant I had to look up in the headers.

Now we adjust the window procedure a bit to do the painting:
```rust
pub unsafe extern "system" fn window_procedure(
  hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> LRESULT {
  match Msg {
    WM_CLOSE => drop(DestroyWindow(hWnd)),
    WM_DESTROY => PostQuitMessage(0),
    WM_PAINT => {
      let mut ps = PAINTSTRUCT::default();
      let hdc = BeginPaint(hWnd, &mut ps);
      let _success = FillRect(hdc, &ps.rcPaint, (COLOR_WINDOW + 1) as HBRUSH);
      EndPaint(hWnd, &ps);
    }
    _ => return DefWindowProcW(hWnd, Msg, wParam, lParam),
  }
  0
}
```

Window looks the same as before,
but if we fiddle with the brush value we can see it'll draw using other colors.
Doesn't seem to fix the mouse though.

## Closing The Window

The tutorial page about [Closing The Window](https://docs.microsoft.com/en-us/windows/win32/learnwin32/closing-the-window)
has a fun part where we can open a message box.
I like the occasional message box, let's do that.

```rust
#[link(name = "User32")]
extern "system" {
  /// [`MessageBoxW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messageboxw)
  pub fn MessageBoxW(
    hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT,
  ) -> c_int;
}
const MB_OKCANCEL: u32 = 1;
const IDOK: c_int = 1;
```

So here's a fun question, how do we decide when a constant should be `u32` or `c_int` or whatever type of int?
The correct answer is that there's no correct answer.
I just pick based on how I see the API using it most of the time.
That is, if it's a `const` that gets compared to a return from a function,
we use the function's return type.
If it's a `const` we pass to a function, we use the function's argument type.
Sometimes a value will be used as more than one type of number, then you'll have to just pick one.
In C the number types can just automatically convert, so they don't really care.
In Rust, that's not the case, so I just try to pick a default type for the value.
So that most of the time I can write `MY_CONST` and not `MY_CONST as _`.

## Managing Application State

Ah, we're back to a slightly tricky part of things.
In [Managing Application State](https://docs.microsoft.com/en-us/windows/win32/learnwin32/managing-application-state-)
we get some notions thrown around like "use just global variables until it gets too complicated!"
Well, no thanks MSDN. I'll go directly to the stage where there's no global variables.

First, we need to be ready to handle `WM_NCCREATE` and `WM_CREATE`:
```rust
const WM_NCCREATE: u32 = 0x0081;
const WM_CREATE: u32 = 0x0001;
```

And we check for them in our window procedure:
```rust
  match Msg {
    WM_NCCREATE => {
      println!("NC Create");
      return 1;
    }
    WM_CREATE => println!("Create"),
```

Let's see those messages print out...
```
D:\dev\triangle-from-scratch>cargo run
   Compiling triangle-from-scratch v0.1.0 (D:\dev\triangle-from-scratch)
    Finished dev [unoptimized + debuginfo] target(s) in 0.59s
     Running `target\debug\triangle-from-scratch.exe`
NC Create
thread 'main' panicked at 'Failed to create a window.', src\main.rs:53:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\debug\triangle-from-scratch.exe` (exit code: 101)
```
Nani!?
Something already went wrong.
Better check the full docs for [WM_NCCREATE](https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-nccreate).
Ah, see, it's right there.

> Return Value: If an application processes this message, it should return TRUE to continue creation of the window.
> If the application returns FALSE, the CreateWindow or CreateWindowEx function will return a NULL handle.

Okay, so far all of our messages have asked us to just *always* return 0 when the message was handled,
and this is the first message we've been handling that we had to decide to return 0 or not.
Well, right now our window creation should always proceed, so here we go:
```rust
WM_NCCREATE => {
  println!("NC Create");
  return 1;
}
```

And give this a test now:
```
D:\dev\triangle-from-scratch>cargo run
   Compiling triangle-from-scratch v0.1.0 (D:\dev\triangle-from-scratch)
    Finished dev [unoptimized + debuginfo] target(s) in 0.64s
     Running `target\debug\triangle-from-scratch.exe`
NC Create
Create
```
Naisu!

Hey, better check on [WM_CREATE](https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-create)
to see if it has any return stuff we just got right on accident:

> **Return value:** If an application processes this message, it should return zero to continue creation of the window.
> If the application returns –1, the window is destroyed and the CreateWindowEx or CreateWindow function returns a NULL handle.

Ah, yeah, we were getting it right sorta on accident.
Gotta always read those docs.

Okay now we continue the tutorial:

> The last parameter of CreateWindowEx is a pointer of type `void*`.
> You can pass any pointer value that you want in this parameter.
> When the window procedure handles the `WM_NCCREATE` or `WM_CREATE` message,
> it can extract this value from the message data.

Right, so, we have to have a void pointer to pass to the message.
Uh, just to pick something, let's pass our message a pointer to the number 5.

```rust
let lparam: *mut i32 = Box::leak(Box::new(5_i32));
let hwnd = unsafe {
  CreateWindowExW(
    0,
    sample_window_class_wn.as_ptr(),
    sample_window_name_wn.as_ptr(),
    WS_OVERLAPPEDWINDOW,
    CW_USEDEFAULT,
    CW_USEDEFAULT,
    CW_USEDEFAULT,
    CW_USEDEFAULT,
    null_mut(),
    null_mut(),
    hInstance,
    lparam.cast(),
  )
};
```

So we're making a boxed `i32`,
then we leak the box because we don't want Rust to drop this box based on scope.
Instead, we'll clean up the box as part of the window destruction.

> When you receive the `WM_NCCREATE` and `WM_CREATE` messages,
> the lParam parameter of each message is a pointer to a `CREATESTRUCT` structure.

The `CREATESTRUCT` type has `A` and `W` forms.
Since we're using `CreateWindowExW`, we'll assume that we use `CREATESTRUCTW` here.
```rust
#[repr(C)]
pub struct CREATESTRUCTW {
  lpCreateParams: LPVOID,
  hInstance: HINSTANCE,
  hMenu: HMENU,
  hwndParent: HWND,
  cy: c_int,
  cx: c_int,
  y: c_int,
  x: c_int,
  style: LONG,
  lpszName: LPCWSTR,
  lpszClass: LPCWSTR,
  dwExStyle: DWORD,
}
unsafe_impl_default_zeroed!(CREATESTRUCTW);
```

Now we can get out the boxed pointer thing from the create struct:
```rust
WM_NCCREATE => {
  println!("NC Create");
  let createstruct: *mut CREATESTRUCTW = lParam as *mut _;
  if createstruct.is_null() {
    return 0;
  }
  let boxed_i32_ptr: *mut i32 = (*createstruct).lpCreateParams.cast();
  return 1;
}
```

And then we use [SetWindowLongPtrW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw)
to connect this create struct pointer to the window itself.
```rust
#[link(name = "User32")]
extern "system" {
  /// [`SetWindowLongPtrW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw)
  pub fn SetWindowLongPtrW(
    hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR,
  ) -> LONG_PTR;
}
const GWLP_USERDATA: c_int = -21;
```
And it's fairly simple to call, but we have to put a manual cast to `LONG_PTR` in:
```rust
WM_NCCREATE => {
  println!("NC Create");
  let createstruct: *mut CREATESTRUCTW = lParam as *mut _;
  if createstruct.is_null() {
    return 0;
  }
  let boxed_i32_ptr = (*createstruct).lpCreateParams;
  SetWindowLongPtrW(hWnd, GWLP_USERDATA, boxed_i32_ptr as LONG_PTR);
  return 1;
}
```

And then we can use [GetWindowLongPtrW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowlongptrw)
to get our windows's custom user data:
```rust
#[link(name = "User32")]
extern "system" {
  /// [`GetWindowLongPtrW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowlongptrw)
  pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
}
```

Now, uh, we'll print out the current value or something.
I guess, each time we go to `WM_PAINT` we'll print the value and add 1 to it.
Just any old thing to see that it's working.

```rust
WM_PAINT => {
  let ptr = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut i32;
  println!("Current ptr: {}", *ptr);
  *ptr += 1;
  let mut ps = PAINTSTRUCT::default();
  let hdc = BeginPaint(hWnd, &mut ps);
  let _success = FillRect(hdc, &ps.rcPaint, (COLOR_WINDOW + 1) as HBRUSH);
  EndPaint(hWnd, &ps);
}
```

That'll print 5, and then if you force a bunch of paint messages you can see it count up.
The easiest way to do that is to adjust the window's size so that it's small,
then drag it to be bigger.
Each time the window's size expands it triggers new paint messages.

Of course, we also can't forget that cleanup code we promised.
The way we do the cleanup is to just turn the raw pointer back into a `Box<i32>`.
The drop code for the Box type will handle the rest for us.
Of course, we should only do this right as the window is being destroyed.
```rust
WM_DESTROY => {
  let ptr = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut i32;
  Box::from_raw(ptr);
  println!("Cleaned up the box.");
  PostQuitMessage(0);
}
```

And finally, I think we're done!

## Hey, What About The Triangle?

Well, there's several ways to draw a triangle in windows.
You can use DirectX, OpenGL, Vulkan, probably some other ways I don't even know about.
This lesson is going to stop at *just* the window creation part.
Then, each other lesson on a particular Windows drawing API can assume you've read this as a baseline level of understanding.

## Fixing That Stupid Cursor

Before I finished this, I really wanted to figure out what was going on with that cursor.

So what I did was, first we want to handle the `WM_SETCURSOR` event,
and then in the event we call the `SetCursor` function.

```rust
// ...
    WM_SETCURSOR => {
      let hInstance = GetModuleHandleW(null());
      let cursor = LoadCursorW(hInstance, IDC_ARROW);
      let _old_cursor = SetCursor(cursor);
      //
      return 1;
    }
// ...

#[link(name = "User32")]
extern "system" {
  /// [`SetCursor`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setcursor)
  pub fn SetCursor(hCursor: HCURSOR) -> HCURSOR;
}
const WM_SETCURSOR: u32 = 0x0020;
```

Alright, so, what happens if you do that?
Well, the cursor disappears entirely.
What? Why?
Well, let's check the docs for [SetCursor](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setcursor).
Hm, hmm, hmmmm, yeah.

> If this parameter is NULL, the cursor is removed from the screen.

Okay, so we must be getting null for `cursor`.
So `LoadCursorW(hInstance, IDC_ARROW)` is returning null...
and that's how we're setting `wc.hCursor` during startup!
So we must have been setting null *this whole time*.
What fools we've been.

Okay let's check out the MSDN guide on [Setting the Cursor Image](https://docs.microsoft.com/en-us/windows/win32/learnwin32/setting-the-cursor-image).
They've got some sample code:

```cpp
hCursor = LoadCursor(NULL, cursor);
SetCursor(hCursor);
```

Oh. Huh. So, you're supposed to pass null to the load call?
I guess that makes sense.
I mean we had an HINSTANCE sitting around and we just used it,
but whe you think about it,
our own executable file probably *doesn't* contain the `IDC_ARROW` cursor.

So if we delete the cursor handling code from the window procedure,
and just adjust the startup code to be correct:

```rust
let mut wc = WNDCLASSW::default();
wc.lpfnWndProc = Some(window_procedure);
wc.hInstance = hInstance;
wc.lpszClassName = sample_window_class_wn.as_ptr();
wc.hCursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
```

Well now our cursor works just fine!

The day is saved!

