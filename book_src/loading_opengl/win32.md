
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

pub unsafe fn create_app_window(
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

TODO: debug for Win32Error
