
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

So for this, we'll be using the "cleaned up" Win32 example as our base,
and then continuing on from there.

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

The numeric value of `CS_OWNDC` can be found with a [search of MSDN](https://docs.microsoft.com/en-us/search/?scope=Desktop&terms=CS_OWNDC),
which leads us to the [Window Class Styles](https://docs.microsoft.com/en-us/windows/win32/winmsg/window-class-styles) page.
If we glance at the other options there's mostly stuff we don't need.
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
Except, the `PIXELFORMATDESCRIPTOR` page on MSDN doesn't list the values.
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

We're making most of the consts `u8` while the rest of them are `u32` just because that way they naturally have the type of the field in the struct that they go with.

## ChoosePixelFormat

Okay once we have a `PIXELFORMATDESCRIPTOR` value we call [ChoosePixelFormat](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-choosepixelformat)
to get a "pixel format index" that's the closest available pixel format to our request.

First we declare the external call of course:
```rust
#[link(name = "Gdi32")]
extern "system" {
  /// [`ChoosePixelFormat`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-choosepixelformat)
  pub fn ChoosePixelFormat(
    hdc: HDC, ppfd: *const PIXELFORMATDESCRIPTOR,
  ) -> c_int;
}
```

Oh look, we're using a new external library.
Instead of just Kernel32 and User32, now we've got Gdi32 in the mix.
Neat, I guess. Doesn't really make a difference.

Of course, this function can fail,
and we want to have a `Result` return type in the final version we'll use.
Instead of doing a whole thing with the raw calls and then making the "nicer" version after,
we'll just make the nicer version immediately,
since by now we've seen how to do that.
