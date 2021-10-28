#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(unused)]
use core::ptr::null_mut;

use triangle_from_scratch::{c_str, gl::*, win32::*};

struct WindowData {
  hdc: HDC,
  hglrc: HGLRC,
  opengl32: HMODULE,
  gl_clear: glClear_t,
  gl_clear_color: glClearColor_t,
}
impl Default for WindowData {
  fn default() -> Self {
    unsafe { core::mem::zeroed() }
  }
}
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

fn main() {
  let instance = get_process_handle();

  #[allow(non_snake_case)]
  let (
    wgl_extensions,
    wglChoosePixelFormatARB,
    wglCreateContextAttribsARB,
    wglSwapIntervalEXT,
  ) = get_wgl_basics().unwrap();
  println!("> WGL Extensions: {:?}", wgl_extensions);

  // real window stuff
  let sample_window_class = "Sample Window Class";
  let sample_window_class_wn = wide_null(sample_window_class);

  let mut wc = WNDCLASSW::default();
  wc.style = CS_OWNDC | CS_HREDRAW | CS_VREDRAW;
  wc.lpfnWndProc = Some(window_procedure);
  wc.hInstance = instance;
  wc.lpszClassName = sample_window_class_wn.as_ptr();
  wc.hCursor = load_predefined_cursor(IDCursor::Arrow).unwrap();

  let _atom = unsafe { register_class(&wc) }.unwrap();

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
  println!("> Selected Pixel Format: {:?}", pfd);
  unsafe { set_pixel_format(hdc, pix_format, &pfd) }.unwrap();

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

  let opengl32 = load_library("opengl32.dll").unwrap();
  unsafe { (*lparam).opengl32 = opengl32 };
  unsafe { (*lparam).load_gl_functions() };

  // Enable "adaptive" vsync if possible, otherwise normal vsync
  if wgl_extensions.iter().any(|s| s == "WGL_EXT_swap_control_tear") {
    unsafe { (wglSwapIntervalEXT.unwrap())(-1) };
  } else {
    unsafe { (wglSwapIntervalEXT.unwrap())(1) };
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

pub unsafe extern "system" fn window_procedure(
  hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
  match msg {
    WM_NCCREATE => {
      println!("WM_NCCREATE");
      let createstruct: *mut CREATESTRUCTW = lparam as *mut _;
      if createstruct.is_null() {
        eprintln!("createstruct pointer was null");
        return 0;
      }
      let ptr = (*createstruct).lpCreateParams as *mut WindowData;
      if let Err(e) = set_window_userdata::<WindowData>(hwnd, ptr) {
        println!("Couldn't set the WindowData pointer: {}", e);
        return 0;
      }
      // This is required for the window title to be drawn!
      return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    WM_CREATE => println!("WM_CREATE"),
    WM_CLOSE => {
      println!("WM_CLOSE");
      let _success = DestroyWindow(hwnd);
    }
    WM_DESTROY => {
      println!("WM_DESTROY");
      match get_window_userdata::<WindowData>(hwnd) {
        Ok(ptr) if !ptr.is_null() => {
          let window_data = Box::from_raw(ptr);
          FreeLibrary(window_data.opengl32);
          wgl_delete_context(window_data.hglrc)
            .unwrap_or_else(|e| eprintln!("GL Context deletion error: {}", e));
          if !release_dc(hwnd, window_data.hdc) {
            eprintln!("There was an HDC release error.");
          }
          eprintln!("WM_DESTROY> Cleanup complete.");
        }
        Ok(_) => {
          eprintln!("WM_DESTROY> userdata ptr is null, no cleanup.")
        }
        Err(e) => {
          eprintln!("WM_DESTROY> Error getting userdata ptr: {}", e)
        }
      }
      post_quit_message(0);
    }
    WM_PAINT => match get_window_userdata::<WindowData>(hwnd) {
      Ok(ptr) if !ptr.is_null() => {
        let window_data = ptr.as_mut().unwrap();
        (window_data.gl_clear_color.unwrap())(0.6, 0.7, 0.8, 1.0);
        (window_data.gl_clear.unwrap())(GL_COLOR_BUFFER_BIT);
        SwapBuffers(window_data.hdc);
      }
      Ok(_) => {
        eprintln!("WM_PAINT> userdata ptr is null")
      }
      Err(e) => {
        eprintln!("WM_PAINT> Error while getting the userdata ptr: {}", e)
      }
    },
    _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
  }
  0
}
