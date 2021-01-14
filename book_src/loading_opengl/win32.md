
# Loading OpenGL With Win32

On Windows, turning on OpenGL requires... a few steps.

It's honestly gonna seem borderline silly when I explain it.

Basically, Microsoft would rather that you use DirectX,
the Windows-exclusive graphics card API,
and so they don't really bother to make it *easy* to turn on OpenGL.

## Documentation

In terms of documentation for opening a GL Context,
you can check out [MSDN](https://docs.microsoft.com/en-us/windows/win32/opengl/rendering-contexts)
like we've done before,
but you'll probably get a little more help from
[The OpenGL Wiki](https://www.khronos.org/opengl/wiki/Creating_an_OpenGL_Context_(WGL)).
Technically, the context creation process is fully outside the OpenGL specification.
However, they still explain how to create a context on the wiki because it's obviously a necessary step to use GL.

# Expanding the "Cleaned Up" Win32 Example

For this lesson we'll be using the "cleaned up" Win32 example as our starting point.

## Device Context

The first thing we need to do is adjust the `WNDCLASSW` value that we register.

According to the [wiki article](https://www.khronos.org/opengl/wiki/Creating_an_OpenGL_Context_(WGL)),
we need to set `CS_OWNDC` in the `style` field so that each window has its own device context:

```rust
// in our fn main()
  // ...
  let mut wc = WNDCLASSW::default();
  wc.style = CS_OWNDC; // NEW
  // ...
```

If we search for `CS_OWNDC` [in the MSDN pages](https://docs.microsoft.com/en-us/search/?scope=Desktop&terms=CS_OWNDC),
it leads us to the [Window Class Styles](https://docs.microsoft.com/en-us/windows/win32/winmsg/window-class-styles) page.
Ah, so many things to look at.
We know we want `CS_OWNDC`,
but if we glance at the other options there's mostly stuff we don't need.
Interestingly, it looks like maybe you need to enable double-click support on your window if you want it?
We don't need that now, but just something to remember if you want it later on.
There's also `CS_HREDRAW` and `CS_VREDRAW`,
which cause the full window to be redrawn if the window changes horizontal or vertical size.
That seems fairly handy, it means we just always draw everything if the window resizes,
and we don't have to think about re-drawing sub-regions.
We'll use those too.

All three new declarations can go directly into the `win32` module of our library:
```rust
// src/win32.rs

/// Allocates a unique device context for each window in the class.
pub const CS_OWNDC: u32 = 0x0020;

/// Redraws the entire window if a movement or size adjustment changes the width
/// of the client area.
pub const CS_HREDRAW: u32 = 0x0002;

/// Redraws the entire window if a movement or size adjustment changes the
/// height of the client area.
pub const CS_VREDRAW: u32 = 0x0001;
```

## Pixel Format Descriptor

Next we need to fill out a [PIXELFORMATDESCRIPTOR](https://docs.microsoft.com/en-us/windows/win32/api/Wingdi/ns-wingdi-pixelformatdescriptor).
This is our *request* for what we want the eventual pixel format of the window to be.
We can't be sure we'll get exactly what we ask for,
but if we ask for something fairly common we'll probably get it.

Looking at the MSDN page for `PIXELFORMATDESCRIPTOR`,
there's two very important fields that are different from any struct fields we've seen so far.
The `nSize` and `nVersion` fields specify the struct's size (in bytes) along with a version number.
That's a little... odd.

### Versioned Structs

Because *of course* you know what the size of the struct is, right?
You're using struct `S` from crate `foo-x.y.z`,
and in version `x.y.z` of crate `foo` the struct `S` had... whatever the fields are.
If the fields ever *were* changed,
that would have to be published as a new version of the crate,
then the compiler would know about the change when you updated the dependency.
So there's totally no reason to ever write down the size of the struct within the struct itself, right?

Nope.

This is one area where C still has Rust completely beat: binary stability.

See the folks at Microsoft want programs to *last*.
Like, they want programs to keep working *forever* if possible.
Users have their old video games,
or their old art software,
or their old whatever software,
and those users want their stuff to keep working when there's a Windows Update.
And they often *won't* update to a newer version of Windows if it breaks their stuff.
So Microsoft would really rather that all old programs keep working for as long as possible,
because they want to keep selling you new versions of Windows.
They're usually pretty good at this.

A few months ago I downloaded an old tile editor from an ancient website,
and the site said "this was tested on windows 95, it might work on windows 98".
I opened it up on my modern Windows 10 machine, and it worked.
A few of the GUI buttons looked kinda "off", visually, but it worked.
I was able to edit some tiles and export them,
despite the program being from so long ago that the author didn't know if it would work on Windows 98.
That's a stability success story right there.

On the other hand, Diablo 2 (circa 2000) doesn't run on Windows 10 without the 1.14 patch,
because the 1.0 patch level of the game can't launch the program for whatever reason.
So the world isn't always as stable as we'd like.

"Lokathor what *are* you rambling on about? `cargo` uses SemVer, it's all plenty stable!"

No friends, no, hold on, not quite.

You see `cargo` builds the whole universe from source every time.
When you use `foo-x.y.z` in your project,
`cargo` downloads the *source* for that crate with that version,
then it builds that crate version right on your own machine,
and then it builds your crate and links it all together.
This all works out when you're making just *one* program.
I mean, clean build times are huge, but whatever, right?

However, when you're asking *two* programs to communicate with each other,
obviously you need to be able to update one of the programs without updating the other program right away (or ever).
Which means that if you ever want to be able to change your message format in the future,
you need to put a version on that kind of message *now*,
so that the possibility to change the version number later is left open to you.

"Lokathor! We're only building one program! What are you even talking about!?"

No, friends, *the operating system is the other program*.

"...what?"

Yeah, the operating system isn't magic.
It's just a program.
It's a program that runs programs for you.
And manages when they get to use the CPU and RAM and stuff.
So the word "just" is doing a lot of heavy lifting when I say "it's just a program",
but ultimately that's true.

Which means that you'll often see things like the `PIXELFORMATDESCRIPTOR` struct,
where there's something that lets you say what version of the API's protocol you're intending to use when you make the API call.
The operating system looks at those signs, and does the right thing for the version you expect.

So you end up seeing this pattern a lot any time there's an interface that's intended to be stable over a long period of time.
It's used quite a bit within Win32, and it's *all over* in Vulkan, for example.

Okay, fun digression, let's get back to the task at hand.

### Defining PIXELFORMATDESCRIPTOR

In terms of the fields, the `PIXELFORMATDESCRIPTOR` type is fairly obvious:
```rust
#[repr(C)]
pub struct PIXELFORMATDESCRIPTOR {
  pub nSize: WORD,
  pub nVersion: WORD,
  pub dwFlags: DWORD,
  pub iPixelType: BYTE,
  pub cColorBits: BYTE,
  pub cRedBits: BYTE,
  pub cRedShift: BYTE,
  pub cGreenBits: BYTE,
  pub cGreenShift: BYTE,
  pub cBlueBits: BYTE,
  pub cBlueShift: BYTE,
  pub cAlphaBits: BYTE,
  pub cAlphaShift: BYTE,
  pub cAccumBits: BYTE,
  pub cAccumRedBits: BYTE,
  pub cAccumGreenBits: BYTE,
  pub cAccumBlueBits: BYTE,
  pub cAccumAlphaBits: BYTE,
  pub cDepthBits: BYTE,
  pub cStencilBits: BYTE,
  pub cAuxBuffers: BYTE,
  pub iLayerType: BYTE,
  pub bReserved: BYTE,
  pub dwLayerMask: DWORD,
  pub dwVisibleMask: DWORD,
  pub dwDamageMask: DWORD,
}
```

However, since we want the type to *always* say it's size and version,
we'll have a `Default` impl that just sets the size and version for us.
```rust
impl Default for PIXELFORMATDESCRIPTOR {
  #[inline]
  #[must_use]
  fn default() -> Self {
    let mut out: Self = unsafe { core::mem::zeroed() };
    out.nSize = core::mem::size_of::<Self>() as WORD;
    out.nVersion = 1;
    out
  }
}
```

### Filling Our Our PIXELFORMATDESCRIPTOR

The wiki page suggests that we fill out our pixel format like this:
```c
PIXELFORMATDESCRIPTOR pfd =
{
	sizeof(PIXELFORMATDESCRIPTOR),
	1,
	PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,    // Flags
	PFD_TYPE_RGBA,        // The kind of framebuffer. RGBA or palette.
	32,                   // Colordepth of the framebuffer.
	0, 0, 0, 0, 0, 0,
	0,
	0,
	0,
	0, 0, 0, 0,
	24,                   // Number of bits for the depthbuffer
	8,                    // Number of bits for the stencilbuffer
	0,                    // Number of Aux buffers in the framebuffer.
	PFD_MAIN_PLANE,
	0,
	0, 0, 0
};
```

Well, okay, sure, let's just kinda convert that into some Rust:
```rust
// after we register the class in fn main
  let pfd = PIXELFORMATDESCRIPTOR {
    dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
    iPixelType: PFD_TYPE_RGBA,
    cColorBits: 32,
    cDepthBits: 24,
    cStencilBits: 8,
    iLayerType: PFD_MAIN_PLANE,
    ..Default::default()
  };
```

Oh that's actually pretty clear.
This time we're using the [functional update syntax](https://doc.rust-lang.org/reference/expressions/struct-expr.html#functional-update-syntax)
for struct creation.
We haven't used that before,
so I'll link the reference there for you.
We could also use it in other places,
such as filling out our `WNDCLASSW` value.
Honestly, I don't even use it myself that much,
but I thought we'd just give it a try,
and see how it feels.

Anyway, now we need to declare a bunch of consts.

Except, the `PIXELFORMATDESCRIPTOR` page on MSDN **doesn't** list the values.

Guess it's back to grepping through the windows header files.
By the way, if you're not aware,
there's a very fast grep tool written in rust called `ripgrep` you might want to try.

```
C:\Program Files (x86)\Windows Kits\10\Include\10.0.16299.0>rg "PFD_DRAW_TO_WINDOW"
um\wingdi.h
3640:#define PFD_DRAW_TO_WINDOW          0x00000004

C:\Program Files (x86)\Windows Kits\10\Include\10.0.16299.0>rg "PFD_SUPPORT_OPENGL"
um\wingdi.h
3643:#define PFD_SUPPORT_OPENGL          0x00000020
```
Looks like it's generally around line 3640 of `wingdi.h`,
so we just have a look at those defines and then do a little Rust.

```rust
/// [`PIXELFORMATDESCRIPTOR`] pixel type
pub const PFD_TYPE_RGBA: u8 = 0;
/// [`PIXELFORMATDESCRIPTOR`] pixel type
pub const PFD_TYPE_COLORINDEX: u8 = 1;

/// [`PIXELFORMATDESCRIPTOR`] layer type
pub const PFD_MAIN_PLANE: u8 = 0;
/// [`PIXELFORMATDESCRIPTOR`] layer type
pub const PFD_OVERLAY_PLANE: u8 = 1;
/// [`PIXELFORMATDESCRIPTOR`] layer type
pub const PFD_UNDERLAY_PLANE: u8 = u8::MAX /* was (-1) */;

pub const PFD_DOUBLEBUFFER: u32 = 0x00000001;
pub const PFD_STEREO: u32 = 0x00000002;
pub const PFD_DRAW_TO_WINDOW: u32 = 0x00000004;
pub const PFD_DRAW_TO_BITMAP: u32 = 0x00000008;
pub const PFD_SUPPORT_GDI: u32 = 0x00000010;
pub const PFD_SUPPORT_OPENGL: u32 = 0x00000020;
pub const PFD_GENERIC_FORMAT: u32 = 0x00000040;
pub const PFD_NEED_PALETTE: u32 = 0x00000080;
pub const PFD_NEED_SYSTEM_PALETTE: u32 = 0x00000100;
pub const PFD_SWAP_EXCHANGE: u32 = 0x00000200;
pub const PFD_SWAP_COPY: u32 = 0x00000400;
pub const PFD_SWAP_LAYER_BUFFERS: u32 = 0x00000800;
pub const PFD_GENERIC_ACCELERATED: u32 = 0x00001000;
pub const PFD_SUPPORT_DIRECTDRAW: u32 = 0x00002000;
pub const PFD_DIRECT3D_ACCELERATED: u32 = 0x00004000;
pub const PFD_SUPPORT_COMPOSITION: u32 = 0x00008000;

/// use with [`ChoosePixelFormat`] only
pub const PFD_DEPTH_DONTCARE: u32 = 0x20000000;
/// use with [`ChoosePixelFormat`] only
pub const PFD_DOUBLEBUFFER_DONTCARE: u32 = 0x40000000;
/// use with [`ChoosePixelFormat`] only
pub const PFD_STEREO_DONTCARE: u32 = 0x80000000;
```

When translating a C `#define` into a Rust declaration you've gotta use a little judgement.
In addition to looking at the raw numeric value,
you've gotta try and match the type of the const to the type of the place it gets used the most.
The pixel types and layer types get assigned to a field that's a `u8`, so we declare them as `u8`.
The flags all get combined and assigned to a field that's a `u32`, so we declare them as a `u32`.
In C it makes little difference (because C numbers will *generally* coerce automatically),
but in Rust we would have to write some casts somewhere if the types don't line up,
so we try to make our lives easy later by getting the declaration itself to have the most used type.

## ChoosePixelFormat

Okay once we have a `PIXELFORMATDESCRIPTOR` value we call [ChoosePixelFormat](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-choosepixelformat)
to get a "pixel format index" that's the closest available pixel format to our request.

First we declare the external call of course:
```rust
// note: our first use of Gdi32!
#[link(name = "Gdi32")]
extern "system" {
  /// [`ChoosePixelFormat`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-choosepixelformat)
  pub fn ChoosePixelFormat(
    hdc: HDC, ppfd: *const PIXELFORMATDESCRIPTOR,
  ) -> c_int;
}
```

Of course, this function can fail,
so we want to have a `Result` as the output type in the final version we'll use.
Instead of doing a whole thing with the raw calls,
and then doing an entire revision phase to make all the "nicer" versions after that,
we'll just jump right to the part where we make the nice version as we cover each new function.
Personally, I think that seeing the "all raw calls" version of something is a little fun once,
but once you start making things Rusty you might as well keep doing it as you go.
We won't always know exactly how we want to use each extern function the moment we first see it,
but each wrapper function is generally quite small,
so we can usually make a change if we realize something new later on.

Let's check the docs:

> If the function succeeds, the return value is a pixel format index (one-based) that is the closest match to the given pixel format descriptor.
> If the function fails, the return value is zero. To get extended error information, call GetLastError.

The docs do *not* seem to indicate that you're allowed to pass a null pointer for the pixel format descriptor,
so we can just require a reference instead of requiring a const pointer.

Okay, easy enough:
```rust
/// See [`ChoosePixelFormat`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-choosepixelformat)
pub unsafe fn choose_pixel_format(
  hdc: HDC, ppfd: &PIXELFORMATDESCRIPTOR,
) -> Result<c_int, Win32Error> {
  let index = ChoosePixelFormat(hdc, ppfd);
  if index != 0 {
    Ok(index)
  } else {
    Err(get_last_error())
  }
}
```

Can we improve it any more than this?
The only thing I can think of would be *maybe* to newtype that `c_int` value.
We could make a `PixelFormatIndex` or something if we wanted to.
I sure thought about it for a while, sitting on the bus.
But now that I'm at the keyboard, it doesn't seem very error prone even as just a `c_int`.
I think we're fine without doing that extra work.

Code you don't write at all is more valuable than code you do write.
Or, something wise sounding like that.

Alright so we're gonna set up a `PIXELFORMATDESCRIPTOR` and choose a pixel format:
```rust
  let pfd = PIXELFORMATDESCRIPTOR {
    dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
    iPixelType: PFD_TYPE_RGBA,
    cColorBits: 32,
    cDepthBits: 24,
    cStencilBits: 8,
    iLayerType: PFD_MAIN_PLANE,
    ..Default::default()
  };
  // Oops, we don't have an HDC value yet!
  let pixel_format_index = unsafe { choose_pixel_format(hdc, &pfd) }.unwrap();
```
AGH! With no HDC from anywhere we can't choose a pixel format!
Blast, and such.

### We Need An HDC

Alright since we need an HDC to choose a pixel format,
we'll move the choosing *after* we create our window,
before we show it,
and then we can get the DC for our window,
and it'll all be fine.

We'll need to use [GetDC](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdc)
to get the HDC value for our window,
and then eventually [ReleaseDC](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-releasedc)
at some point.

```rust
#[link(name = "User32")]
extern "system" {
  /// [`GetDC`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdc)
  pub fn GetDC(hWnd: HWND) -> HDC;

  /// [`ReleaseDC`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-releasedc)
  pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
}
```

Okay, now we're ready.
So we're going to get the DC for our window,
choose a pixel format,
we'll set that pixel format (see next section),
and then we're all good, right?

No.

Again, of course, this has to be more complicated than that.

If we glance back at [the wiki](https://www.khronos.org/opengl/wiki/Creating_an_OpenGL_Context_(WGL))
we'll see a section called "Proper Context Creation" with the following warning:

> **Warning:** Unfortunately, Windows does not allow the user to change the pixel format of a window.
> You get to set it exactly once.
> Therefore, if you want to use a different pixel format from the one your fake context used (for sRGB or multisample framebuffers, or just different bit-depths of buffers),
> you must destroy the window entirely and recreate it after we are finished with the dummy context.

...oookay

Sure, we'll just do *that* thing.

### Making A Fake Window

So in between registering the class and making the `i32` box,
we've got quite a bit of new middle work
```rust
// in fn main
  let _atom = unsafe { register_class(&wc) }.unwrap();

  // fake window stuff
  let pfd = PIXELFORMATDESCRIPTOR {
    dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
    iPixelType: PFD_TYPE_RGBA,
    cColorBits: 32,
    cDepthBits: 24,
    cStencilBits: 8,
    iLayerType: PFD_MAIN_PLANE,
    ..Default::default()
  };
  let fake_hwnd = unsafe {
    create_app_window(
      sample_window_class,
      "Fake Window",
      None,
      [1, 1],
      null_mut(),
    )
  }
  .unwrap();
  let fake_hdc = unsafe { GetDC(fake_hwnd) };
  let pf_index = unsafe { choose_pixel_format(fake_hdc, &pfd) }.unwrap();
  // TODO: SetPixelFormat
  assert!(unsafe { ReleaseDC(fake_hwnd, fake_hdc) } != 0);
  assert!(unsafe { DestroyWindow(fake_hwnd) } != 0);

  // real window stuff
  let lparam: *mut i32 = Box::leak(Box::new(5_i32));
```

Okay, I guess that makes sense.

Hmm, I don't like those mysterious `!= 0` parts.
Let's wrap those into our lib.
And since we're wrapping `ReleaseDC` we might as well do `GetDC` too.

```rust
/// See [`GetDC`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdc)
pub unsafe fn get_dc(hwnd: HWND) -> Option<HDC> {
  let hdc = GetDC(hwnd);
  if hdc.is_null() {
    None
  } else {
    Some(hdc)
  }
}

/// See [`ReleaseDC`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-releasedc)
#[must_use]
pub unsafe fn release_dc(hwnd: HWND, hdc: HDC) -> bool {
  let was_released = ReleaseDC(hwnd, hdc);
  was_released != 0
}

/// See [`DestroyWindow`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)
pub unsafe fn destroy_window(hwnd: HWND) -> Result<(), Win32Error> {
  let destroyed = DestroyWindow(hwnd);
  if destroyed != 0 {
    Ok(())
  } else {
    Err(get_last_error())
  }
}
```
Here the output *isn't* always a result.
Since `GetDC` doesn't have any error code to report (according to its docs), we just use `Option`.
Similarly, `ReleaseDC` doesn't seem to report an error code, so we just return a `bool`.
However, in both cases we want to encourage the caller to actually use the output,
because not checking these could lead to nasty memory leaks.
So we use the `#[must_use]` attribute to ensure that they get a warning if they don't use the output value.

Which means our program looks more like this now:
```rust
  let fake_hdc = unsafe { get_dc(fake_hwnd) }.unwrap();
  let pf_index = unsafe { choose_pixel_format(fake_hdc, &pfd) }.unwrap();
  // TODO: SetPixelFormat
  assert!(unsafe { release_dc(fake_hwnd, fake_hdc) });
  unsafe { destroy_window(fake_hwnd) }.unwrap();
```

## SetPixelFormat

Once we've chosen a pixel format, we set it to the HDC with [SetPixelFormat](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setpixelformat).
```rust
#[link(name = "Gdi32")]
extern "system" {
  /// [`SetPixelFormat`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setpixelformat)
  pub fn SetPixelFormat(
    hdc: HDC, format: c_int, ppfd: *const PIXELFORMATDESCRIPTOR,
  ) -> BOOL;
}
```

Which we wrap up a little nicer like this:
```rust
/// Sets the pixel format of an HDC.
///
/// * If it's a window's HDC then it sets the pixel format of the window.
/// * You can't set a window's pixel format more than once.
/// * Call this *before* creating an OpenGL context.
/// * OpenGL windows should use [`WS_CLIPCHILDREN`] and [`WS_CLIPSIBLINGS`]
/// * OpenGL windows should *not* use `CS_PARENTDC`
///
/// See [`SetPixelFormat`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setpixelformat)
pub unsafe fn set_pixel_format(
  hdc: HDC, format: c_int, ppfd: &PIXELFORMATDESCRIPTOR,
) -> Result<(), Win32Error> {
  let success = SetPixelFormat(hdc, format, ppfd);
  if success != 0 {
    Ok(())
  } else {
    Err(get_last_error())
  }
}
```
Oh, what's that?
Yeah we need some extra window styles.

```rust
/// Excludes the area occupied by child windows when drawing occurs within the
/// parent window.
///
/// This style is used when creating the parent window.
pub const WS_CLIPCHILDREN: u32 = 0x02000000;

/// Clips child windows relative to each other.
///
/// That is, when a particular child window receives a WM_PAINT message,
/// the WS_CLIPSIBLINGS style clips all other overlapping child windows out of
/// the region of the child window to be updated. If WS_CLIPSIBLINGS is not
/// specified and child windows overlap, it is possible, when drawing within the
/// client area of a child window, to draw within the client area of a
/// neighboring child window.
pub const WS_CLIPSIBLINGS: u32 = 0x04000000;

pub unsafe fn create_app_window() {
  // ...
  let hwnd = CreateWindowExW(
    WS_EX_APPWINDOW | WS_EX_OVERLAPPEDWINDOW,
    class_name_null.as_ptr(),
    window_name_null.as_ptr(),
    // New Style!
    WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
    x,
    y,
    width,
    height,
    null_mut(),
    null_mut(),
    get_process_handle(),
    create_param,
  );
  // ...
}
```

Okay, so now we can set the pixel format on our fake HDC:
```rust
  let fake_hdc = unsafe { get_dc(fake_hwnd) }.unwrap();
  let pf_index = unsafe { choose_pixel_format(fake_hdc, &pfd) }.unwrap();
  unsafe { set_pixel_format(fake_hdc, pf_index, &pfd) }.unwrap();
  assert!(unsafe { release_dc(fake_hwnd, fake_hdc) });
  unsafe { destroy_window(fake_hwnd) }.unwrap();
```

Hmm, wait, hold on.
So we choose a pixel format, and get an index.
How do we check if that index is close to what we wanted?
Ah, the docs say we need [DescribePixelFormat](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-describepixelformat).
This is one of those "does two things at once" functions.
When it succeeds, not only is the return code a non-zero value,
it's the maximum index of pixel formats.
So you can call the function with a null pointer to just get the maximum index.
We'll split this up into two different functions.

```rust
/// Gets the maximum pixel format index for the HDC.
///
/// Pixel format indexes are 1-based.
///
/// To print out info on all the pixel formats you'd do something like this:
/// ```no_run
/// # use triangle_from_scratch::win32::*;
/// let hdc = todo!("create a window to get an HDC");
/// let max = unsafe { get_max_pixel_format_index(hdc).unwrap() };
/// for index in 1..=max {
///   let pfd = unsafe { describe_pixel_format(hdc, index).unwrap() };
///   todo!("print the pfd info you want to know");
/// }
/// ```
///
/// See [`DescribePixelFormat`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-describepixelformat)
pub unsafe fn get_max_pixel_format_index(
  hdc: HDC,
) -> Result<c_int, Win32Error> {
  let max_index = DescribePixelFormat(
    hdc,
    1,
    size_of::<PIXELFORMATDESCRIPTOR>() as _,
    null_mut(),
  );
  if max_index == 0 {
    Err(get_last_error())
  } else {
    Ok(max_index)
  }
}

/// Gets the pixel format info for a given pixel format index.
///
/// See [`DescribePixelFormat`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-describepixelformat)
pub unsafe fn describe_pixel_format(
  hdc: HDC, format: c_int,
) -> Result<PIXELFORMATDESCRIPTOR, Win32Error> {
  let mut pfd = PIXELFORMATDESCRIPTOR::default();
  let max_index = DescribePixelFormat(
    hdc,
    format,
    size_of::<PIXELFORMATDESCRIPTOR>() as _,
    &mut pfd,
  );
  if max_index == 0 {
    Err(get_last_error())
  } else {
    Ok(pfd)
  }
}
```

So we'll print out our pixel format info when we boot the program, seems neat to know.
We just throw a `#[derive(Debug)]` on the `PIXELFORMATDESCRIPTOR` struct and add a little bit to our `main`.

```rust
  if let Ok(pfd) = unsafe { describe_pixel_format(fake_hdc, pf_index) } {
    println!("{:?}", pfd);
  } else {
    println!("Error: Couldn't get pixel format description.");
  }
```

Let's give this a try and see what pixel format info prints out:
```
D:\dev\triangle-from-scratch>cargo run
   Compiling triangle-from-scratch v0.1.0 (D:\dev\triangle-from-scratch)
    Finished dev [unoptimized + debuginfo] target(s) in 1.34s
     Running `target\debug\triangle-from-scratch.exe`
NC Create
Create
PIXELFORMATDESCRIPTOR { nSize: 40, nVersion: 1, dwFlags: 33317, iPixelType: 0, cColorBits: 32, cRedBits: 8, cRedShift: 16, cGreenBits: 8, cGreenShift: 8, cBlueBits: 8, cBlueShift: 0, cAlphaBits: 0, cAlphaShift: 0, cAccumBits: 64, cAccumRedBits: 16, cAccumGreenBits: 16, cAccumBlueBits: 16, cAccumAlphaBits: 16, cDepthBits: 24, cStencilBits: 8, cAuxBuffers: 4, iLayerType: 0, bReserved: 0, dwLayerMask: 0, dwVisibleMask: 0, dwDamageMask: 0 }
userdata ptr is null, no cleanup
NC Create
Create
```

Uh... huh? So we're seeing the info, but there's no window!
Some sort of problem has prevented the real window from showing up.
If we comment out all the "fake window" stuff the real window comes back,
so some part of that code is at fault here.

Hmm.

What if we make a fake window class to go with our fake window?

```rust
fn main() {
  // fake window stuff
  let fake_window_class = "Fake Window Class";
  let fake_window_class_wn = wide_null(fake_window_class);

  let mut fake_wc = WNDCLASSW::default();
  fake_wc.style = CS_OWNDC;
  fake_wc.lpfnWndProc = Some(DefWindowProcW);
  fake_wc.hInstance = get_process_handle();
  fake_wc.lpszClassName = fake_window_class_wn.as_ptr();

  let _atom = unsafe { register_class(&fake_wc) }.unwrap();

  let pfd = // ...
  let fake_hwnd = unsafe {
    create_app_window(
      fake_window_class,
      "Fake Window",
      None,
      [1, 1],
      null_mut(),
    )
  }
  .unwrap();
```
Okay, now it works.
Not sure *what* the difference is here, but I guess we can investigate that later.
Only little problem is that now we have an extra window class floating around.
If we use [UnregisterClassW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unregisterclassw)
we can clean that up.

With `UnregisterClassW` you can pass in a pointer to a class name,
*or* you can pass in an atom value.
We'll make a separate function for each style.
```rust
#[link(name = "User32")]
extern "system" {
  /// [`UnregisterClassW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unregisterclassw)
  pub fn UnregisterClassW(lpClassName: LPCWSTR, hInstance: HINSTANCE) -> BOOL;
}

/// Un-registers the window class from the `HINSTANCE` given.
///
/// * The name must be the name of a registered window class.
/// * This requires re-encoding the name to null-terminated utf-16, which
///   allocates. Using [`unregister_class_by_atom`] instead does not allocate,
///   if you have the atom available.
/// * Before calling this function, an application must destroy all windows
///   created with the specified class.
///
/// See
/// [`UnregisterClassW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unregisterclassw)
pub unsafe fn unregister_class_by_name(
  name: &str, instance: HINSTANCE,
) -> Result<(), Win32Error> {
  let name_null = wide_null(name);
  let out = UnregisterClassW(name_null.as_ptr(), instance);
  if out != 0 {
    Ok(())
  } else {
    Err(get_last_error())
  }
}

/// Un-registers the window class from the `HINSTANCE` given.
///
/// * The atom must be the atom of a registered window class.
/// * Before calling this function, an application must destroy all windows
///   created with the specified class.
///
/// See [`UnregisterClassW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unregisterclassw)
pub unsafe fn unregister_class_by_atom(
  a: ATOM, instance: HINSTANCE,
) -> Result<(), Win32Error> {
  let out = UnregisterClassW(a as LPCWSTR, instance);
  if out != 0 {
    Ok(())
  } else {
    Err(get_last_error())
  }
}
```

And here's another thing that went wrong when I tried to figure this "window doesn't show up" problem.
At one point I was registering the same class twice,
because my `fake_wc` had the same name string as the "real" `wc`.
Then I got this error:

```
D:\dev\triangle-from-scratch>cargo run
   Compiling triangle-from-scratch v0.1.0 (D:\dev\triangle-from-scratch)
    Finished dev [unoptimized + debuginfo] target(s) in 0.73s
     Running `target\debug\triangle-from-scratch.exe`        
PIXELFORMATDESCRIPTOR { nSize: 40, nVersion: 1, dwFlags: 33317, iPixelType: 0, cColorBits: 32, cRedBits: 8, cRedShift: 16, cGreenBits: 8, cGreenShift: 8, cBlueBits: 8, cBlueShift: 0, cAlphaBits: 0, cAlphaShift: 0, cAccumBits: 64, cAccumRedBits: 16, cAccumGreenBits: 16, cAccumBlueBits: 16, cAccumAlphaBits: 16, cDepthBits: 24, cStencilBits: 8, cAuxBuffers: 4, iLayerType: 0, bReserved: 0, dwLayerMask: 0, dwVisibleMask: 0, dwDamageMask: 0 }
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Win32Error(1410)', src\main.rs:63:46
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\debug\triangle-from-scratch.exe` (exit code: 101)
```

Not helpful!
Error 1410, what on Earth does that mean?
So we should adjust the error lookups to happen in `Debug` as well as `Display`.

```rust
#[repr(transparent)]
pub struct Win32Error(pub DWORD);
impl std::error::Error for Win32Error {}

impl core::fmt::Debug for Win32Error {
  /// Displays the error using `FormatMessageW`
  ///
  /// ```
  /// use triangle_from_scratch::win32::*;
  /// let s = format!("{:?}", Win32Error(0));
  /// assert_eq!("The operation completed successfully.  ", s);
  /// let app_error = format!("{:?}", Win32Error(1 << 29));
  /// assert_eq!("Win32ApplicationError(536870912)", app_error);
  /// ```
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    // everything from before
  }
}
impl core::fmt::Display for Win32Error {
  /// Same as `Debug` impl
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "{:?}", self)
  }
}
```

But what if you *really wanted that error number*.
Well, maybe you do, and for that, we can use the "alternate" flag.

```rust
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    // ...
    if f.alternate() {
      return write!(f, "Win32Error({})", self.0);
    }
```

Now if you format with "{:?}" (which is what `unwrap` uses)
then you get the message form,
and if you really want to see the number you can format with "{:#?}".

Now when an `unwrap` goes bad it looks like this:
```
D:\dev\triangle-from-scratch>cargo run
   Compiling triangle-from-scratch v0.1.0 (D:\dev\triangle-from-scratch)
    Finished dev [unoptimized + debuginfo] target(s) in 0.87s
     Running `target\debug\triangle-from-scratch.exe`
PIXELFORMATDESCRIPTOR { nSize: 40, nVersion: 1, dwFlags: 33317, iPixelType: 0, cColorBits: 32, cRedBits: 8, cRedShift: 16, cGreenBits: 8, cGreenShift: 8, cBlueBits: 8, cBlueShift: 0, cAlphaBits: 0, cAlphaShift: 0, cAccumBits: 64, cAccumRedBits: 16, cAccumGreenBits: 16, cAccumBlueBits: 16, cAccumAlphaBits: 16, cDepthBits: 24, cStencilBits: 8, cAuxBuffers: 4, iLayerType: 0, bReserved: 0, dwLayerMask: 0, dwVisibleMask: 0, dwDamageMask: 0 }
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Class already exists.  ', src\main.rs:63:46
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: process didn't exit successfully: `target\debug\triangle-from-scratch.exe` (exit code: 101)
```
Ah, look, the class already existed, of course!

So finally we can set a pixel format on a fake window,
and then clean it all up,
and then make our real window.

## wglCreateContext

As fun as it is to make a fake window and do not much with it and then throw it away,
it might be even better if we did something with it.

The point of all this is that we *want* to be able to create an OpenGL context with the latest version,
and other advanced features.
However, Windows only lets you directly make an OpenGL 1.1 context.
To make a context with a newer version than that, you need to use an extension.
To use an extension, you need to check the extension string to see what extensions are available.
To check the extension string, you need to have a current OpenGL context.

What we do is we use our fake window to make a fake GL context,
which will be for the old OpenGL 1.1,
then we can get the extension string to check what extensions are available.
This lets us use the "advanced" capabilities like "making a context with a modern API version".

I told you at the start that this was gonna seem silly when I explained what was going on.
[I warned you about stairs bro!](http://www.mspaintadventures.com/sweetbroandhellajeff/)

Anyway, once we've gotten the info we need from the fake context then we close it,
and we close out all the other "fake" stuff,
and then we make the "real" stuff.

That means our next step is [wglCreateContext](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-wglcreatecontext),
and also the inverse, [wglDeleteContext](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-wgldeletecontext).

```rust
/// Handle (to a) GL Rendering Context
pub type HGLRC = HANDLE;

#[link(name = "Opengl32")]
extern "system" {
  /// [`wglCreateContext`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-wglcreatecontext)
  pub fn wglCreateContext(Arg1: HDC) -> HGLRC;

  /// [`wglDeleteContext`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-wgldeletecontext)
  pub fn wglDeleteContext(Arg1: HGLRC) -> BOOL;
}

/// See [`wglCreateContext`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-wglcreatecontext)
pub unsafe fn wgl_create_context(hdc: HDC) -> Result<HGLRC, Win32Error> {
  let hglrc = wglCreateContext(hdc);
  if hglrc.is_null() {
    Err(get_last_error())
  } else {
    Ok(hglrc)
  }
}

/// Deletes a GL Context.
///
/// * You **cannot** use this to delete a context current in another thread.
/// * You **can** use this to delete the current thread's context. The context
///   will be made not-current automatically before it is deleted.
///
/// See
/// [`wglDeleteContext`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-wgldeletecontext)
pub unsafe fn wgl_delete_context(hglrc: HGLRC) -> Result<(), Win32Error> {
  let success = wglDeleteContext(hglrc);
  if success != 0 {
    Ok(())
  } else {
    Err(get_last_error())
  }
}
```

And then in `main`:
```rust
  unsafe { set_pixel_format(fake_hdc, pf_index, &pfd) }.unwrap();
  let fake_hglrc = unsafe { wgl_create_context(fake_hdc) }.unwrap();

  // TODO: work with the fake context.

  // cleanup the fake stuff
  unsafe { wgl_delete_context(fake_hglrc) }.unwrap();
  assert!(unsafe { release_dc(fake_hwnd, fake_hdc) });
  unsafe { destroy_window(fake_hwnd) }.unwrap();
  unsafe { unregister_class_by_atom(fake_atom, instance) }.unwrap();
```

## wglMakeCurrent

It's no use making and destroying this fake context if we don't make it current:
```rust
#[link(name = "Opengl32")]
extern "system" {
  /// [`wglMakeCurrent`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-wglmakecurrent)
  pub fn wglMakeCurrent(hdc: HDC, hglrc: HGLRC) -> BOOL;
}

/// Makes a given HGLRC current in the thread and targets it at the HDC given.
///
/// * You can safely pass `null_mut` for both parameters if you wish to make no
///   context current in the thread.
///
/// See
/// [`wglMakeCurrent`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-wglmakecurrent)
pub unsafe fn wgl_make_current(
  hdc: HDC, hglrc: HGLRC,
) -> Result<(), Win32Error> {
  let success = wglMakeCurrent(hdc, hglrc);
  if success != 0 {
    Ok(())
  } else {
    Err(get_last_error())
  }
}
```

Before we can use the context we have to make it current,
and before we destroy it we have to make it not be current.
On Windows we don't *have* to make it not-current,
but it's just good habit because on *other* systems you must make a context not-current before you destroy it.

```rust
  let fake_hglrc = unsafe { wgl_create_context(fake_hdc) }.unwrap();
  unsafe { wgl_make_current(fake_hdc, fake_hglrc) }.unwrap();

  // TODO: work with the fake context.

  // cleanup the fake stuff
  unsafe { wgl_make_current(null_mut(), null_mut()) }.unwrap();
  unsafe { wgl_delete_context(fake_hglrc) }.unwrap();
```

## wglGetProcAddress

This is what we've been after the whole time!

```rust
// macros.rs

/// Turns a rust string literal into a null-terminated `&[u8]`.
#[macro_export]
macro_rules! c_str {
  ($text:expr) => {{
    concat!($text, '\0').as_bytes()
  }};
}

// win32.rs

/// Pointer to an ANSI string.
pub type LPCSTR = *const c_char;
/// Pointer to a procedure of unknown type.
pub type PROC = *mut c_void;

#[link(name = "Opengl32")]
extern "system" {
  /// [`wglGetProcAddress`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-wglgetprocaddress)
  pub fn wglGetProcAddress(Arg1: LPCSTR) -> PROC;
}

/// Gets a GL function address.
///
/// The input should be a null-terminated function name string. Use the
/// [`c_str!`] macro for assistance.
///
/// * You must have an active GL context for this to work. Otherwise you will
///   always get an error.
/// * The function name is case sensitive, and spelling must be exact.
/// * All outputs are context specific. Functions supported in one rendering
///   context are not necessarily supported in another.
/// * The extension function addresses are unique for each pixel format. All
///   rendering contexts of a given pixel format share the same extension
///   function addresses.
///
/// This *will not* return function pointers exported by `OpenGL32.dll`, meaning
/// that it won't return OpenGL 1.1 functions. For those old function, use
/// [`GetProcAddress`].
pub fn wgl_get_proc_address(func_name: &[u8]) -> Result<PROC, Win32Error> {
  // check that we end the slice with a \0 as expected.
  match func_name.last() {
    Some(b'\0') => (),
    _ => return Err(Win32Error(Win32Error::APPLICATION_ERROR_BIT)),
  }
  // Safety: we've checked that the end of the slice is null-terminated.
  let proc = unsafe { wglGetProcAddress(func_name.as_ptr().cast()) };
  match proc as usize {
    // Some non-zero values can also be errors,
    // https://www.khronos.org/opengl/wiki/Load_OpenGL_Functions#Windows
    0 | 1 | 2 | 3 | usize::MAX => return Err(get_last_error()),
    _ => Ok(proc),
  }
}
```

This part is pretty simple.
We get a value back, and on success it's a pointer to a function.
We'll have to use [transmute](https://doc.rust-lang.org/core/mem/fn.transmute.html)
to change the type into the proper function type,
but that's a concern for the caller to deal with.

## wglGetExtensionsStringARB

Okay, now we can get function pointers.
This lets us check for [platform specific extensions](https://www.khronos.org/opengl/wiki/Load_OpenGL_Functions#Windows_2)
using the [wglGetExtensionsStringARB](https://www.khronos.org/registry/OpenGL/extensions/ARB/WGL_ARB_extensions_string.txt) function.

For this, we'll want a helper macro to make C-style string literals for us:
```rust
/// Turns a rust string literal into a null-terminated `&[u8]`.
#[macro_export]
macro_rules! c_str {
  ($text:expr) => {{
    concat!($text, '\0').as_bytes()
  }};
}
```

And then we:
* lookup the function
* call the function
* get the info from the string pointer we get back

This part is kinda *a lot* when you just write it all inline in `main`:
```rust
  unsafe { wgl_make_current(fake_hdc, fake_hglrc) }.unwrap();

  #[allow(non_camel_case_types)]
  type wglGetExtensionsStringARB_t =
    unsafe extern "system" fn(HDC) -> *const c_char;
  let wgl_get_extension_string_arb: Option<wglGetExtensionsStringARB_t> = unsafe {
    core::mem::transmute(
      wgl_get_proc_address(c_str!("wglGetExtensionsStringARB")).unwrap(),
    )
  };
  let mut extension_string: *const u8 =
    unsafe { (wgl_get_extension_string_arb.unwrap())(fake_hdc) }.cast();
  assert!(!extension_string.is_null());
  let mut s = String::new();
  unsafe {
    while *extension_string != 0 {
      s.push(*extension_string as char);
      extension_string = extension_string.add(1);
    }
  }
  println!("> Extension String: {}", s);

  // cleanup the fake stuff
```

but if we break it down it's not so bad.
First let's put a function for gathering up a null-terminated byte string into our library.
This isn't Win32 specific, so we'll put it in `lib.rs`:
```rust
/// Gathers up the bytes from a pointer.
///
/// The byte sequence must be null-terminated.
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
```

Now that we have a `Vec<u8>` we want a `String`.
We can use [String::from_utf8](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8),
but that returns a Result (it fails if the bytes aren't valid utf8).
There's also [String::from_utf8_lossy](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy),
but if the bytes *are* valid utf8 then we get a borrow on our vec and we'd have to clone it to get the `String`.
What we *want* is to move the Vec if we can, and only allocate a new Vec if we must.
You'd think that this is a completely common thing to want,
but for whatever reason it's not in the standard library.

```rust
// PS: naming is hard :(

/// Converts a `Vec<u8>` into a `String` using the minimum amount of
/// re-allocation.
pub fn min_alloc_lossy_into_string(bytes: Vec<u8>) -> String {
  match String::from_utf8(bytes) {
    Ok(s) => s,
    Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
  }
}
```

Now in `win32.rs` we'll just `use super::*;` and make a function to get the extension string:
```rust
/// Gets the WGL extension string for the HDC passed.
///
/// * This relies on [`wgl_get_proc_address`], and so you must have a context
///   current for it to work.
/// * If `wgl_get_proc_address` fails then an Application Error is generated.
/// * If `wgl_get_proc_address` succeeds but the extension string can't be
///   obtained for some other reason you'll get a System Error.
///
/// The output is a space-separated list of extensions that are supported.
///
/// See
/// [`wglGetExtensionsStringARB`](https://www.khronos.org/registry/OpenGL/extensions/ARB/WGL_ARB_extensions_string.txt)
pub unsafe fn wgl_get_extension_string_arb(
  hdc: HDC,
) -> Result<String, Win32Error> {
  let f: wglGetExtensionsStringARB_t = core::mem::transmute(
    wgl_get_proc_address(c_str!("wglGetExtensionsStringARB"))?,
  );
  let p: *const u8 =
    (f.ok_or(Win32Error(Win32Error::APPLICATION_ERROR_BIT))?)(hdc).cast();
  if p.is_null() {
    Err(get_last_error())
  } else {
    let bytes = gather_null_terminated_bytes(p);
    Ok(min_alloc_lossy_into_string(bytes))
  }
}
```

And now we can try to get the extension string with a single call to that:
```rust
// main.rs: fn main
  unsafe { wgl_make_current(fake_hdc, fake_hglrc) }.unwrap();

  let res = unsafe { wgl_get_extension_string_arb(fake_hdc) };
  println!("> Extension String Result: {:?}", res);

  // cleanup the fake stuff
```

And with a little iterator magic:
```rust
  let extensions: Vec<String> =
    unsafe { wgl_get_extension_string_arb(fake_hdc) }
      .map(|s| {
        s.split(' ').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect()
      })
      .unwrap_or(Vec::new());
  println!("> Extensions: {:?}", extensions);
```

Which prints out alright:
```
D:\dev\triangle-from-scratch>cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target\debug\triangle-from-scratch.exe`        
> Got Pixel Format: PIXELFORMATDESCRIPTOR { nSize: 40, nVersion: 1, dwFlags: 33317, iPixelType: 0, cColorBits: 32, cRedBits: 8, cRedShift: 16, cGreenBits: 8, cGreenShift: 8, cBlueBits: 8, cBlueShift: 0, cAlphaBits: 0, cAlphaShift: 0, cAccumBits: 64, cAccumRedBits: 16, cAccumGreenBits: 16, cAccumBlueBits: 16, cAccumAlphaBits: 16, cDepthBits: 24, cStencilBits: 8, cAuxBuffers: 4, iLayerType: 0, bReserved: 0, dwLayerMask: 0, dwVisibleMask: 0, dwDamageMask: 0 }
> Extensions: ["WGL_ARB_buffer_region", "WGL_ARB_create_context", "WGL_ARB_create_context_no_error", "WGL_ARB_create_context_profile", "WGL_ARB_create_context_robustness", "WGL_ARB_context_flush_control", "WGL_ARB_extensions_string", "WGL_ARB_make_current_read", "WGL_ARB_multisample", "WGL_ARB_pbuffer", "WGL_ARB_pixel_format", "WGL_ARB_pixel_format_float", "WGL_ARB_render_texture", "WGL_ATI_pixel_format_float", "WGL_EXT_colorspace", "WGL_EXT_create_context_es_profile", "WGL_EXT_create_context_es2_profile", "WGL_EXT_extensions_string", "WGL_EXT_framebuffer_sRGB", "WGL_EXT_pixel_format_packed_float", "WGL_EXT_swap_control", "WGL_EXT_swap_control_tear", "WGL_NVX_DX_interop", "WGL_NV_DX_interop", "WGL_NV_DX_interop2", "WGL_NV_copy_image", "WGL_NV_delay_before_swap", "WGL_NV_float_buffer", "WGL_NV_multisample_coverage", "WGL_NV_render_depth_texture", "WGL_NV_render_texture_rectangle"]
```

## Grab Some Function Pointers

Now that we can see what WGL extensions are available,
we can grab out some function pointers.

Here's the key part:
*We don't call them yet.*

This is another silly thing, but it's true.
We just get the function pointers for now.
Then we destroy the fake stuff,
then we use the function pointers that we stored to make our real stuff.

We want to get function pointers for:
* [wglChoosePixelFormatARB](https://www.khronos.org/registry/OpenGL/extensions/ARB/WGL_ARB_pixel_format.txt)
  (provided by `WGL_ARB_pixel_format`) is required to choose advanced pixel formats (such as multisampling).
* [wglCreateContextAttribsARB](https://www.khronos.org/registry/OpenGL/extensions/ARB/WGL_ARB_create_context.txt)
  (provided by `WGL_ARB_create_context`) is required to make GL contexts with API versions above 1.1.
* [wglSwapIntervalEXT](https://www.khronos.org/registry/OpenGL/extensions/EXT/WGL_EXT_swap_control.txt)
  (provided by `WGL_EXT_swap_control`) is *not* required but is very handy, because it lets you enable
  [vsync](https://en.wikipedia.org/wiki/Screen_tearing#Vertical_synchronization)

These are our core three extension functions.
Many of the extensions listed above don't add new functions,
they just extend what values you can send to these three.

First we declare the types we'll be using:
```rust
// lib.rs

/// Type for [wglChoosePixelFormatARB](https://www.khronos.org/registry/OpenGL/extensions/ARB/WGL_ARB_pixel_format.txt)
pub type wglChoosePixelFormatARB_t = Option<
  unsafe extern "system" fn(
    hdc: HDC,
    piAttribIList: *const c_int,
    pfAttribFList: *const f32,
    nMaxFormats: UINT,
    piFormats: *mut c_int,
    nNumFormats: *mut UINT,
  ) -> BOOL,
>;
pub type FLOAT = c_float;
pub type c_float = f32;
/// Type for [wglCreateContextAttribsARB](https://www.khronos.org/registry/OpenGL/extensions/ARB/WGL_ARB_create_context.txt)
pub type wglCreateContextAttribsARB_t = Option<
  unsafe extern "system" fn(
    hDC: HDC,
    hShareContext: HGLRC,
    attribList: *const c_int,
  ) -> HGLRC,
>;
/// Type for [wglSwapIntervalEXT](https://www.khronos.org/registry/OpenGL/extensions/EXT/WGL_EXT_swap_control.txt)
pub type wglSwapIntervalEXT_t =
  Option<unsafe extern "system" fn(interval: c_int) -> BOOL>;
```

And then we store the function pointers:
```rust
  println!("> Extensions: {:?}", extensions);

  let wglChoosePixelFormatARB: wglChoosePixelFormatARB_t = unsafe {
    core::mem::transmute(
      wgl_get_proc_address(c_str!("wglChoosePixelFormatARB")).unwrap(),
    )
  };
  let wglCreateContextAttribsARB: wglCreateContextAttribsARB_t = unsafe {
    core::mem::transmute(
      wgl_get_proc_address(c_str!("wglCreateContextAttribsARB")).unwrap(),
    )
  };
  let wglSwapIntervalEXT: wglSwapIntervalEXT_t = unsafe {
    core::mem::transmute(
      wgl_get_proc_address(c_str!("wglSwapIntervalEXT")).unwrap(),
    )
  };

  // cleanup the fake stuff
```

Alright, I think we're done with all this fake context stuff.
We can move on to setting up our real context.

Actually, let's briefly put all that stuff into a single library function.
```rust
/// Grabs out the stuff you'll need to have fun with WGL.
pub fn get_wgl_basics() -> Result<
  (
    Vec<String>,
    wglChoosePixelFormatARB_t,
    wglCreateContextAttribsARB_t,
    wglSwapIntervalEXT_t,
  ),
  Win32Error,
> {
  // ...
}
```

It's just moving all the stuff you've seen over,
and then putting in a lot of drop guard types like we saw in format message.
There's not much new to talk about, so we'll keep moving.

## Our New Window Setup

Alright, so now let's get some useful stuff with our window:
```rust
struct WindowData {
  hdc: HDC,
}
impl Default for WindowData {
  fn default() -> Self {
    unsafe { core::mem::zeroed() }
  }
}
```

Since it's going to be connected to a GL context now we don't want to get and free it with every `WM_PAINT`.
Instead, we'll get it once after the window is created,
then stuff it into the WindowData field and leave it there.
The `WM_DESTROY` can clean it up before destroying the window itself.

```rust
// fn main
  let lparam: *mut WindowData = Box::leak(Box::new(WindowData::default()));
  let hwnd = unsafe {
    create_app_window(
      sample_window_class,
      "Sample Window Name",
      None,
      [800, 600],
      lparam.cast(),
    )
  }
  .unwrap();
  let hdc = unsafe { get_dc(hwnd) }.unwrap();
  unsafe { (*lparam).hdc = hdc };
```

And we need to adjust our window procedure:
```rust
    WM_DESTROY => {
      match get_window_userdata::<WindowData>(hwnd) {
        Ok(ptr) if !ptr.is_null() => {
          let window_data = Box::from_raw(ptr);
          let _ = release_dc(hwnd, window_data.hdc);
          println!("Cleaned up the box.");
        }
        Ok(_) => {
          println!("userdata ptr is null, no cleanup")
        }
        Err(e) => {
          println!("Error while getting the userdata ptr for cleanup: {}", e)
        }
      }
      post_quit_message(0);
    }
    WM_PAINT => match get_window_userdata::<WindowData>(hwnd) {
      Ok(ptr) if !ptr.is_null() => {
        // TODO: real painting, eventually
        println!("painting!");
      }
      Ok(_) => {
        println!("userdata ptr is null")
      }
      Err(e) => {
        println!("Error while getting the userdata ptr: {}", e)
      }
    },
```

## Finally We Call wglChoosePixelFormatARB

We're finally ready to call our [wglChoosePixelFormatARB](https://www.khronos.org/registry/OpenGL/extensions/ARB/WGL_ARB_pixel_format.txt) pointer.

This one is fairly flexible.
We can pass a list of integer (key,value) pairs,
a list of float (key,value) pairs,
and the space to get some outputs.

As far as I can tell,
there's no reason in the basic extension for any float attributes to be specified.
Other extensions might add options in the future,
but there's nothing for us there right now.
The int attributes, on the other hand, have many interesting things.
For both lists, the function knows the list is over when it sees a 0 in the key position.
Also for both lists, we can pass a null instead of a list pointer.

Meanwhile, we can have more than one output if we want.
We pass in a count, and a pointer to an array of that length.
The function will fill in as many array values as it can.
There's also a pointer to an integer that we pass,
and it gets written the number of outputs that were found.
This could be the full array, but it could also be less than the full array.

Interestingly, using `min_const_generics` might work here.
We could make the array of values to return be a const generic.
But we only actually need *one* pixel format,
so we'll just pick the first format they give us.

The wrapper function for this is not complex, but it is *tall*.
```rust
/// Arranges the data for calling a [`wglChoosePixelFormatARB_t`] procedure.
///
/// * Inputs are slices of [key, value] pairs.
/// * Input slices **can** be empty.
/// * Non-empty slices must have a zero value in the key position of the final
///   pair.
pub unsafe fn do_wglChoosePixelFormatARB(
  f: wglChoosePixelFormatARB_t, hdc: HDC, int_attrs: &[[c_int; 2]],
  float_attrs: &[[FLOAT; 2]],
) -> Result<c_int, Win32Error> {
  let app_err = Win32Error(Win32Error::APPLICATION_ERROR_BIT);
  let i_ptr = match int_attrs.last() {
    Some([k, _v]) => {
      if *k == 0 {
        int_attrs.as_ptr()
      } else {
        return Err(app_err);
      }
    }
    None => null(),
  };
  let f_ptr = match float_attrs.last() {
    Some([k, _v]) => {
      if *k == 0.0 {
        int_attrs.as_ptr()
      } else {
        return Err(app_err);
      }
    }
    None => null(),
  };
  let mut out_format = 0;
  let mut out_format_count = 0;
  let b = (f.ok_or(app_err)?)(
    hdc,
    i_ptr.cast(),
    f_ptr.cast(),
    1,
    &mut out_format,
    &mut out_format_count,
  );
  if b != 0 && out_format_count == 1 {
    Ok(out_format)
  } else {
    Err(get_last_error())
  }
}
```

Now the way we call this thing is that we're gonna have some "base" requirements,
then we can look at the extensions and maybe ask for a little more if it's available,
then we finalize the list by putting in that zero.

After we get the pixel format index,
we can't set it directly, because we need a `PIXELFORMATDESCRIPTOR` to go with it.
First we use `describe_pixel_format`, then we can `set_pixel_format`.
```rust
  // base criteria
  let mut int_attribs = vec![
    [WGL_DRAW_TO_WINDOW_ARB, true as _],
    [WGL_SUPPORT_OPENGL_ARB, true as _],
    [WGL_DOUBLE_BUFFER_ARB, true as _],
    [WGL_PIXEL_TYPE_ARB, WGL_TYPE_RGBA_ARB],
    [WGL_COLOR_BITS_ARB, 32],
    [WGL_DEPTH_BITS_ARB, 24],
    [WGL_STENCIL_BITS_ARB, 8],
  ];
  // if sRGB is supported, ask for that
  if wgl_extensions.iter().any(|s| s == "WGL_EXT_framebuffer_sRGB") {
    int_attribs.push([WGL_FRAMEBUFFER_SRGB_CAPABLE_EXT, true as _]);
  };
  // let's have some multisample if we can get it
  if wgl_extensions.iter().any(|s| s == "WGL_ARB_multisample") {
    int_attribs.push([WGL_SAMPLE_BUFFERS_ARB, 1]);
  };
  // finalize our list
  int_attribs.push([0, 0]);
  // choose a format, get the PIXELFORMATDESCRIPTOR, and set it.
  let pix_format = unsafe {
    do_wglChoosePixelFormatARB(wglChoosePixelFormatARB, hdc, &int_attribs, &[])
  }
  .unwrap();
  let pfd = unsafe { describe_pixel_format(hdc, pix_format) }.unwrap();
  unsafe { set_pixel_format(hdc, pix_format, &pfd) }.unwrap();
```

## Open That Stupid Context Already (wglCreateContextAttribsARB)

Now that we have a pixel format set, we can create a context.

To create an advanced context,
we call [wglCreateContextAttribsARB](https://www.khronos.org/registry/OpenGL/extensions/ARB/WGL_ARB_create_context.txt).

It's not too different from the last function we used.
We pass a list of (key,value) pairs in,
with a 0 key to signal the final entry.

The wrapper for this should look familiar, it's the same basic idea:
```rust
/// Arranges the data for calling a [`wglCreateContextAttribsARB_t`] procedure.
///
/// * The input slice consists of [key, value] pairs.
/// * The input slice **can** be empty.
/// * Any non-empty input must have zero as the key value of the last position.
pub unsafe fn do_wglCreateContextAttribsARB(
  f: wglCreateContextAttribsARB_t, hdc: HDC, hShareContext: HGLRC,
  attribList: &[[i32; 2]],
) -> Result<HGLRC, Win32Error> {
  let app_err = Win32Error(Win32Error::APPLICATION_ERROR_BIT);
  let i_ptr = match attribList.last() {
    Some([k, _v]) => {
      if *k == 0 {
        attribList.as_ptr()
      } else {
        return Err(app_err);
      }
    }
    None => null(),
  };
  let hglrc = (f.ok_or(app_err)?)(hdc, hShareContext, i_ptr.cast());
  if hglrc.is_null() {
    Err(get_last_error())
  } else {
    Ok(hglrc)
  }
}
```

And this time we don't even have to use a vec to store all our settings.
We don't have a dynamic number of settings, so a plain array will do fine.
```rust
  // now we create a context.
  const FLAGS: c_int = WGL_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB
    | if cfg!(debug_assertions) { WGL_CONTEXT_DEBUG_BIT_ARB } else { 0 };
  let hglrc = unsafe {
    do_wglCreateContextAttribsARB(
      wglCreateContextAttribsARB,
      hdc,
      null_mut(),
      &[
        [WGL_CONTEXT_MAJOR_VERSION_ARB, 3],
        [WGL_CONTEXT_MINOR_VERSION_ARB, 3],
        [WGL_CONTEXT_PROFILE_MASK_ARB, WGL_CONTEXT_CORE_PROFILE_BIT_ARB],
        [WGL_CONTEXT_FLAGS_ARB, FLAGS],
        [0, 0],
      ],
    )
  }
  .unwrap();
  unsafe { wgl_make_current(hdc, hglrc) }.unwrap();
  unsafe { (*lparam).hglrc = hglrc };
```

I'm here selecting OpenGL 3.3 Core,
because some day,
when this tutorial is finally over,
I'm going to say,
"and now you can learn how to do the rest of OpenGL by going over to [LearnOpenGL.com](https://LearnOpenGL.com)!".
And they teach 3.3 Core.
If you don't yet know about OpenGL versions,
that's the oldest version of the "newer" set of OpenGL versions.
If you do know enough about OpenGL to have an opinion on what other version to use,
you could also use any other version as well.

## LoadLibraryW

On both MSDN and the OpenGL Wiki it says that any function that's in `OpenGL32.dll`
is *not* able to be looked up with `wglGetProcAddress`.
Instead you have to use the [GetProcAddress](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getprocaddress) function.
To use that, we need to have a loaded library.
The library loading itself uses a textual name, so it has `A` and `W` versions.
As usual, we want the `W` version, so we want [LoadLibraryW](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryw).
When we're all done with the library we'll use [FreeLibrary](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-freelibrary)
to close out the library.
The `FreeLibrary` call just takes the handle to the module, so it doesn't have `A` and `W` variants.

```rust
/// Pointer to a procedure of unknown type.
pub type FARPROC = *mut c_void;

#[link(name = "Kernel32")]
extern "system" {
  /// [`LoadLibraryW`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryw)
  pub fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;

  /// [`FreeLibrary`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-freelibrary)
  pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;

  /// [`GetProcAddress`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getprocaddress)
  pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
}

/// Loads a dynamic library.
///
/// The precise details of how the library is searched for depend on the input
/// string.
///
/// See [`LoadLibraryW`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryw)
pub fn load_library(name: &str) -> Result<HMODULE, Win32Error> {
  let name_null = wide_null(name);
  // Safety: the input pointer is to a null-terminated string
  let hmodule = unsafe { LoadLibraryW(name_null.as_ptr()) };
  if hmodule.is_null() {
    Err(get_last_error())
  } else {
    Ok(hmodule)
  }
}
```

Also, if you're wondering why `GetProcAddress` doesn't have `A` and `W` versions,
it's because C function names can only ever be ANSI content.

Now we can put an `HMODULE` into our WindowData struct.

```rust
struct WindowData {
  hdc: HDC,
  hglrc: HGLRC,
  opengl32: HMODULE,
}
```

Then we can load a module and assign it.
We can do this basically anywhere in the startup process,
but it's emotionally connected to using GL,
so we'll do it right after we make our context.

```rust
  unsafe { wgl_make_current(hdc, hglrc) }.unwrap();
  unsafe { (*lparam).hglrc = hglrc };

  let opengl32 = load_library("opengl32.dll").unwrap();
  unsafe { (*lparam).opengl32 = opengl32 };
```

And we have to properly close out the module when we're cleaning up the window.

```rust
    WM_DESTROY => {
      println!("WM_DESTROY");
      match get_window_userdata::<WindowData>(hwnd) {
        Ok(ptr) if !ptr.is_null() => {
          let window_data = Box::from_raw(ptr);
          FreeLibrary(window_data.opengl32);
          wgl_delete_context(window_data.hglrc)
            .unwrap_or_else(|e| eprintln!("GL Context deletion error: {}", e));
          // ...
```

## What Do We Load?

Now that we can load up GL functions, what do we want to load?
And what are the type signatures?

Well, for that, we could look at our old friend [gl.xml](https://github.com/KhronosGroup/OpenGL-Registry/blob/master/xml/gl.xml).
It describes the entire GL API in a structured way.

However, that's overkill for what we need at the moment.
We only need to use like two functions to just clear the screen to a color,
so instead we'll just check the online manual pages.
What we're after for is [glClearColor](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glClearColor.xhtml)
and also [glClear](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glClear.xhtml).

What's a GLfloat and a GLbitfield? For that we can look in `gl.xml`.
If we look around we'll eventually find these entries:
```xml
<type>typedef unsigned int <name>GLbitfield</name>;</type>

<type requires="khrplatform">typedef khronos_float_t <name>GLfloat</name>;</type>
```

Cool.
Hmm, we need a new library module for this.
These definitions will be common to our GL usage across all the platforms,
so let's start a new file for that.

```rust
// lib.rs

//! Library for the [Triangle From Scratch][tfs] project.
//!
//! [tfs]: https://rust-tutorials.github.io/triangle-from-scratch/

mod macros;

pub mod util;

#[cfg(windows)]
pub mod win32;
// this is so that gl will see the C types
#[cfg(windows)]
use win32::*;

pub mod gl;
```

And then our fun new module
```rust
#![allow(non_camel_case_types)]

use super::*;

/// From `gl.xml`
pub type GLbitfield = c_uint;

/// From `gl.xml`
pub type GLfloat = c_float;

/// See [`glClearColor`](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glClearColor.xhtml)
pub type glClearColor_t = Option<
  unsafe extern "system" fn(
    red: GLfloat,
    green: GLfloat,
    blue: GLfloat,
    alpha: GLfloat,
  ),
>;

/// See [`glClear`](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glClear.xhtml)
pub type glClear_t = Option<unsafe extern "system" fn(mask: GLbitfield)>;
```

Hmm, but where do we get the values for the `GL_COLOR_BUFFER_BIT` and so on?
That's also stored in `gl.xml`.

If you search for `GL_COLOR_BUFFER_BIT` you'll see a lot of "group" info,
but that's old and not what we want.
Eventually if you keep looking you'll see a line like this:
```xml
<enum value="0x00004000" name="GL_COLOR_BUFFER_BIT" group="ClearBufferMask,AttribMask"/>
```
This is good, this is good.

So what are the groups?
Well, there's been a heroic effort by the GL maintainers to get the `gl.xml` descriptions
of functions to be *slightly* better documented by giving each function input a group,
and then each constant lists what groups it's in.

If we look at the XML definition for `glClear`:
```xml
<command>
  <proto>void <name>glClear</name></proto>
  <param group="ClearBufferMask"><ptype>GLbitfield</ptype> <name>mask</name></param>
  <glx type="render" opcode="127"/>
</command>
```
See, that mask argument should be a constant in the "ClearBufferMask" group.
And `GL_COLOR_BUFFER_BIT` is in the "ClearBufferMask" group,
so it would be a correct call to make.

This is just some info to try and help static checkers,
but it's still pretty loosey-goosey,
and you don't really have to pay much attention if you don't care to.
We won't be following the group info while we're doing this by hand.
If we make a fancy generator then that might care to track the group info.

So we add some fields to our window data:
```rust
struct WindowData {
  hdc: HDC,
  hglrc: HGLRC,
  opengl32: HMODULE,
  gl_clear: glClear_t,
  gl_clear_color: glClearColor_t,
}
```

Then we add some functions to our window data:
```rust
impl WindowData {
  pub fn gl_get_proc_address(&self, name: &[u8]) -> *mut c_void {
    assert!(*name.last().unwrap() == 0);
    let p = unsafe { wglGetProcAddress(name.as_ptr().cast()) };
    match p as usize {
      0 | 1 | 2 | 3 | usize::MAX => unsafe {
        GetProcAddress(self.opengl32, name.as_ptr().cast())
      },
      _ => p,
    }
  }
  #[rustfmt::skip]
  pub unsafe fn load_gl_functions(&mut self) {
    self.gl_clear = core::mem::transmute(self.gl_get_proc_address(c_str!("glClear")));
    self.gl_clear_color = core::mem::transmute(self.gl_get_proc_address(c_str!("glClearColor")));
  }
}
```

And then, in addition to simply setting the loaded library,
we also tell the window data to do its loading process:
```rust
  let opengl32 = load_library("opengl32.dll").unwrap();
  unsafe { (*lparam).opengl32 = opengl32 };
  unsafe { (*lparam).load_gl_functions() };
```

## Clear The Screen

To clear the screen's color we call `glClear(GL_COLOR_BUFFER_BIT)`.
We can also set the color we want to clear things to.
By default it'll clear to black,
but we can select any color we want.

```rust
    WM_PAINT => match get_window_userdata::<WindowData>(hwnd) {
      Ok(ptr) if !ptr.is_null() => {
        let window_data = ptr.as_mut().unwrap();
        (window_data.gl_clear_color.unwrap())(0.6, 0.7, 0.8, 1.0);
        (window_data.gl_clear.unwrap())(GL_COLOR_BUFFER_BIT);
      }
```

And there's one more step!

We need to call [SwapBuffers](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-swapbuffers)
on our HDC to tell windows to swap the front and back buffers.

Declare it.
```rust
// in the library's win32.rs

#[link(name = "Gdi32")]
extern "system" {
  /// [`SwapBuffers`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-swapbuffers)
  pub fn SwapBuffers(Arg1: HDC) -> BOOL;
}
```

And then call it.
```rust
    WM_PAINT => match get_window_userdata::<WindowData>(hwnd) {
      Ok(ptr) if !ptr.is_null() => {
        let window_data = ptr.as_mut().unwrap();
        (window_data.gl_clear_color.unwrap())(0.6, 0.7, 0.8, 1.0);
        (window_data.gl_clear.unwrap())(GL_COLOR_BUFFER_BIT);
        SwapBuffers(window_data.hdc);
      }
```

And we finally get a nice, soft, blue sort of color in our window.

## Swap Interval

Oh heck, we didn't set a swap interval!

Remember how we loaded up a pointer for [wglSwapIntervalEXT](https://www.khronos.org/registry/OpenGL/extensions/EXT/WGL_EXT_swap_control.txt),
and then we didn't use it at all?
Uh, I guess we can call it after we load the GL functions.
We just need to set it once and it'll stay that way for the rest of the program.

```rust
  let opengl32 = load_library("opengl32.dll").unwrap();
  unsafe { (*lparam).opengl32 = opengl32 };
  unsafe { (*lparam).load_gl_functions() };

  // Enable "adaptive" vsync if possible, otherwise normal vsync
  if wgl_extensions.iter().any(|s| s == "WGL_EXT_swap_control_tear") {
    unsafe { (wglSwapIntervalEXT.unwrap())(-1) };
  } else {
    unsafe { (wglSwapIntervalEXT.unwrap())(1) };
  }
```

Now, any time we call `SwapBuffers`,
the system will sync the swap with the vertical trace of the screen,
and it'll wait at least 1 full monitor cycle between each swap.

If we have the adaptive vsync available, it'll still wait at least 1 frame,
but if we're only slightly off from the correct time, it'll swap immediately.
This reduces visual stutter by allowing occasional visual tearing.
Neither of those are great, but sometimes the program will struggle to keep up.
Usually the vsync mode is a user setting within a game or whatever,
so you can just let users pick how they want to handle things.

## Are We Done?

Yeah, basically!

You understand the basics of how we find a GL function type,
lookup the function to load it into the program at runtime,
and call it to make something happen.

> and now you can learn how to do the rest of OpenGL by going over to [LearnOpenGL.com](https://LearnOpenGL.com)!

I promised I'd say that to you one day.

No, but really, *the basics* have all been explained.
There's a lot of stuff that's still clunky as heck,
but it all works.

What's certainly left to do is make it more ergonomic.
However, we're already at just over 9300 words.
We might talk about ways to make GL nice to work with in another lesson some day.
