#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core::ptr::{null, null_mut, NonNull};
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

  let instance_version =
    get_proc_address(vk_module_handle, c_str!("vkEnumerateInstanceVersion"))
      .map(|nn| {
        let vkEnumerateInstanceVersion = unsafe {
          core::mem::transmute::<NonNull<c_void>, vkEnumerateInstanceVersion_t>(
            nn,
          )
        };
        let mut v = VulkanVersion::default();
        let _ = unsafe { vkEnumerateInstanceVersion(&mut v) };
        v
      })
      .unwrap_or(VulkanVersion::_1_0);
  println!("vkEnumerateInstanceVersion reports: {:?}", instance_version);

  let available_layers = get_proc_address(
    vk_module_handle,
    c_str!("vkEnumerateInstanceLayerProperties"),
  )
  .map(|nn| {
    let vkEnumerateInstanceLayerProperties = unsafe {
      core::mem::transmute::<
        NonNull<c_void>,
        vkEnumerateInstanceLayerProperties_t,
      >(nn)
    };
    let mut property_count: u32 = 0;
    unsafe {
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

  let _ = get_proc_address(
    vk_module_handle,
    c_str!("vkEnumerateInstanceExtensionProperties"),
  )
  .map(|nn| {
    let vkEnumerateInstanceExtensionProperties = unsafe {
      core::mem::transmute::<
        NonNull<c_void>,
        vkEnumerateInstanceExtensionProperties_t,
      >(nn)
    };
    match do_vkEnumerateInstanceExtensionProperties(
      vkEnumerateInstanceExtensionProperties,
      None,
    ) {
      Ok(v) => {
        println!("vkEnumerateInstanceExtensionProperties(root): {:#?}", v)
      }
      Err(e) => println!(
        "vkEnumerateInstanceExtensionProperties(root): failed. {e:?}",
        e = e
      ),
    }
    for layer in available_layers.iter() {
      let name = str_from_null_terminated_byte_array(&layer.layerName)
        .unwrap_or("unknown");
      match do_vkEnumerateInstanceExtensionProperties(
        vkEnumerateInstanceExtensionProperties,
        Some(&layer.layerName),
      ) {
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
  unsafe { f(layer_name_ptr, &mut ext_count, null_mut()) };
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
