
# Vulkan Compute

The "compute" ability of Vulkan lets you make your GPU perform a calculation for you.
Not all tasks are well suited to running on the GPU.
When the task is highly parallel instead of highly sequential,
then GPU compute might help you solve it quickly.
If you're familiar with SIMD, you can broadly think of the GPU as being "more intense SIMD".

Note: This lesson brought to you based on a C++ version by [Neil Henning](https://www.duskborn.com/posts/a-simple-vulkan-compute-example/)

## Background

Before you begin this lesson you should have read the appropriate previous lesson for your platform about how to get a `vkGetInstanceProcAddr` pointer.
You should also have read the [Using `vkGetInstanceProcAddr`](using_vkGetInstanceProcAddr.md) lesson.

## Creating Our Instance

To create an instance suitable for using with compute we don't need to do anything special.
That's why we're doing this as our Vulkan warm-up.

First let's get out that function pointer:
```rust
  let instance = {
    let vkCreateInstance = unsafe {
      let p = vkGetInstanceProcAddr(
        VkInstance::null(),
        c_str!("vkCreateInstance").as_ptr(),
      );
      transmute::<_, PFN_vkCreateInstance>(p)
    }
    .unwrap();

    // TODO
  };
```

Now we need to get all the layers we're going to use.
In our case, if debug asserts are on we'll try and use the `VK_LAYER_KHRONOS_validation` layer.
To know if it's available, we just look in that `available_layers` vector we made earlier.
```rust
    // compute desired layers
    let mut desired_layer_names = Vec::new();
    for layer in available_layers.iter() {
      if cfg!(debug_assertions)
        && str_from_null_terminated_byte_slice(&layer.layerName)
          == Ok("VK_LAYER_KHRONOS_validation")
      {
        desired_layer_names.push(layer.layerName);
      }
    }
    let desired_layer_pointers: Vec<*const u8> =
      desired_layer_names.iter().map(|n| n.as_ptr()).collect();
    
    // TODO
```

Extensions is the same idea, but since we didn't make a list of all our available extensions we have to go look it up again:
```rust
    // compute desired extensions
    let mut desired_extension_names = Vec::new();
    unsafe {
      let p = vkGetInstanceProcAddr(
        VkInstance::null(),
        c_str!("vkEnumerateInstanceExtensionProperties").as_ptr(),
      );
      transmute::<_, PFN_vkEnumerateInstanceExtensionProperties>(p)
    }
    .map(|f| match do_vkEnumerateInstanceExtensionProperties(f, None) {
      Ok(v) => {
        for extension in v.iter() {
          if cfg!(debug_assertions)
            && str_from_null_terminated_byte_slice(&extension.extensionName)
              == Ok("VK_EXT_debug_utils")
          {
            desired_extension_names.push(extension.extensionName);
          }
        }
      }
      Err(_) => (),
    });
    let desired_extension_pointers: Vec<*const u8> =
      desired_extension_names.iter().map(|n| n.as_ptr()).collect();
    
    // TODO
```

Finally, we can fill out the creation struct and make the instance.
Note that the `VkApplicationInfo` is totally optional,
but telling Vulkan what API version you're targeting helps the validation layer tell you when you messed up.
```rust
    // do the call
    let app_info = VkApplicationInfo {
      apiVersion: VulkanVersion::new(1, 2, 0),
      ..Default::default()
    };
    let create_info = VkInstanceCreateInfo {
      pApplicationInfo: &app_info,
      enabledLayerCount: desired_layer_pointers.len() as _,
      ppEnabledLayerNames: desired_layer_pointers.as_ptr(),
      enabledExtensionCount: desired_extension_pointers.len() as _,
      ppEnabledExtensionNames: desired_extension_pointers.as_ptr(),
      ..Default::default()
    };
    let mut instance = VkInstance::null();
    let r = unsafe { vkCreateInstance(&create_info, None, &mut instance) };
    if r != VK_SUCCESS {
      panic!("instance creation failure: {:?}", r);
    }
    instance
  };
```

## Checking Physical Devices (available to an Instance)

Now that we have a valid instance, we can access a little more of the Vulkan API.

To proceed further we need to select a **Physical Device**.
A physical device is often a graphics card,
but it can also be anything else that can do "Vulkan stuff".

To check what physical devices are available, we'll use
[vkEnumeratePhysicalDevices](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumeratePhysicalDevices.html).
It works approximately how `vkEnumerateInstanceLayerProperties` worked:

1) You call with a null slice pointer and it tells you how many physical devices will be enumerated.
2) You allocate space for however many it told you.
3) You call again with the allocated slice pointer and it writes all the info.

This is a very common C idiom because it allows the *caller* to better control the allocation.

```rust
// in vk.rs

define_fn_ptr!(
  /// [vkEnumeratePhysicalDevices](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumeratePhysicalDevices.html)
  PFN_vkEnumeratePhysicalDevices<vkEnumeratePhysicalDevices_t> = Option<unsafe extern "system" fn(
    instance: VkInstance,
    pPhysicalDeviceCount: &mut u32,
    pPhysicalDevices: *mut VkPhysicalDevice,
  ) -> VkResult>
);

vk_define_handle!(VkPhysicalDevice);
```

This time when we look up the function pointer, we pass our instance instead of passing a null.
Other than that, it's not too exciting.
```rust
  let physical_device_list = {
    let vkEnumeratePhysicalDevices = unsafe {
      let p = vkGetInstanceProcAddr(
        instance,
        c_str!("vkEnumeratePhysicalDevices").as_ptr(),
      );
      transmute::<_, PFN_vkEnumeratePhysicalDevices>(p)
    }
    .unwrap();

    let mut physical_device_count: u32 = 0;
    assert_eq!(
      unsafe {
        vkEnumeratePhysicalDevices(
          instance,
          &mut physical_device_count,
          null_mut(),
        )
      },
      VK_SUCCESS
    );
    let mut physical_devices: Vec<VkPhysicalDevice> =
      Vec::with_capacity(physical_device_count as _);
    assert_eq!(
      unsafe {
        vkEnumeratePhysicalDevices(
          instance,
          &mut physical_device_count,
          physical_devices.as_mut_ptr(),
        )
      },
      VK_SUCCESS
    );
    unsafe { physical_devices.set_len(physical_device_count as _) };
    physical_devices
  };
  println!(
    "vkEnumeratePhysicalDevices: {} Physical Device{}.",
    physical_device_list.len(),
    if physical_device_list.len() == 1 { "" } else { "s" }
  );
```
On my desktop there's just the 1 physical device.

Once we've got some handles to the physical devices,
there's a number of functions we could use to get more information.
* [vkGetPhysicalDeviceFeatures](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceFeatures.html)
* [vkGetPhysicalDeviceFormatProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceFormatProperties.html)
* [vkGetPhysicalDeviceImageFormatProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html)
* [vkGetPhysicalDeviceProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceProperties.html)
* [vkGetPhysicalDeviceQueueFamilyProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceQueueFamilyProperties.html)
* [vkGetPhysicalDeviceMemoryProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceMemoryProperties.html)

We're not going to use all of these right now,
these are just things that you could do if you wanted.

## Checking Queue Families (of each Physical Device)

One thing we do need to check for sure is if a physical device supports a **Queue Family** that will allow compute.
A queue family is how you get a queue,
and a queue is how you send work to Vulkan to make it do something.
If there aren't any queue families that support compute,
then we can't send any compute work to vulkan.

It's extremely unlikely that *none* of the queue families within a physical device will support compute,
but it's entirely possible that *only some* of the queue families will support compute.

To check what queue families can do what,
we use [vkGetPhysicalDeviceQueueFamilyProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceQueueFamilyProperties.html)
This is another "call to check your required buffer size, allocate, then call again with a buffer pointer" thing.

```rust
// vk.rs

define_fn_ptr!(
  /// [vkGetPhysicalDeviceQueueFamilyProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetPhysicalDeviceQueueFamilyProperties.html)
  PFN_vkGetPhysicalDeviceQueueFamilyProperties<vkGetPhysicalDeviceQueueFamilyProperties_t> = Option<unsafe extern "system" fn(
    physicalDevice: VkPhysicalDevice,
    pQueueFamilyPropertyCount: &mut u32,
    pQueueFamilyProperties: *mut VkQueueFamilyProperties,
  )> // Note!! Does NOT return a VkResult this time!
);

/// [VkQueueFamilyProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkQueueFamilyProperties.html)
#[repr(C)]
#[derive(Debug)]
pub struct VkQueueFamilyProperties {
  pub queueFlags: VkQueueFlags,
  pub queueCount: u32,
  pub timestampValidBits: u32,
  pub minImageTransferGranularity: VkExtent3D,
}

/// [VkExtent3D](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkExtent3D.html)
#[repr(C)]
#[derive(Debug)]
pub struct VkExtent3D {
  pub width: u32,
  pub height: u32,
  pub depth: u32,
}

define_flags!(VkQueueFlags);
```

Okay so let's give that a call:
```rust
let vkGetPhysicalDeviceQueueFamilyProperties = unsafe {
    let p = vkGetInstanceProcAddr(
      instance,
      c_str!("vkGetPhysicalDeviceQueueFamilyProperties").as_ptr(),
    );
    transmute::<_, PFN_vkGetPhysicalDeviceQueueFamilyProperties>(p)
  }
  .unwrap();
  for physical_device in physical_device_list.iter().copied() {
    let mut queue_family_property_count: u32 = 0;
    unsafe {
      vkGetPhysicalDeviceQueueFamilyProperties(
        physical_device,
        &mut queue_family_property_count,
        null_mut(),
      )
    };
    let mut queue_family_properties: Vec<VkQueueFamilyProperties> =
      Vec::with_capacity(queue_family_property_count as _);
    unsafe {
      vkGetPhysicalDeviceQueueFamilyProperties(
        physical_device,
        &mut queue_family_property_count,
        queue_family_properties.as_mut_ptr(),
      )
    };
    unsafe {
      queue_family_properties.set_len(queue_family_property_count as _)
    };
    println!(
      "vkGetPhysicalDeviceQueueFamilyProperties({physical_device:?}): {properties:#?}",
      physical_device = physical_device,
      properties = queue_family_properties,
    );
  }
```

Hmm, and then the output looks like...
```
vkGetPhysicalDeviceQueueFamilyProperties(VkPhysicalDevice(0x1ea1f476a30)): [
    VkQueueFamilyProperties {
        queueFlags: VkQueueFlags(
            15,
        ),
        queueCount: 16,
        timestampValidBits: 64,
        minImageTransferGranularity: VkExtent3D {
            width: 1,
            height: 1,
            depth: 1,
        },
    },
    VkQueueFamilyProperties {
        queueFlags: VkQueueFlags(
            12,
        ),
        queueCount: 2,
        timestampValidBits: 64,
        minImageTransferGranularity: VkExtent3D {
            width: 1,
            height: 1,
            depth: 1,
        },
    },
    VkQueueFamilyProperties {
        queueFlags: VkQueueFlags(
            14,
        ),
        queueCount: 8,
        timestampValidBits: 64,
        minImageTransferGranularity: VkExtent3D {
            width: 1,
            height: 1,
            depth: 1,
        },
    },
]
```

Which, isn't the most helpful.

Remember how we kinda *skipped* making the `define_flags!` macro very useful?
Better fix that.

```rust
macro_rules! define_flags {
  ($id:ident) => {
    #[derive(Copy, Clone)]
    #[repr(transparent)]
    pub struct $id(pub u32);
    impl core::ops::BitAnd for $id {
      type Output = Self;
      fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
      }
    }
    impl core::ops::BitOr for $id {
      type Output = Self;
      fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
      }
    }
    impl core::ops::BitXor for $id {
      type Output = Self;
      fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
      }
    }
    impl core::ops::Not for $id {
      type Output = Self;
      fn not(self) -> Self::Output {
        Self(!self.0)
      }
    }
  };
}
define_flags!(VkInstanceCreateFlags);
impl core::fmt::Debug for VkInstanceCreateFlags {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VkInstanceCreateFlags")
  }
}
define_flags!(VkQueueFlags);
impl core::fmt::Debug for VkQueueFlags {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VkQueueFlags {{")?;
    let mut printed = false;
    if (*self & VK_QUEUE_GRAPHICS_BIT).0 != 0 {
      write!(f, "Graphics")?;
      printed = true;
    }
    if (*self & VK_QUEUE_COMPUTE_BIT).0 != 0 {
      write!(f, "{}Compute", if printed { ", " } else { "" })?;
      printed = true;
    }
    if (*self & VK_QUEUE_TRANSFER_BIT).0 != 0 {
      write!(f, "{}Transfer", if printed { ", " } else { "" })?;
      printed = true;
    }
    if (*self & VK_QUEUE_SPARSE_BINDING_BIT).0 != 0 {
      write!(f, "{}Sparse_Binding", if printed { ", " } else { "" })?;
      printed = true;
    }
    if (*self & VK_QUEUE_PROTECTED_BIT).0 != 0 {
      write!(f, "{}Protected", if printed { ", " } else { "" })?;
    }
    write!(f, "}}")
  }
}
```

Ah ha, and now we can actually understand our output:
```
vkGetPhysicalDeviceQueueFamilyProperties(VkPhysicalDevice(0x2b0bc40f030)): [
    VkQueueFamilyProperties {
        queueFlags: VkQueueFlags {Graphics, Compute, Transfer, Sparse_Binding},
        queueCount: 16,
        timestampValidBits: 64,
        minImageTransferGranularity: VkExtent3D {
            width: 1,
            height: 1,
            depth: 1,
        },
    },
    VkQueueFamilyProperties {
        queueFlags: VkQueueFlags {Transfer, Sparse_Binding},
        queueCount: 2,
        timestampValidBits: 64,
        minImageTransferGranularity: VkExtent3D {
            width: 1,
            height: 1,
            depth: 1,
        },
    },
    VkQueueFamilyProperties {
        queueFlags: VkQueueFlags {Compute, Transfer, Sparse_Binding},
        queueCount: 8,
        timestampValidBits: 64,
        minImageTransferGranularity: VkExtent3D {
            width: 1,
            height: 1,
            depth: 1,
        },
    },
]
```

And this is a normal looking list of queue families for an nVidia graphics card (which reports slightly differently compared to some other graphics cards).
Each queue family is for specialized hardware.
Without checking any other properties of the queue families,
it's *likely* that the queue family with `Graphics` is the "general stuff" family,
the one with only `Transfer`/`Sparse_Binding` probably lets you use the GPU's DMA unit for memory transfers,
and the `Compute`/`Transfer`/`Sparse_Binding` family lets us use spare compute power when it's available.
Carefully using all queue families to their maximum potential and get the "most" out of the hardware is "just" an optimization problem.
It's at least as complex as trying to get the most out of the CPU.

As with any optimization problem,
the best choice is usually to just take reasonable default actions without worrying too much about it,
and then fix up the performance later when it's actually a problem.
In our case,
the reasonable default action is to just use the "general stuff" queue family,
since our workload is going to be very small anyway.

## Creating A Device (from a Physical Device)

TODO
