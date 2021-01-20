#![allow(non_camel_case_types)]

use super::c_char;
use core::ffi::c_void;

macro_rules! VK_DEFINE_HANDLE {
  ($id:ident) => {
    #[repr(transparent)]
    pub struct $id(*mut c_void);
  };
}

VK_DEFINE_HANDLE!(VkInstance);

macro_rules! VK_DEFINE_NON_DISPATCHABLE_HANDLE {
  ($id:ident) => {
    #[repr(transparent)]
    pub struct $id(u64);
  };
}

VK_DEFINE_NON_DISPATCHABLE_HANDLE!(VkDeviceMemory);

type PFN_vkVoidFunction = Option<unsafe extern "system" fn() -> c_void>;

pub type PFN_vkGetInstanceProcAddr = Option<
  unsafe extern "system" fn(
    instance: VkInstance,
    p_name: *const c_char,
  ) -> PFN_vkVoidFunction,
>;
