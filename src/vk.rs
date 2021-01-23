#![allow(bad_style)]

use core::ffi::c_void;

use crate::str_from_null_terminated_byte_array;

/// Maximum size of an extension name.
pub const VK_MAX_EXTENSION_NAME_SIZE: usize = 256;

/// Maximum size of an extension description.
pub const VK_MAX_DESCRIPTION_SIZE: usize = 256;

/// Command successfully completed.
pub const VK_SUCCESS: VkResult = VkResult(0);
/// A return array was too small for the result.
pub const VK_INCOMPLETE: VkResult = VkResult(5);

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

macro_rules! define_enumeration {
  ($id:ident) => {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct $id(pub u32);
  };
}
define_enumeration!(VkInternalAllocationType);
define_enumeration!(VkStructureType);
define_enumeration!(VkSystemAllocationScope);
define_enumeration!(VkResult);

macro_rules! define_flags {
  ($id:ident) => {
    #[derive(Debug, Copy, Clone)]
    #[repr(transparent)]
    pub struct $id(pub u32);
  };
}
define_flags!(VkInstanceCreateFlags);

macro_rules! define_fn_ptr {
  ($(#[$m:meta])* $pfn:ident<$t_name:ident> = Option<$raw_f:ty>) => {
    $(#[$m])*
    pub type $pfn = Option<$t_name>;
    $(#[$m])*
    pub type $t_name = $raw_f;
  };
}
define_fn_ptr!(
  /// [PFN_vkVoidFunction](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/PFN_vkVoidFunction.html)
  PFN_vkVoidFunction<system_void_fn> = Option<unsafe extern "system" fn()>
);

define_fn_ptr!(
  /// [vkGetInstanceProcAddr](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkGetInstanceProcAddr.html)
  PFN_vkGetInstanceProcAddr<vkGetInstanceProcAddr_t> = Option<unsafe extern "system" fn(
    instance: VkInstance,
    p_name: *const u8,
  ) -> PFN_vkVoidFunction>
);

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

define_fn_ptr!(
  /// [PFN_vkFreeFunction](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/PFN_vkFreeFunction.html)
  PFN_vkFreeFunction<vkFreeFunction_t> = Option<unsafe extern "system" fn(
    pUserData: *mut c_void,
    pMemory: *mut c_void
  )>
);

define_fn_ptr!(
  /// [PFN_vkInternalAllocationNotification](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/PFN_vkInternalAllocationNotification.html)
  PFN_vkInternalAllocationNotification<vkInternalAllocationNotification_t> = Option<unsafe extern "system" fn(
    pUserData: *mut c_void,
    size: usize,
    allocationType: VkInternalAllocationType,
    allocationScope: VkSystemAllocationScope,
  )>
);

define_fn_ptr!(
  /// [PFN_vkInternalFreeNotification](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/PFN_vkInternalFreeNotification.html)
  PFN_vkInternalFreeNotification<vkInternalFreeNotification_t> = Option<unsafe extern "system" fn(
    pUserData: *mut c_void,
    size: usize,
    allocationType: VkInternalAllocationType,
    allocationScope: VkSystemAllocationScope,
  )>
);

define_fn_ptr!(
  /// [vkCreateInstance](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCreateInstance.html)
  PFN_vkCreateInstance<vkCreateInstance_t> = Option<unsafe extern "system" fn(
    pCreateInfo: &VkInstanceCreateInfo,
    pAllocator: Option<&VkAllocationCallbacks>,
    pInstance: &mut VkInstance
  )>
);

define_fn_ptr!(
  /// [vkEnumerateInstanceVersion](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceVersion.html)
  PFN_vkEnumerateInstanceVersion<vkEnumerateInstanceVersion_t> = Option<unsafe extern "system" fn(
    &mut VulkanVersion
  ) -> VkResult>
);

define_fn_ptr!(
  /// [vkEnumerateInstanceLayerProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceLayerProperties.html):
  /// Returns up to the requested number of global layer properties.
  PFN_vkEnumerateInstanceLayerProperties<vkEnumerateInstanceLayerProperties_t> = Option<unsafe extern "system" fn(
    pPropertyCount: &mut u32,
    pProperties: *mut VkLayerProperties
  ) -> VkResult>
);

define_fn_ptr!(
  /// [vkEnumerateInstanceExtensionProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkEnumerateInstanceExtensionProperties.html):
  /// Returns up to the requested number of global extension properties.
  PFN_vkEnumerateInstanceExtensionProperties<vkEnumerateInstanceExtensionProperties_t> = Option<unsafe extern "system" fn(
    pLayerName: *const u8,
    pPropertyCount: &mut u32,
    pProperties: *mut VkExtensionProperties
  ) -> VkResult>
);

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
  pub const fn make(major: u32, minor: u32, patch: u32) -> Self {
    Self((major << 22) | (minor << 22) | patch)
  }
  pub const _1_0: VulkanVersion = VulkanVersion::make(1, 0, 0);
  pub const _1_1: VulkanVersion = VulkanVersion::make(1, 1, 0);
  pub const _1_2: VulkanVersion = VulkanVersion::make(1, 2, 0);
}
impl core::fmt::Debug for VulkanVersion {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    if f.alternate() {
      write!(f, "VulkanVersion({})", self.0)
    } else {
      write!(
        f,
        "VulkanVersion {{ major: {major}, minor: {minor}, patch: {patch} }}",
        major = self.major(),
        minor = self.minor(),
        patch = self.patch(),
      )
    }
  }
}

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
      str_from_null_terminated_byte_array(&self.layerName).unwrap_or("");
    let the_description =
      str_from_null_terminated_byte_array(&self.description).unwrap_or("");
    write!(f, "VkLayerProperties {{ name: {name:?}, spec: {spec:?}, impl: {implementation:?}, desc: {description:?} }}",
      name = name,
      spec = self.specVersion,
      implementation = self.implementationVersion,
      description = the_description,
    )
  }
}

/// [VkExtensionProperties](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkExtensionProperties.html)
#[repr(C)]
pub struct VkExtensionProperties {
  pub extensionName: [u8; VK_MAX_EXTENSION_NAME_SIZE],
  pub specVersion: u32,
}
impl core::fmt::Debug for VkExtensionProperties {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    let name =
      str_from_null_terminated_byte_array(&self.extensionName).unwrap_or("");
    write!(
      f,
      "VkExtensionProperties {{ name: {name:?}, spec: {spec:?} }}",
      name = name,
      spec = self.specVersion,
    )
  }
}
