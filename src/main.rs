#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ptr::{null, null_mut};

macro_rules! unsafe_impl_default_zeroed {
  ($t:ty) => {
    impl Default for $t {
      #[inline]
      #[must_use]
      fn default() -> Self {
        unsafe { core::mem::zeroed() }
      }
    }
  };
}

fn main() {
  let hInstance = unsafe { GetModuleHandleW(null()) };
  let sample_window_class_wn = wide_null("Sample Window Class");

  let mut wc = WNDCLASSW::default();
  wc.lpfnWndProc = Some(window_procedure);
  wc.hInstance = hInstance;
  wc.lpszClassName = sample_window_class_wn.as_ptr();
  wc.hCursor = unsafe { LoadCursorW(hInstance, IDC_ARROW) };

  let atom = unsafe { RegisterClassW(&wc) };
  if atom == 0 {
    let last_error = unsafe { GetLastError() };
    panic!("Could not register the window class, error code: {}", last_error);
  }

  let sample_window_name_wn = wide_null("Sample Window Name");
  let lparam: *mut i32 = Box::leak(Box::new(5_i32));
  let hwnd = unsafe {
    CreateWindowExW(
      0,
      sample_window_class_wn.as_ptr(),
      sample_window_name_wn.as_ptr(),
      WS_OVERLAPPEDWINDOW,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      null_mut(),
      null_mut(),
      hInstance,
      lparam.cast(),
    )
  };
  if hwnd.is_null() {
    panic!("Failed to create a window.");
  }
  let _previously_visible = unsafe { ShowWindow(hwnd, SW_SHOW) };

  let mut msg = MSG::default();
  loop {
    let message_return = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
    if message_return == 0 {
      break;
    } else if message_return == -1 {
      let last_error = unsafe { GetLastError() };
      panic!("Error with `GetMessageW`: {}", last_error);
    } else {
      unsafe {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      }
    }
  }
}

pub unsafe extern "system" fn window_procedure(
  hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> LRESULT {
  match Msg {
    WM_NCCREATE => {
      println!("NC Create");
      let createstruct: *mut CREATESTRUCTW = lParam as *mut _;
      if createstruct.is_null() {
        return 0;
      }
      let boxed_i32_ptr = (*createstruct).lpCreateParams;
      SetWindowLongPtrW(hWnd, GWLP_USERDATA, boxed_i32_ptr as LONG_PTR);
      return 1;
    }
    WM_CREATE => println!("Create"),
    WM_CLOSE => {
      let text_null = wide_null("Really quit?");
      let caption_null = wide_null("My Caption");
      let mb_output = MessageBoxW(
        hWnd,
        text_null.as_ptr(),
        caption_null.as_ptr(),
        MB_OKCANCEL,
      );
      if mb_output == IDOK {
        let _success = DestroyWindow(hWnd);
      }
    }
    WM_DESTROY => {
      let ptr = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut i32;
      Box::from_raw(ptr);
      println!("Cleaned up the box.");
      PostQuitMessage(0);
    }
    WM_PAINT => {
      let ptr = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut i32;
      println!("Current ptr: {}", *ptr);
      *ptr += 1;
      let mut ps = PAINTSTRUCT::default();
      let hdc = BeginPaint(hWnd, &mut ps);
      let _success = FillRect(hdc, &ps.rcPaint, (COLOR_WINDOW + 1) as HBRUSH);
      EndPaint(hWnd, &ps);
    }
    _ => return DefWindowProcW(hWnd, Msg, wParam, lParam),
  }
  0
}

type ATOM = WORD;
type BOOL = c_int;
type c_int = i32;
type c_long = i32;
type c_uint = u32;
type c_ulong = u32;
type c_ushort = u16;
type DWORD = c_ulong;
type HANDLE = PVOID;
type HBRUSH = HANDLE;
type HCURSOR = HICON;
type HICON = HANDLE;
type HINSTANCE = HANDLE;
type HMENU = HANDLE;
type HMODULE = HINSTANCE;
type HWND = HANDLE;
type LONG_PTR = isize;
type LPARAM = LONG_PTR;
type LPCWSTR = *const WCHAR;
type LPVOID = *mut core::ffi::c_void;
type LRESULT = LONG_PTR;
type PVOID = *mut core::ffi::c_void;
type UINT = c_uint;
type UINT_PTR = usize;
type WCHAR = wchar_t;
type wchar_t = u16;
type WORD = c_ushort;
type WPARAM = UINT_PTR;
type LONG = c_long;
type LPMSG = *mut MSG;
type LPWSTR = *mut WCHAR;
type ULONG_PTR = usize;
type HDC = HANDLE;
type BYTE = u8;
type LPPAINTSTRUCT = *mut PAINTSTRUCT;

type WNDPROC = Option<
  unsafe extern "system" fn(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
  ) -> LRESULT,
>;

#[repr(C)]
pub struct WNDCLASSW {
  style: UINT,
  lpfnWndProc: WNDPROC,
  cbClsExtra: c_int,
  cbWndExtra: c_int,
  hInstance: HINSTANCE,
  hIcon: HICON,
  hCursor: HCURSOR,
  hbrBackground: HBRUSH,
  lpszMenuName: LPCWSTR,
  lpszClassName: LPCWSTR,
}
unsafe_impl_default_zeroed!(WNDCLASSW);

#[repr(C)]
pub struct MSG {
  hwnd: HWND,
  message: UINT,
  wParam: WPARAM,
  lParam: LPARAM,
  time: DWORD,
  pt: POINT,
  lPrivate: DWORD,
}
unsafe_impl_default_zeroed!(MSG);

#[repr(C)]
pub struct POINT {
  x: LONG,
  y: LONG,
}
unsafe_impl_default_zeroed!(POINT);

#[repr(C)]
pub struct PAINTSTRUCT {
  hdc: HDC,
  fErase: BOOL,
  rcPaint: RECT,
  fRestore: BOOL,
  fIncUpdate: BOOL,
  rgbReserved: [BYTE; 32],
}
unsafe_impl_default_zeroed!(PAINTSTRUCT);

#[repr(C)]
pub struct RECT {
  left: LONG,
  top: LONG,
  right: LONG,
  bottom: LONG,
}
unsafe_impl_default_zeroed!(RECT);

#[repr(C)]
pub struct CREATESTRUCTW {
  lpCreateParams: LPVOID,
  hInstance: HINSTANCE,
  hMenu: HMENU,
  hwndParent: HWND,
  cy: c_int,
  cx: c_int,
  y: c_int,
  x: c_int,
  style: LONG,
  lpszName: LPCWSTR,
  lpszClass: LPCWSTR,
  dwExStyle: DWORD,
}
unsafe_impl_default_zeroed!(CREATESTRUCTW);

const WS_OVERLAPPED: u32 = 0x00000000;
const WS_CAPTION: u32 = 0x00C00000;
const WS_SYSMENU: u32 = 0x00080000;
const WS_THICKFRAME: u32 = 0x00040000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_MAXIMIZEBOX: u32 = 0x00010000;
const WS_OVERLAPPEDWINDOW: u32 = WS_OVERLAPPED
  | WS_CAPTION
  | WS_SYSMENU
  | WS_THICKFRAME
  | WS_MINIMIZEBOX
  | WS_MAXIMIZEBOX;
const CW_USEDEFAULT: c_int = 0x80000000_u32 as c_int;
const SW_SHOW: c_int = 5;
const WM_CLOSE: u32 = 0x0010;
const WM_DESTROY: u32 = 0x0002;
const WM_PAINT: u32 = 0x000F;
const WM_NCCREATE: u32 = 0x0081;
const WM_CREATE: u32 = 0x0001;
const IDC_ARROW: LPCWSTR = MAKEINTRESOURCEW(32512);
const COLOR_WINDOW: u32 = 5;
const MB_OKCANCEL: u32 = 1;
const IDOK: c_int = 1;
const GWLP_USERDATA: c_int = -21;

#[link(name = "Kernel32")]
extern "system" {
  /// [`GetModuleHandleW`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew)
  pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;

  /// [`GetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror)
  pub fn GetLastError() -> DWORD;
}

#[link(name = "User32")]
extern "system" {
  /// [`RegisterClassW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassw)
  pub fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> ATOM;

  /// [`CreateWindowExW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw)
  pub fn CreateWindowExW(
    dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR,
    dwStyle: DWORD, X: c_int, Y: c_int, nWidth: c_int, nHeight: c_int,
    hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID,
  ) -> HWND;

  /// [`DefWindowProcW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-defwindowprocw)
  pub fn DefWindowProcW(
    hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM,
  ) -> LRESULT;

  /// [`ShowWindow`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow)
  pub fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> BOOL;

  /// [`GetMessageW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew)
  pub fn GetMessageW(
    lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT,
  ) -> BOOL;

  /// [`TranslateMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translatemessage)
  pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;

  /// [`DispatchMessageW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-dispatchmessagew)
  pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;

  /// [`DestroyWindow`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow)
  pub fn DestroyWindow(hWnd: HWND) -> BOOL;

  /// [`PostQuitMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage)
  pub fn PostQuitMessage(nExitCode: c_int);

  /// [`LoadCursorW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadcursorw)
  pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> HCURSOR;

  /// [`BeginPaint`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-beginpaint)
  pub fn BeginPaint(hWnd: HWND, lpPaint: LPPAINTSTRUCT) -> HDC;

  /// [`FillRect`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-fillrect)
  pub fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> c_int;

  /// [`EndPaint`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-endpaint)
  pub fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;

  /// [`MessageBoxW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messageboxw)
  pub fn MessageBoxW(
    hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT,
  ) -> c_int;

  /// [`SetWindowLongPtrW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw)
  pub fn SetWindowLongPtrW(
    hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR,
  ) -> LONG_PTR;

  /// [`GetWindowLongPtrW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowlongptrw)
  pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
}

/// [`MAKEINTRESOURCEW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-makeintresourcew)
const fn MAKEINTRESOURCEW(i: WORD) -> LPWSTR {
  i as ULONG_PTR as LPWSTR
}

/// Turns a Rust string slice into a null-terminated utf-16 vector.
pub fn wide_null(s: &str) -> Vec<u16> {
  s.encode_utf16().chain(Some(0)).collect()
}
