
# Loading Vulkan Using Win32

For this lesson we'll be using the "cleaned up" Win32 example as our starting point.

## Function Loading

The first thing we need to do is something that we saw in the GL tutorial:
We load a dynamic library and then get a function pointer out of it.

The extern functions look like this:

```rust
#[link(name = "Kernel32")]
extern "system" {
  /// [`LoadLibraryW`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryw)
  pub fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;

  /// [`FreeLibrary`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-freelibrary)
  pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;

  /// [`GetProcAddress`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getprocaddress)
  pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
}
```

Which we'll wrap up like this:
```rust
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

pub fn get_proc_address(
  hmodule: HMODULE, name: &[u8],
) -> Result<NonNull<c_void>, Win32Error> {
  if let Some(0) = name.last() {
    let p = unsafe { GetProcAddress(hmodule, name.as_ptr().cast()) };
    NonNull::new(p).ok_or_else(|| get_last_error())
  } else {
    Err(Win32Error(Win32Error::APPLICATION_ERROR_BIT))
  }
}
```
And, uh, that's all fine, I guess.

We don't really need to wrap `FreeLibrary` because we'd only be freeing the library when the program is shutting down anyway.
Actually, we won't free the library period, we'll just let the OS clean up after us.

## Obtaining `vkGetInstanceProcAddr`

Now in our `main.rs`, in the `main` function,
after we make the window and before we show it,
let's put our VK initialization in there.

We're going to open a DLL called `vulkan-1.dll`,
and we're going to get out a pointer for a function named [vkGetInstanceProcAddr](https://vulkan.lunarg.com/doc/view/1.2.162.0/mac/chunked_spec/chap5.html)
```rust
  let lparam: *mut i32 = Box::leak(Box::new(5_i32));
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

  let vk_module_handle = load_library("vulkan-1.dll").unwrap();
  let p: NonNull<c_void> =
    get_proc_address(vk_module_handle, c_str!("vkGetInstanceProcAddr"))
      .unwrap();

  let _previously_visible = unsafe { ShowWindow(hwnd, SW_SHOW) };
```

So that gives us a `NonNull<c_void>`.
We can `transmute` that into a function pointer type, but... what type?
The vulkan spec sorta says...
```c
// Provided by VK_VERSION_1_0
PFN_vkVoidFunction vkGetInstanceProcAddr(
    VkInstance                                  instance,
    const char*                                 pName);
```
Now we need to know about `VkInstance`...
```c
// Provided by VK_VERSION_1_0
VK_DEFINE_HANDLE(VkInstance)
```
Okay so what's *that* mean?

For this we need to go to [gl.xml](https://raw.githubusercontent.com/KhronosGroup/Vulkan-Headers/master/registry/vk.xml)
(warning: this is a slightly large file, I wouldn't open it in a mobile browser).

## vk.xml

This is the "machine readable" version of the Vulkan Spec.
It's used to generated the Vulkan headers in C and C++,
and it can be used to generate bindings for other languages too.
The whole file is very large, and parsing it properly is the subject of an entire lesson all by itself.
For the moment, we'll just use Ctrl+F to find the bit we want and then convert by hand.

```xml
<type category="define">
#define <name>VK_DEFINE_HANDLE</name>(object) typedef struct object##_T* object;</type>
```
So this is... one of the parts where the Vulkan spec is a little *too* much C.
This isn't a direct definition, it's a C pre-processor directive that eventually gives us a definition.
The new detail here, that we haven't seen when looking at C stuff before, is the `##` part.
That "pastes" the left side and right side into a single identifier.
An input like `foo` pasted to `_T` would become `foo_T`.

So `VK_DEFINE_HANDLE(VkInstance)` expands to:
```c
typedef struct VkInstance_T* VkInstance;
```
And *now* we can read it, this is a classic opaque object thingy like we've seen a few times.

Let's start a new `vk.rs` file and start writing down our Vulkan specific library stuff.

```rust
macro_rules! VK_DEFINE_HANDLE {
  ($id:ident) => {
    #[repr(transparent)]
    pub struct $id(*mut c_void);
  };
}

VK_DEFINE_HANDLE!(VkInstance);
```

Now we have a `VkInstance` type that's just an opaque pointer.

We also needed to define `PFN_vkVoidFunction`:

```xml
<comment>The PFN_vkVoidFunction type are used by VkGet*ProcAddr below</comment>
<type category="funcpointer">typedef void (VKAPI_PTR *<name>PFN_vkVoidFunction</name>)(void);</type>
```
Well, let's get all the XML out of there
```c
typedef void (VKAPI_PTR *PFN_vkVoidFunction)(void);
```
Ah, okay, so that's basically like how `WNDPROC` was defined.
Now we have no inputs or return value, but the same general idea.

Also, we need to check that [VKAPI_PTR](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VKAPI_PTR.html) thing.
Looking in the docs, it seems to mean either [VKAPI_CALL](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VKAPI_CALL.html) (on MSVC)
or [VKAPI_ATTR](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VKAPI_ATTR.html) (on GCC/Clang).
That means that we want to use the `"system"` calling convention,
which will be "stdcall" on MSVC and "C" on other targets.

```rust
/// [PFN_vkVoidFunction](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/PFN_vkVoidFunction.html)
pub type PFN_vkVoidFunction = Option<unsafe extern "system" fn()>;
```
Note: PFN == "Pointer To A Function"

So finally, we can define the type of `vkGetInstanceProcAddr`:
```rust
/// [vkGetInstanceProcAddr](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetInstanceProcAddr.html)
pub type PFN_vkGetInstanceProcAddr = Option<
  unsafe extern "system" fn(
    instance: VkInstance,
    p_name: *const c_char,
  ) -> PFN_vkVoidFunction,
>;
```

And then use that `transmute` we talked about:
```rust
  let vk_module_handle = load_library("vulkan-1.dll").unwrap();
  let vkGetInstanceProcAddr = unsafe {
    transmute::<NonNull<c_void>, PFN_vkGetInstanceProcAddr>(
      get_proc_address(vk_module_handle, c_str!("vkGetInstanceProcAddr"))
        .unwrap(),
    )
  };
```

Actually... hold on.
That's a little sub-optimal.
The way we have it now, we've got all these optional function pointers,
but no way to name the type of the *non-optional* function pointer inside.
Let's make up our own name for the inner function type,
and then use that to define the `PFN_` version that's using the `Option`.

```rust
/// [PFN_vkVoidFunction](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/PFN_vkVoidFunction.html)
pub type system_void_fn = unsafe extern "system" fn();

/// [PFN_vkVoidFunction](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/PFN_vkVoidFunction.html)
pub type PFN_vkVoidFunction = Option<system_void_fn>;

/// [vkGetInstanceProcAddr](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetInstanceProcAddr.html)
pub type vkGetInstanceProcAddr_t =
  unsafe extern "system" fn(
    instance: VkInstance,
    p_name: *const c_char,
  ) -> PFN_vkVoidFunction;

/// [vkGetInstanceProcAddr](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetInstanceProcAddr.html)
pub type PFN_vkGetInstanceProcAddr = Option<vkGetInstanceProcAddr_t>;
```
That seems much better.

(The naming scheme here is that we're just putting a `_t` suffix on the name of the function instead of a `PFN_` prefix.)

Okay, and.... we're gonna fudge it a tiny tiny bit.
According to the C spec the `vkGetInstanceProcAddr` function is supposed to take a `*const c_char`,
but when you read the docs the second arg **must** be a null-terminated utf-8 string.
In Rust, that would be more like a `*const u8` than a `*const c_char`.
Remember that a `c_char` is either `i8` or `u8` depending on the C library you're linking to.
So really there's *essentially* no difference.
It's 8-bits either way.
Since it's a compatible definition, we'll have our definition use `*const u8`.
That way, we'll be able to easily use this with our `c_str!` macro (which produces `&[u8]` values).

Finally, we don't really want to ever unload that vulkan library.
If we did, all our vulkan function pointers would become invalidated.
Also we don't want to accidentally load any *other* function pointers using `get_proc_address` and the library handle,
instead of properly going through `vkGetInstanceProcAddr` (which I **totally** did in a previous draft of this tutorial).
To fix both of these things at once, we'll open the library handle inside the block that assigns `vkGetInstanceProcAddr`,
and then just... forget to close it.
The "leaked" handle will continue to be live for the rest of the program,
and the OS will pick up after us when the process dies.

It might at first seem irresponsible to not close a "resource" like this,
but with dynamic libraries this is honestly best practice.

In fact, trying to unload and reload a dynamic library,
such as for "live hotloading",
can cause horrible soundness issues, because of some of the optimizations that Rust does.
I'm not saying you *can't* do it,
I'm just saying that it should be considered a development-only sort of thing,
and it might explode on you.
Don't ship it to users, keep it just for development purposes.

Anyway, here's our final form of how to get `vkGetInstanceProcAddr` on Windows:
```rust
  let vkGetInstanceProcAddr = unsafe {
    let vk_module_handle = load_library("vulkan-1.dll").unwrap();
    let fn_ptr: NonNull<c_void> =
      get_proc_address(vk_module_handle, c_str!("vkGetInstanceProcAddr"))
        .unwrap();
    transmute::<_, vkGetInstanceProcAddr_t>(fn_ptr)
    // Note(Lokathor): Here we're just "leaking" the vulkan library handle, and
    // leaving the library loaded for the rest of the program. The OS will clean
    // things up when the process exits.
  };
```

Once all those background types are defined for us, it's actually simple!

## End Of Chapter

Now that we're equipped with a `vkGetInstanceProcAddr` value,
we're mostly done with the Win32 part of all this.

We'll have to do a little more Win32 related stuff when go to make our surface,
which is the thing that connects Vulkan's output to our window on screen,
but other than that the rest of all our Vulkan setup and usage is pretty well platform agnostic.
