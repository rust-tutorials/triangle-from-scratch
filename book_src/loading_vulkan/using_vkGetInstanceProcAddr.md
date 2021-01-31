# Using `vkGetInstanceProcAddr`

If we look in the spec at [section 4.1](https://renderdoc.org/vkspec_chunked/chap5.html#initialization-functionpointers)
then we'll see "Table 1. vkGetInstanceProcAddr behavior".
This shows us all the stuff we're able to do with our fancy new `vkGetInstanceProcAddr`.

Basically, this is a function in the "get procedure" family.
You give it names of functions to look up,
and it gives you pointers to those functions,
assuming they're available.

Without an **Instance**, we're limited to being able to look up only the following functions:
* [vkEnumerateInstanceVersion](https://renderdoc.org/vkspec_chunked/chap5.html#vkEnumerateInstanceVersion)
  lets you find out what version of vulkan is supported.
  It was added in 1.1, so if it's not available then you're limited to 1.0
* [vkEnumerateInstanceExtensionProperties](https://renderdoc.org/vkspec_chunked/chap40.html#vkEnumerateInstanceExtensionProperties)
  lets us look up what *instance* level extensions are available.
  There's also device level extensions, but that's a separate function.
* [vkEnumerateInstanceLayerProperties](https://renderdoc.org/vkspec_chunked/chap40.html#vkEnumerateInstanceLayerProperties)
  lets us check the available layers.
  A "layer" inserts itself into the call stack of Vulkan and provides something extra.
  Usually it provides argument validation or logging.
  All layers are off by default, and we'll want to see about turning on some of them.
* [vkCreateInstance](https://renderdoc.org/vkspec_chunked/chap5.html#vkCreateInstance)
  is how we actually create the instance.
  However, arranging all the correct info for this call is a little tricky,
  and we'll usually want to use the other functions to see what's available to ask for.
* [vkGetInstanceProcAddr](https://renderdoc.org/vkspec_chunked/chap5.html#vkGetInstanceProcAddr) (in VK 1.2 or later)
  I'm not sure *why* you'd want to do this, but the spec says that in VK 1.2 or later you can use vkGetInstanceProcAddr to get the address of itself.
  Someone wanted this enough to get the spec changed over it, but I really don't know why you'd do this.
  
To look up *any* other function at all we need to provide a valid instance.

Since we don't have an instance, we better create one.

## Using `vkCreateInstance`

Instance creation is done with `vkCreateInstance`,
but before that we'll need to have three pieces of info to hand over:
* A pointer to a [VkInstanceCreateInfo](https://renderdoc.org/vkspec_chunked/chap5.html#VkInstanceCreateInfo), properly filled out
* An optional pointer to our [VkAllocationCallbacks](https://renderdoc.org/vkspec_chunked/chap12.html#VkAllocationCallbacks) for host memory allocation.
  In a Vulkan context, "host" always means the CPU and OS side of things, where our program lives.
  We can pass a null pointer, and vulkan will just use some default allocation system, depending on the OS.
  That's fine for us.
  This is one of those "if you need it you'll know" sorts of things,
  and we don't need it.
* A pointer to a `VkInstance` value.
  This is an "out pointer", the new instance is written to this pointer instead of being in the return value itself.
  Meanwhile, the the "actual" return value is what indicates success or failure.
  This is somewhat clunkier than having a `Result<VkInstance, VkError>` output,
  but it's about the best you can do with a C ABI.

Before we can call `vkCreateInstance` we need to define the type.
We could define the function *only* in terms of raw pointers,
but by reading the docs we can use references and Option types to better signal when you can pass null or not.
```rust
pub type vkCreateInstance_t = unsafe extern "system" fn(
    pCreateInfo: &VkInstanceCreateInfo,
    pAllocator: Option<&VkAllocationCallbacks>,
    pInstance: &mut VkInstance
  ) -> VkResult;
```
The main time that you *can't* use references instead of pointers is with "list of" values.
If you have a list of things and you're supposed to pass a pointer to the first element,
then passing a reference instead would tag things with the wrong LLVM IR internally.
That's why `vkGetInstanceProcAddr` didn't use `&u8` for the function name argument.
Still, often enough a pointer is only to a single struct, and in those cases we can use references.

Unfortunately for us,
this one innocent little function pointer type is gonna *explode* our module with additional required definitions.

### VkInstanceCreateInfo

The [VkInstanceCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkInstanceCreateInfo.html)
struct follows a pattern we'll become familiar with as we see more Vulkan.
The first field is a `VkStructureType` entry.
This lets you read out of an unknown void pointer to see what type the rest of the struct is.
The next field is an example of such a void pointer,
it extends the info here in some way.
The other fields aren't too interesting,
we'll talk about them when we're actually filling one of these out.
```rust
/// [VkInstanceCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkInstanceCreateInfo.html)
#[repr(C)]
pub struct VkInstanceCreateInfo {
  pub sType: VkStructureType,
  pub pNext: *const c_void,
  pub flags: VkInstanceCreateFlags,
  pub pApplicationInfo: *const VkApplicationInfo,
  pub enabledLayerCount: u32,
  pub ppEnabledLayerNames: *const *const u8,
  pub enabledExtensionCount: u32,
  pub ppEnabledExtensionNames: *const *const u8,
}
impl Default for VkInstanceCreateInfo {
  fn default() -> Self {
    let mut x: Self = unsafe { core::mem::zeroed() };
    x.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
    x
  }
}
pub const VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO: VkStructureType =
  VkStructureType(1);
```
Because the `sType` field must *always* be a specific value for this type of struct,
we just go ahead and set it within the `Default` impl for the type.

Except that now we need to have [VkStructureType](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkStructureType.html).
This is a C-style enum.
A C-style enum in C is *basically* what Rust would just call "a list of `const` values".
The names defined by the enum are all just integer values,
and since integers can fairly freely convert between types automatically there's not much type enforcement.
We can improve the situation a little more by using the newtype pattern.
We're going to quickly get a lot of these C-styled enums, so let's make a macro for this.
For the moment, the macro will just define the type.
We won't declare all the names and values within the macro right now, though maybe we could later.
Also, I'll throw in the other enumerations we'll soon need too.
```rust
macro_rules! define_enumeration {
  ($(#[$m:meta])* $id:ident) => {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    $(#[$m])*
    pub struct $id(pub u32);
  };
}
define_enumeration!(VkInternalAllocationType);
define_enumeration!(VkStructureType);
define_enumeration!(VkSystemAllocationScope);
define_enumeration!(
  #[must_use]
  VkResult
);
```

### VkApplicationInfo

Okay now we need [VkApplicationInfo](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkApplicationInfo.html).
Look, another `VkStructureType` as the first field,
and then an extension pointer as well.
If you're disciplined about it you can have a fairly future-compatible C ABI interface.
```rust
/// [VkApplicationInfo](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkApplicationInfo.html)
#[repr(C)]
pub struct VkApplicationInfo {
  pub sType: VkStructureType,
  pub pNext: *const c_void,
  pub pApplicationName: *const u8,
  pub applicationVersion: u32,
  pub pEngineName: *const u8,
  pub engineVersion: u32,
  pub apiVersion: u32,
}
impl Default for VkApplicationInfo {
  fn default() -> Self {
    let mut x: Self = unsafe { core::mem::zeroed() };
    x.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
    x
  }
}
pub const VK_STRUCTURE_TYPE_APPLICATION_INFO: VkStructureType =
  VkStructureType(0);
```

### VkAllocationCallbacks

Okay so, I think that means we're up to the [VkAllocationCallbacks](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkAllocationCallbacks.html)
```rust
/// [VkAllocationCallbacks](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkAllocationCallbacks.html)
#[repr(C)]
pub struct VkAllocationCallbacks {
  pub pUserData: *mut c_void,
  pub pfnAllocation: PFN_vkAllocationFunction,
  pub pfnReallocation: PFN_vkReallocationFunction,
  pub pfnFree: PFN_vkFreeFunction,
  pub pfnInternalAllocation: PFN_vkInternalAllocationNotification,
  pub pfnInternalFree: PFN_vkInternalFreeNotification,
}
```
Yikes.
That's five new function types all at once, and they're kinda verbose to make in the first place.

Alright, time for another macro.
This one will let us more efficiently define aliases for the nullable and non-nullable forms of each function pointer.
```rust
macro_rules! define_fn_ptr {
  ($(#[$m:meta])* $pfn:ident<$t_name:ident> = Option<$raw_f:ty>) => {
    $(#[$m])*
    pub type $pfn = Option<$t_name>;
    $(#[$m])*
    pub type $t_name = $raw_f;
  };
}

define_fn_ptr!(
  /// [PFN_vkAllocationFunction](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/PFN_vkAllocationFunction.html)
  PFN_vkAllocationFunction<vkAllocationFunction_t> = Option<unsafe extern "system" fn(
    pUserData: *mut c_void,
    size: usize,
    alignment: usize,
    allocationScope: VkSystemAllocationScope,
  ) -> *mut c_void>
);

define_fn_ptr!(
  /// [PFN_vkReallocationFunction](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/PFN_vkReallocationFunction.html)
  PFN_vkReallocationFunction<vkReallocationFunction_t> = Option<unsafe extern "system" fn(
    pUserData: *mut c_void,
    pOriginal: *mut c_void,
    size: usize,
    alignment: usize,
    allocationScope: VkSystemAllocationScope,
  ) -> *mut c_void>
);

// And so on, look in the module in the github repository if you want to see them all.
// We don't use them in this lesson, so i'll skip the rest.
```

### VkInstance

The `VkInstance` type is what Vulkan calls a "handle".
It's just an opaque void pointer.

```rust
macro_rules! vk_define_handle {
  ($($id:ident),*) => {
    $(
      #[derive(Debug, Copy, Clone)]
      #[repr(transparent)]
      pub struct $id(*mut c_void);
      impl $id {
        pub const fn null() -> Self {
          Self(core::ptr::null_mut())
        }
        pub fn is_null(self) -> bool {
          self.0.is_null()
        }
      }
    )*
  };
}
vk_define_handle!(VkInstance);
```

### Getting the vkGetInstanceProcAddr pointer

And now that everything is all defined,
we can look up the function pointer in our program's `main` function:
```rust
  let vk_module_handle = load_library("vulkan-1.dll").unwrap();
  let vkGetInstanceProcAddr = unsafe {
    core::mem::transmute::<NonNull<c_void>, vkGetInstanceProcAddr_t>(
      get_proc_address(vk_module_handle, c_str!("vkGetInstanceProcAddr"))
        .unwrap(),
    )
  };
  let vkCreateInstance = unsafe {
    core::mem::transmute::<NonNull<c_void>, vkCreateInstance_t>(
      get_proc_address(vk_module_handle, c_str!("vkCreateInstance")).unwrap(),
    )
  };
```

Very good.

To actually *call* `vkCreateInstance` we'll need to have filled out a `VkInstanceCreateInfo` value.
To do that we'll need to know a little more about what the local Vulkan implementation has available.
Let's have a look at those other functions we can call *without* an instance.

## `vkEnumerateInstanceVersion`

This handy function lets us check on what the Vulkan version of our instance will be.
We can't request a version of Vulkan that's higher than this,
so it's an important thing to know.

Technically the function was added in VK 1.1,
so if only 1.0 is available, we won't even get a function pointer back.

The way that a Vulkan version works is that it's a single `u32` value,
with the `major.minor.patch` info packed into the bits of the `u32`.

Similar to how enums in C provide little type enforcement,
making the version values just be a `u32` is a little too fast and loose for me.
We'll once again do a newtype.
Because this isn't an "official" Vulkan type we'll call it "VulkanVersion" rather than just "VkVersion".
Hopefully the abnormality in name will give a programmer a clue that something is different here,
and then they'll check the docs and notice that this is an extra type we made ourselves.
```rust
/// Provides simple access to a vulkan version value.
/// 
/// This isn't an official Vulkan type, it's just a Rusty helper type.
///
/// See [39.2.1. Version Numbers](https://renderdoc.org/vkspec_chunked/chap40.html#extendingvulkan-coreversions-versionnumbers)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct VulkanVersion(pub u32);
impl VulkanVersion {
  pub const fn major(self) -> u32 {
    self.0 >> 22
  }
  pub const fn minor(self) -> u32 {
    (self.0 >> 12) & 0x3ff
  }
  pub const fn patch(self) -> u32 {
    self.0 & 0xfff
  }
  pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
    Self((major << 22) | (minor << 22) | patch)
  }
  pub const _1_0: VulkanVersion = VulkanVersion::new(1, 0, 0);
  pub const _1_1: VulkanVersion = VulkanVersion::new(1, 1, 0);
  pub const _1_2: VulkanVersion = VulkanVersion::new(1, 2, 0);
}
impl core::fmt::Debug for VulkanVersion {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    if f.alternate() {
      write!(f, "VulkanVersion({})", self.0)
    } else {
      write!(
        f,
        "VulkanVersion({major}.{minor}.{patch})",
        major = self.major(),
        minor = self.minor(),
        patch = self.patch(),
      )
    }
  }
}
```

Also we should define the function pointer type:
```rust
define_fn_ptr!(
  /// [vkEnumerateInstanceVersion](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceVersion.html)
  PFN_vkEnumerateInstanceVersion<vkEnumerateInstanceVersion_t> = Option<unsafe extern "system" fn(
    &mut VulkanVersion
  ) -> VkResult>
);
```

Also we should update the `apiVersion` field of `VkApplicationInfo` to use our newtype:
```rust
/// [VkApplicationInfo](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkApplicationInfo.html)
#[repr(C)]
pub struct VkApplicationInfo {
  pub sType: VkStructureType,
  pub pNext: *const c_void,
  pub pApplicationName: *const u8,
  pub applicationVersion: u32,
  pub pEngineName: *const u8,
  pub engineVersion: u32,
  pub apiVersion: VulkanVersion,
}
```

Now we'll *attempt* to call it in our code,
but remember if we don't find the function pointer we'll default the version to 1.0:
```rust
  let instance_version = unsafe {
    let p = vkGetInstanceProcAddr(
      VkInstance::null(),
      c_str!("vkEnumerateInstanceVersion").as_ptr(),
    );
    transmute::<_, PFN_vkEnumerateInstanceVersion>(p)
  }
  .map(|vkEnumerateInstanceVersion| {
    let mut v = VulkanVersion::default();
    let _ = unsafe { vkEnumerateInstanceVersion(&mut v) };
    v
  })
  .unwrap_or(VulkanVersion::_1_0);
  println!("vkEnumerateInstanceVersion reports: {:?}", instance_version);
```

Notice that there's a `VkResult` that we're ignoring.
If we check the spec we see this:

> **Note:**
> The intended behaviour of `vkEnumerateInstanceVersion` is that an implementation
> should not need to perform memory allocations and should unconditionally return
> `VK_SUCCESS`. The loader, and any enabled layers, may return
> `VK_ERROR_OUT_OF_HOST_MEMORY` in the case of a failed memory allocation.

We don't have a loader, and we don't have any layers on,
so we'll assume that there's a `VK_SUCCESS`.
We could assert on it or something, but I don't think that's really necessary.

And hey, now I can see that I've got Vulkan 1.1 installed. Neat.
Wait, the latest is 1.2, *sigh* okay I'll go update my drivers.
Alright, there we go, it shows Vulkan 1.2.162, that sounds right.

## `vkEnumerateInstanceLayerProperties`

Next let's use [vkEnumerateInstanceLayerProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceLayerProperties.html),
which will let us get information on the layers that are available to us.

We'll need the function type, and also we'll need to be able to tell "success" from "incomplete".

```rust
define_fn_ptr!(
  /// [vkEnumerateInstanceLayerProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceLayerProperties.html)
  PFN_vkEnumerateInstanceLayerProperties<vkEnumerateInstanceLayerProperties_t> = Option<unsafe extern "system" fn(
    pPropertyCount: &mut u32,
    pProperties: *mut VkLayerProperties
  ) -> VkResult>
);
```

This is one of those functions with more than one "mode" for how you can call it:
* If `pProperties` is null,
  the number of available properties will be written to `pPropertyCount`.
* If `pProperties` is non-null,
  it **must** point to a valid slice of `VkLayerProperties` values,
  and `pPropertyCount` must be the length of that slice.
  The slice will be overwritten with the various layer properties.
  If there's more layers than you provide space for in your slice it will write as much as it can and return `VK_INCOMPLETE`.

So first we need to be able to tell "success" from "incomplete".
```rust
/// Command successfully completed.
pub const VK_SUCCESS: VkResult = VkResult(0);
/// A return array was too small for the result.
pub const VK_INCOMPLETE: VkResult = VkResult(5);
```

And we'll need the struct for the layer info:
```rust
/// [VkLayerProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkLayerProperties.html)
#[repr(C)]
pub struct VkLayerProperties {
  pub layerName: [u8; VK_MAX_EXTENSION_NAME_SIZE],
  pub specVersion: VulkanVersion,
  pub implementationVersion: u32,
  pub description: [u8; VK_MAX_EXTENSION_NAME_SIZE],
}
impl core::fmt::Debug for VkLayerProperties {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    let name =
      str_from_null_terminated_byte_slice(&self.layerName).unwrap_or("");
    let the_description =
      str_from_null_terminated_byte_slice(&self.description).unwrap_or("");
    write!(f, "VkLayerProperties {{ name: {name:?}, spec: {spec:?}, impl: {implementation:?}, desc: {description:?} }}",
      name = name,
      spec = self.specVersion,
      implementation = self.implementationVersion,
      description = the_description,
    )
  }
}
```

That `str_from_null_terminated_byte_slice` helper will be used a lot,
and it's useful outside of Vulkan too,
so we'll define it in `lib.rs`:
```rust
pub fn str_from_null_terminated_byte_slice(
  bytes: &[u8],
) -> Result<&str, core::str::Utf8Error> {
  let terminal_position =
    bytes.iter().copied().position(|u| u == 0).unwrap_or(bytes.len());
  core::str::from_utf8(&bytes[..terminal_position])
}
```

Now we can check what layers are available:
```rust
  let available_layers = unsafe {
    let p = vkGetInstanceProcAddr(
      VkInstance::null(),
      c_str!("vkEnumerateInstanceLayerProperties").as_ptr(),
    );
    transmute::<_, PFN_vkEnumerateInstanceLayerProperties>(p)
  }
  .map(|vkEnumerateInstanceLayerProperties| {
    let mut property_count: u32 = 0;
    let _ = unsafe {
      vkEnumerateInstanceLayerProperties(&mut property_count, null_mut())
    };
    let mut v: Vec<VkLayerProperties> =
      Vec::with_capacity(property_count as usize);
    let got = unsafe {
      vkEnumerateInstanceLayerProperties(&mut property_count, v.as_mut_ptr())
    };
    if got == VK_SUCCESS || got == VK_INCOMPLETE {
      unsafe { v.set_len(property_count as usize) }
    }
    v
  })
  .unwrap_or(Vec::new());
  println!(
    "vkEnumerateInstanceLayerProperties reports: {:#?}",
    available_layers
  );
```

For me it shows:
```
vkEnumerateInstanceVersion reports: VulkanVersion { major: 1, minor: 2, patch: 162 }
vkEnumerateInstanceLayerProperties reports: [
    VkLayerProperties { name: "VK_LAYER_NV_optimus", spec: VulkanVersion { major: 1, minor: 2, patch: 155 }, impl: 1, desc: "NVIDIA Optimus layer" },
    VkLayerProperties { name: "VK_LAYER_RENDERDOC_Capture", spec: VulkanVersion { major: 1, minor: 2, patch: 131 }, impl: 9, desc: "Debugging capture layer for RenderDoc" },
    VkLayerProperties { name: "VK_LAYER_OBS_HOOK", spec: VulkanVersion { major: 1, minor: 2, patch: 131 }, impl: 1, desc: "Open Broadcaster Software hook" },
    VkLayerProperties { name: "VK_LAYER_VALVE_steam_overlay", spec: VulkanVersion { major: 1, minor: 2, patch: 136 }, impl: 1, desc: "Steam Overlay Layer" },
    VkLayerProperties { name: "VK_LAYER_VALVE_steam_fossilize", spec: VulkanVersion { major: 1, minor: 2, patch: 136 }, impl: 1, desc: "Steam Pipeline Caching Layer" },
]
```

Sure seems to work.

But there's no validation layers, what the heck?

Oh: https://github.com/KhronosGroup/Vulkan-Ecosystem/issues/38

And I *did* just update from a very old driver version.
Well if I re-install the SDK now, will that fix it?

```
vkEnumerateInstanceLayerProperties reports: [
    VkLayerProperties { name: "VK_LAYER_NV_optimus", spec: VulkanVersion(1.2.155), impl: 1, desc: "NVIDIA Optimus layer" },
    VkLayerProperties { name: "VK_LAYER_RENDERDOC_Capture", spec: VulkanVersion(1.2.131), impl: 9, desc: "Debugging capture layer for RenderDoc" },
    VkLayerProperties { name: "VK_LAYER_OBS_HOOK", spec: VulkanVersion(1.2.131), impl: 1, desc: "Open Broadcaster Software hook" },
    VkLayerProperties { name: "VK_LAYER_VALVE_steam_overlay", spec: VulkanVersion(1.2.136), impl: 1, desc: "Steam Overlay Layer" },
    VkLayerProperties { name: "VK_LAYER_VALVE_steam_fossilize", spec: VulkanVersion(1.2.136), impl: 1, desc: "Steam Pipeline Caching Layer" },
    VkLayerProperties { name: "VK_LAYER_LUNARG_api_dump", spec: VulkanVersion(1.2.162), impl: 2, desc: "LunarG API dump layer" },
    VkLayerProperties { name: "VK_LAYER_LUNARG_device_simulation", spec: VulkanVersion(1.2.162), impl: 1, desc: "LunarG device simulation layer" },
    VkLayerProperties { name: "VK_LAYER_LUNARG_gfxreconstruct", spec: VulkanVersion(1.2.162), impl: 36869, desc: "GFXReconstruct Capture Layer Version 0.9.5" },
    VkLayerProperties { name: "VK_LAYER_KHRONOS_validation", spec: VulkanVersion(1.2.162), impl: 1, desc: "Khronos Validation Layer" },
    VkLayerProperties { name: "VK_LAYER_LUNARG_monitor", spec: VulkanVersion(1.2.162), impl: 1, desc: "Execution Monitoring Layer" },
    VkLayerProperties { name: "VK_LAYER_LUNARG_screenshot", spec: VulkanVersion(1.2.162), impl: 1, desc: "LunarG image capture layer" },
]
```
Hey, that does look better.

So if you don't see the LunarG and Khronos layers in your list,
be sure to check your SDK installation.

## `vkEnumerateInstanceExtensionProperties`

The [vkEnumerateInstanceExtensionProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceExtensionProperties.html)
function is similar to the last one.
This gets extensions info instead.

```rust
define_fn_ptr!(
  /// [vkEnumerateInstanceExtensionProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceExtensionProperties.html):
  /// Returns up to the requested number of global extension properties.
  PFN_vkEnumerateInstanceExtensionProperties<vkEnumerateInstanceExtensionProperties_t> = Option<unsafe extern "system" fn(
    pLayerName: *const u8,
    pPropertyCount: &mut u32,
    pProperties: *mut VkExtensionProperties
  ) -> VkResult>
);

/// [VkExtensionProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkExtensionProperties.html)
#[repr(C)]
pub struct VkExtensionProperties {
  pub extensionName: [u8; VK_MAX_EXTENSION_NAME_SIZE],
  pub specVersion: u32,
}
impl core::fmt::Debug for VkExtensionProperties {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    let name =
      str_from_null_terminated_byte_slice(&self.extensionName).unwrap_or("");
    write!(
      f,
      "VkExtensionProperties {{ name: {name:?}, spec: {spec:?} }}",
      name = name,
      spec = self.specVersion,
    )
  }
}
```

There's a new argument this time.
The first argument to the function can name a layer, or it can be null.
* If you name a layer, then the returned info is for extensions available for that layer.
* If you *don't* name a layer, the returned info is for extensions available in the vulkan implementation itself,
  including any implicitly enabled layers.
The second and third arguments are a len and pointer like we saw with `vkEnumerateInstanceLayerProperties`.
As before, if we call once with a null pointer we can get the count of how many things.
Then we call a second time once we have a suitably sized buffer.

Since we want to use this vulkan function both with a null pointer,
and also once for each layer,
Let's bother to make a "rusty wrapper" this time:
```rust
#[allow(bad_style)]
pub fn do_vkEnumerateInstanceExtensionProperties(
  f: vkEnumerateInstanceExtensionProperties_t, layer_name: Option<&[u8]>,
) -> Result<Vec<VkExtensionProperties>, VkResult> {
  let layer_name_ptr = match layer_name {
    Some(layer) => {
      layer.iter().copied().position(|u| u == 0).unwrap();
      layer.as_ptr()
    }
    None => null(),
  };
  let mut ext_count: u32 = 0;
  let _ = unsafe { f(layer_name_ptr, &mut ext_count, null_mut()) };
  let mut v: Vec<VkExtensionProperties> =
    Vec::with_capacity(ext_count as usize);
  let got = unsafe { f(layer_name_ptr, &mut ext_count, v.as_mut_ptr()) };
  if got == VK_SUCCESS {
    unsafe { v.set_len(ext_count as usize) }
    Ok(v)
  } else {
    Err(got)
  }
}
```

And now we can call it.
Unfortunately, the long name of the vulkan function means that this part of the code gets fairly tall.
```rust
  let _ = unsafe {
    let p = vkGetInstanceProcAddr(
      VkInstance::null(),
      c_str!("vkEnumerateInstanceExtensionProperties").as_ptr(),
    );
    transmute::<_, PFN_vkEnumerateInstanceExtensionProperties>(p)
  }
  .map(|f| {
    match do_vkEnumerateInstanceExtensionProperties(f, None) {
      Ok(v) => {
        println!("vkEnumerateInstanceExtensionProperties(root): {:#?}", v)
      }
      Err(e) => println!(
        "vkEnumerateInstanceExtensionProperties(root): failed. {e:?}",
        e = e
      ),
    }
    for layer in available_layers.iter() {
      let name = str_from_null_terminated_byte_slice(&layer.layerName)
        .unwrap_or("unknown");
      match do_vkEnumerateInstanceExtensionProperties(f, Some(&layer.layerName))
      {
        Ok(v) => println!(
          "vkEnumerateInstanceExtensionProperties({name}): {vec:#?}",
          name = name,
          vec = v
        ),
        Err(e) => println!(
          "vkEnumerateInstanceExtensionProperties({name}): failed. {e:?}",
          name = name,
          e = e
        ),
      }
    }
  });
```

And now we can print out our extension properties.
On my machine it looks like this:
```
vkEnumerateInstanceExtensionProperties(root): [
    VkExtensionProperties { name: "VK_KHR_device_group_creation", spec: 1 },
    VkExtensionProperties { name: "VK_KHR_display", spec: 23 },
    VkExtensionProperties { name: "VK_KHR_external_fence_capabilities", spec: 1 },
    VkExtensionProperties { name: "VK_KHR_external_memory_capabilities", spec: 1 },
    VkExtensionProperties { name: "VK_KHR_external_semaphore_capabilities", spec: 1 },
    VkExtensionProperties { name: "VK_KHR_get_display_properties2", spec: 1 },
    VkExtensionProperties { name: "VK_KHR_get_physical_device_properties2", spec: 2 },
    VkExtensionProperties { name: "VK_KHR_get_surface_capabilities2", spec: 1 },
    VkExtensionProperties { name: "VK_KHR_surface", spec: 25 },
    VkExtensionProperties { name: "VK_KHR_surface_protected_capabilities", spec: 1 },
    VkExtensionProperties { name: "VK_KHR_win32_surface", spec: 6 },
    VkExtensionProperties { name: "VK_EXT_debug_report", spec: 9 },
    VkExtensionProperties { name: "VK_EXT_debug_utils", spec: 2 },
    VkExtensionProperties { name: "VK_EXT_swapchain_colorspace", spec: 4 },
    VkExtensionProperties { name: "VK_NV_external_memory_capabilities", spec: 1 },
]
vkEnumerateInstanceExtensionProperties(VK_LAYER_NV_optimus): []
vkEnumerateInstanceExtensionProperties(VK_LAYER_RENDERDOC_Capture): [
    VkExtensionProperties { name: "VK_EXT_debug_utils", spec: 1 },
]
vkEnumerateInstanceExtensionProperties(VK_LAYER_OBS_HOOK): []
vkEnumerateInstanceExtensionProperties(VK_LAYER_VALVE_steam_overlay): []
vkEnumerateInstanceExtensionProperties(VK_LAYER_VALVE_steam_fossilize): []
```

## Next Time We'll Create an Instance!

It might seems like a let-down,
but that's it for now.

The exact info that you want to put in to create an instance depends on your usage,
so as we do a little more each time in the next few lessons we'll be creating separate instances for each scenario.
