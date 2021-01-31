#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core::{
  mem::transmute,
  ptr::{null, null_mut, NonNull},
};
use triangle_from_scratch::{vk::*, win32::*, *};

#[allow(non_snake_case)]
fn main() {
  let sample_window_class = "Sample Window Class";
  let sample_window_class_wn = wide_null(sample_window_class);

  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(window_procedure);
  wc.hInstance = get_process_handle();
  wc.lpszClassName = sample_window_class_wn.as_ptr();
  wc.hCursor = load_predefined_cursor(IDCursor::Arrow).unwrap();

  let _atom = unsafe { register_class(&wc) }.unwrap();

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

  let instance = {
    let vkCreateInstance = unsafe {
      let p = vkGetInstanceProcAddr(
        VkInstance::null(),
        c_str!("vkCreateInstance").as_ptr(),
      );
      transmute::<_, PFN_vkCreateInstance>(p)
    }
    .unwrap();

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

  let _previously_visible = unsafe { ShowWindow(hwnd, SW_SHOW) };

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
}

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

pub unsafe extern "system" fn window_procedure(
  hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
  match msg {
    WM_NCCREATE => {
      println!("NC Create");
      let createstruct: *mut CREATESTRUCTW = lparam as *mut _;
      if createstruct.is_null() {
        return 0;
      }
      let ptr = (*createstruct).lpCreateParams as *mut i32;
      return set_window_userdata::<i32>(hwnd, ptr).is_ok() as LRESULT;
    }
    WM_CREATE => println!("Create"),
    WM_CLOSE => {
      let _success = DestroyWindow(hwnd);
    }
    WM_DESTROY => {
      match get_window_userdata::<i32>(hwnd) {
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
      post_quit_message(0);
    }
    WM_PAINT => {
      match get_window_userdata::<i32>(hwnd) {
        Ok(ptr) if !ptr.is_null() => {
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
    _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
  }
  0
}
