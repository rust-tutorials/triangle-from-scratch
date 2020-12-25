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
  wc.lpfnWndProc = Some(DefWindowProcW);
  wc.hInstance = hInstance;
  wc.lpszClassName = sample_window_class_wn.as_ptr();

  let atom = unsafe { RegisterClassW(&wc) };
  if atom == 0 {
    let last_error = unsafe { GetLastError() };
    panic!("Could not register the window class, error code: {}", last_error);
  }

  let sample_window_name_wn = wide_null("Sample Window Name");
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
      null_mut(),
    )
  };
  if hwnd.is_null() {
    panic!("Failed to create a window.");
  }
  let _previously_visible = unsafe { ShowWindow(hwnd, SW_SHOW) };
}

type ATOM = WORD;
type BOOL = c_int;
type c_int = i32;
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
}

/// Turns a Rust string slice into a null-terminated utf-16 vector.
pub fn wide_null(s: &str) -> Vec<u16> {
  s.encode_utf16().chain(Some(0)).collect()
}
