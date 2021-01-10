#![cfg(windows)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Module for stuff that's specific to the Win32 API on Windows.

pub use core::ffi::c_void;

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

pub type ATOM = WORD;
pub type BOOL = c_int;
pub type BYTE = u8;
pub type c_int = i32;
pub type c_long = i32;
pub type c_uint = u32;
pub type c_ulong = u32;
pub type c_ushort = u16;
pub type c_char = i8;
pub type DWORD = c_ulong;
pub type HANDLE = PVOID;
pub type HBRUSH = HANDLE;
pub type HCURSOR = HICON;
pub type HDC = HANDLE;
pub type HICON = HANDLE;
pub type HINSTANCE = HANDLE;
pub type HMENU = HANDLE;
pub type HMODULE = HINSTANCE;
pub type HWND = HANDLE;
pub type LONG = c_long;
pub type LONG_PTR = isize;
pub type LPARAM = LONG_PTR;
pub type LPCWSTR = *const WCHAR;
pub type LPMSG = *mut MSG;
pub type LPPAINTSTRUCT = *mut PAINTSTRUCT;
pub type LPVOID = *mut c_void;
pub type LPCVOID = *const c_void;
pub type va_list = *mut c_char;
pub type LPWSTR = *mut WCHAR;
pub type LRESULT = LONG_PTR;
pub type PVOID = *mut c_void;
pub type UINT = c_uint;
pub type UINT_PTR = usize;
pub type ULONG_PTR = usize;
pub type WCHAR = wchar_t;
pub type wchar_t = u16;
pub type WORD = c_ushort;
pub type WPARAM = UINT_PTR;
pub type HLOCAL = HANDLE;

pub type WNDPROC = Option<
  unsafe extern "system" fn(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
  ) -> LRESULT,
>;

/// See [`WNDCLASSW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassw)
#[repr(C)]
pub struct WNDCLASSW {
  pub style: UINT,
  pub lpfnWndProc: WNDPROC,
  pub cbClsExtra: c_int,
  pub cbWndExtra: c_int,
  pub hInstance: HINSTANCE,
  pub hIcon: HICON,
  pub hCursor: HCURSOR,
  pub hbrBackground: HBRUSH,
  pub lpszMenuName: LPCWSTR,
  pub lpszClassName: LPCWSTR,
}
unsafe_impl_default_zeroed!(WNDCLASSW);

#[repr(C)]
pub struct MSG {
  pub hwnd: HWND,
  pub message: UINT,
  pub wParam: WPARAM,
  pub lParam: LPARAM,
  pub time: DWORD,
  pub pt: POINT,
  pub lPrivate: DWORD,
}
unsafe_impl_default_zeroed!(MSG);

#[repr(C)]
pub struct POINT {
  pub x: LONG,
  pub y: LONG,
}
unsafe_impl_default_zeroed!(POINT);

#[repr(C)]
pub struct PAINTSTRUCT {
  pub hdc: HDC,
  pub fErase: BOOL,
  pub rcPaint: RECT,
  pub fRestore: BOOL,
  pub fIncUpdate: BOOL,
  pub rgbReserved: [BYTE; 32],
}
unsafe_impl_default_zeroed!(PAINTSTRUCT);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RECT {
  pub left: LONG,
  pub top: LONG,
  pub right: LONG,
  pub bottom: LONG,
}
unsafe_impl_default_zeroed!(RECT);

#[repr(C)]
pub struct CREATESTRUCTW {
  pub lpCreateParams: LPVOID,
  pub hInstance: HINSTANCE,
  pub hMenu: HMENU,
  pub hwndParent: HWND,
  pub cy: c_int,
  pub cx: c_int,
  pub y: c_int,
  pub x: c_int,
  pub style: LONG,
  pub lpszName: LPCWSTR,
  pub lpszClass: LPCWSTR,
  pub dwExStyle: DWORD,
}
unsafe_impl_default_zeroed!(CREATESTRUCTW);

#[repr(C)]
pub struct PIXELFORMATDESCRIPTOR {
  pub nSize: WORD,
  pub nVersion: WORD,
  pub dwFlags: DWORD,
  pub iPixelType: BYTE,
  pub cColorBits: BYTE,
  pub cRedBits: BYTE,
  pub cRedShift: BYTE,
  pub cGreenBits: BYTE,
  pub cGreenShift: BYTE,
  pub cBlueBits: BYTE,
  pub cBlueShift: BYTE,
  pub cAlphaBits: BYTE,
  pub cAlphaShift: BYTE,
  pub cAccumBits: BYTE,
  pub cAccumRedBits: BYTE,
  pub cAccumGreenBits: BYTE,
  pub cAccumBlueBits: BYTE,
  pub cAccumAlphaBits: BYTE,
  pub cDepthBits: BYTE,
  pub cStencilBits: BYTE,
  pub cAuxBuffers: BYTE,
  pub iLayerType: BYTE,
  pub bReserved: BYTE,
  pub dwLayerMask: DWORD,
  pub dwVisibleMask: DWORD,
  pub dwDamageMask: DWORD,
}
impl Default for PIXELFORMATDESCRIPTOR {
  /// Automatically fills out the correct `nSize` and `nVersion` values.
  ///
  /// Other fields are all zeroed.
  #[inline]
  #[must_use]
  fn default() -> Self {
    let mut out: Self = unsafe { core::mem::zeroed() };
    out.nSize = core::mem::size_of::<Self>() as WORD;
    out.nVersion = 1;
    out
  }
}

/// Allocates a unique device context for each window in the class.
pub const CS_OWNDC: u32 = 0x0020;

/// Redraws the entire window if a movement or size adjustment changes the width
/// of the client area.
pub const CS_HREDRAW: u32 = 0x0002;

/// Redraws the entire window if a movement or size adjustment changes the
/// height of the client area.
pub const CS_VREDRAW: u32 = 0x0001;

pub const WS_OVERLAPPED: u32 = 0x00000000;
pub const WS_CAPTION: u32 = 0x00C00000;
pub const WS_SYSMENU: u32 = 0x00080000;
pub const WS_THICKFRAME: u32 = 0x00040000;
pub const WS_MINIMIZEBOX: u32 = 0x00020000;
pub const WS_MAXIMIZEBOX: u32 = 0x00010000;
pub const WS_OVERLAPPEDWINDOW: u32 = WS_OVERLAPPED
  | WS_CAPTION
  | WS_SYSMENU
  | WS_THICKFRAME
  | WS_MINIMIZEBOX
  | WS_MAXIMIZEBOX;
pub const CW_USEDEFAULT: c_int = 0x80000000_u32 as c_int;
pub const SW_SHOW: c_int = 5;

/// Sent as a signal that a window or an application should terminate.
///
/// * `wparam` / `lparam`: Not used.
/// * Application Should Return: 0
pub const WM_CLOSE: u32 = 0x0010;

/// Sent when a window is being destroyed.
///
/// * `wparam` / `lparam`: Not used.
/// * Application Should Return: 0
/// * See [`WM_DESTROY`](https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-destroy)
pub const WM_DESTROY: u32 = 0x0002;

/// Sent when the system or another application makes a request to paint a
/// portion of an application's window.
///
/// * `wparam` / `lparam`: Not used.
/// * Application Should Return: 0
/// * See [`WM_PAINT`](https://docs.microsoft.com/en-us/windows/win32/gdi/wm-paint)
pub const WM_PAINT: u32 = 0x000F;

/// "Non-client Create". Sent prior to the [`WM_CREATE`] message when a window
/// is first created.
///
/// * `wparam`: Not used.
/// * `lparam`: Pointer to a `CREATESTRUCT`
/// * Application Should Return: 1 to continue, 0 to cancel.
/// * See [`WM_NCCREATE`](https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-nccreate)
pub const WM_NCCREATE: u32 = 0x0081;

/// Sent when an application requests that a window be created by calling the
/// `CreateWindowEx` function.
///
/// * `wparam`: Not used.
/// * `lparam`: Pointer to a `CREATESTRUCT`
/// * Application Should Return: 0 to continue, -1 to cancel.
/// * See [`WM_CREATE`](https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-create)
pub const WM_CREATE: u32 = 0x0001;

/// Indicates a request to terminate an application, and is generated when the
/// application calls the [`PostQuitMessage`] function.
///
/// * `wparam` (on `MSG` struct): The exit code that was given to
///   `PostQuitMessage`.
/// * `lparam`: Not used.
/// * See [`WM_QUIT`](https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-quit)
pub const WM_QUIT: u32 = 0x0012;
pub const IDC_ARROW: LPCWSTR = MAKEINTRESOURCEW(32512);
pub const COLOR_WINDOW: u32 = 5;
pub const MB_OKCANCEL: u32 = 1;
pub const IDOK: c_int = 1;
pub const GWLP_USERDATA: c_int = -21;

pub const WS_EX_APPWINDOW: DWORD = 0x00040000;
pub const WS_EX_WINDOWEDGE: DWORD = 0x00000100;
pub const WS_EX_CLIENTEDGE: DWORD = 0x00000200;
pub const WS_EX_OVERLAPPEDWINDOW: DWORD = WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE;

/// [`PIXELFORMATDESCRIPTOR`] pixel type
pub const PFD_TYPE_RGBA: u8 = 0;
/// [`PIXELFORMATDESCRIPTOR`] pixel type
pub const PFD_TYPE_COLORINDEX: u8 = 1;

/// [`PIXELFORMATDESCRIPTOR`] layer type
pub const PFD_MAIN_PLANE: u8 = 0;
/// [`PIXELFORMATDESCRIPTOR`] layer type
pub const PFD_OVERLAY_PLANE: u8 = 1;
/// [`PIXELFORMATDESCRIPTOR`] layer type
pub const PFD_UNDERLAY_PLANE: u8 = u8::MAX /* was (-1) */;

pub const PFD_DOUBLEBUFFER: u32 = 0x00000001;
pub const PFD_STEREO: u32 = 0x00000002;
pub const PFD_DRAW_TO_WINDOW: u32 = 0x00000004;
pub const PFD_DRAW_TO_BITMAP: u32 = 0x00000008;
pub const PFD_SUPPORT_GDI: u32 = 0x00000010;
pub const PFD_SUPPORT_OPENGL: u32 = 0x00000020;
pub const PFD_GENERIC_FORMAT: u32 = 0x00000040;
pub const PFD_NEED_PALETTE: u32 = 0x00000080;
pub const PFD_NEED_SYSTEM_PALETTE: u32 = 0x00000100;
pub const PFD_SWAP_EXCHANGE: u32 = 0x00000200;
pub const PFD_SWAP_COPY: u32 = 0x00000400;
pub const PFD_SWAP_LAYER_BUFFERS: u32 = 0x00000800;
pub const PFD_GENERIC_ACCELERATED: u32 = 0x00001000;
pub const PFD_SUPPORT_DIRECTDRAW: u32 = 0x00002000;
pub const PFD_DIRECT3D_ACCELERATED: u32 = 0x00004000;
pub const PFD_SUPPORT_COMPOSITION: u32 = 0x00008000;

/// use with [`ChoosePixelFormat`] only
pub const PFD_DEPTH_DONTCARE: u32 = 0x20000000;
/// use with [`ChoosePixelFormat`] only
pub const PFD_DOUBLEBUFFER_DONTCARE: u32 = 0x40000000;
/// use with [`ChoosePixelFormat`] only
pub const PFD_STEREO_DONTCARE: u32 = 0x80000000;

#[link(name = "Kernel32")]
extern "system" {
  /// [`GetModuleHandleW`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew)
  pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;

  /// [`GetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror)
  pub fn GetLastError() -> DWORD;

  /// [`SetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-setlasterror)
  pub fn SetLastError(dwErrCode: DWORD);

  /// [`FormatMessageW`](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessagew)
  pub fn FormatMessageW(
    dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD,
    lpBuffer: LPWSTR, nSize: DWORD, Arguments: va_list,
  ) -> DWORD;

  /// [`LocalFree`](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-localfree)
  pub fn LocalFree(hMem: HLOCAL) -> HLOCAL;
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

  /// [`SetCursor`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setcursor)
  pub fn SetCursor(hCursor: HCURSOR) -> HCURSOR;
}

#[link(name = "Gdi32")]
extern "system" {
  /// [`ChoosePixelFormat`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-choosepixelformat)
  pub fn ChoosePixelFormat(
    hdc: HDC, ppfd: *const PIXELFORMATDESCRIPTOR,
  ) -> c_int;
}

/// [`MAKEINTRESOURCEW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-makeintresourcew)
const fn MAKEINTRESOURCEW(i: WORD) -> LPWSTR {
  i as ULONG_PTR as LPWSTR
}

/// Turns a Rust string slice into a null-terminated utf-16 vector.
pub fn wide_null(s: &str) -> Vec<u16> {
  s.encode_utf16().chain(Some(0)).collect()
}

/// Returns a handle to the file used to create the calling process (.exe file)
///
/// See [`GetModuleHandleW`](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew)
pub fn get_process_handle() -> HMODULE {
  // Safety: as per the MSDN docs.
  unsafe { GetModuleHandleW(null()) }
}

/// The predefined cursor styles.
pub enum IDCursor {
  /// Standard arrow and small hourglass
  AppStarting = 32650,
  /// Standard arrow
  Arrow = 32512,
  /// Crosshair
  Cross = 32515,
  /// Hand
  Hand = 32649,
  /// Arrow and question mark
  Help = 32651,
  /// I-beam
  IBeam = 32513,
  /// Slashed circle
  No = 32648,
  /// Four-pointed arrow pointing north, south, east, and west
  SizeAll = 32646,
  /// Double-pointed arrow pointing northeast and southwest
  SizeNeSw = 32643,
  /// Double-pointed arrow pointing north and south
  SizeNS = 32645,
  /// Double-pointed arrow pointing northwest and southeast
  SizeNwSe = 32642,
  /// Double-pointed arrow pointing west and east
  SizeWE = 32644,
  /// Vertical arrow
  UpArrow = 32516,
  /// Hourglass
  Wait = 32514,
}

/// Load one of the predefined cursors.
///
/// See [`LoadCursorW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadcursorw)
pub fn load_predefined_cursor(cursor: IDCursor) -> Result<HCURSOR, Win32Error> {
  // Safety: The enum only allows values from the approved list. See MSDN.
  let hcursor =
    unsafe { LoadCursorW(null_mut(), MAKEINTRESOURCEW(cursor as WORD)) };
  if hcursor.is_null() {
    Err(get_last_error())
  } else {
    Ok(hcursor)
  }
}

/// Registers a window class struct.
///
/// ## Safety
///
/// All pointer fields of the struct must be correct.
///
/// See [`RegisterClassW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassw)
pub unsafe fn register_class(
  window_class: &WNDCLASSW,
) -> Result<ATOM, Win32Error> {
  let atom = RegisterClassW(window_class);
  if atom == 0 {
    Err(get_last_error())
  } else {
    Ok(atom)
  }
}

/// Gets the thread-local last-error code value.
///
/// See [`GetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror)
pub fn get_last_error() -> Win32Error {
  Win32Error(unsafe { GetLastError() })
}

/// Sets the thread-local last-error code value.
///
/// See [`SetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-setlasterror)
pub fn set_last_error(e: Win32Error) {
  unsafe { SetLastError(e.0) }
}

/// Newtype wrapper for a Win32 error code.
///
/// If bit 29 is set, it's an application error.
#[derive(Debug)]
#[repr(transparent)]
pub struct Win32Error(pub DWORD);
impl std::error::Error for Win32Error {}

impl core::fmt::Display for Win32Error {
  /// Displays the error using `FormatMessageW`
  ///
  /// ```
  /// use triangle_from_scratch::win32::*;
  /// let s = format!("{}", Win32Error(0));
  /// assert_eq!("The operation completed successfully.  ", s);
  /// let app_error = format!("{}", Win32Error(1 << 29));
  /// assert_eq!("Win32ApplicationError(536870912)", app_error);
  /// ```
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    pub const FORMAT_MESSAGE_ALLOCATE_BUFFER: DWORD = 0x00000100;
    pub const FORMAT_MESSAGE_FROM_SYSTEM: DWORD = 0x00001000;
    pub const FORMAT_MESSAGE_IGNORE_INSERTS: DWORD = 0x00000200;

    if self.0 & (1 << 29) > 0 {
      return write!(f, "Win32ApplicationError({})", self.0);
    }
    let dwFlags = FORMAT_MESSAGE_ALLOCATE_BUFFER
      | FORMAT_MESSAGE_FROM_SYSTEM
      | FORMAT_MESSAGE_IGNORE_INSERTS;
    let lpSource = null_mut();
    let dwMessageId = self.0;
    let dwLanguageId = 0;
    // this will point to our allocation after the call
    let mut buffer: *mut u16 = null_mut();
    let lpBuffer = &mut buffer as *mut *mut u16 as *mut u16;
    let nSize = 0;
    let Arguments = null_mut();
    let tchar_count_excluding_null = unsafe {
      FormatMessageW(
        dwFlags,
        lpSource,
        dwMessageId,
        dwLanguageId,
        lpBuffer,
        nSize,
        Arguments,
      )
    };
    if tchar_count_excluding_null == 0 || buffer.is_null() {
      // some sort of problem happened. we can't usefully get_last_error since
      // Display formatting doesn't let you give an error value.
      return Err(core::fmt::Error);
    } else {
      struct OnDropLocalFree(HLOCAL);
      impl Drop for OnDropLocalFree {
        fn drop(&mut self) {
          unsafe { LocalFree(self.0) };
        }
      }
      let _on_drop = OnDropLocalFree(buffer as HLOCAL);
      let buffer_slice: &[u16] = unsafe {
        core::slice::from_raw_parts(buffer, tchar_count_excluding_null as usize)
      };
      for decode_result in
        core::char::decode_utf16(buffer_slice.iter().copied())
      {
        match decode_result {
          Ok('\r') | Ok('\n') => write!(f, " ")?,
          Ok(ch) => write!(f, "{}", ch)?,
          Err(_) => write!(f, "�")?,
        }
      }
      Ok(())
    }
  }
}

/// Creates a window.
///
/// * The window is not initially shown, you must call [`ShowWindow`] yourself.
///
/// See [`CreateWindowExW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw)
pub unsafe fn create_app_window(
  class_name: &str, window_name: &str, position: Option<[i32; 2]>,
  [width, height]: [i32; 2], create_param: LPVOID,
) -> Result<HWND, Win32Error> {
  let class_name_null = wide_null(class_name);
  let window_name_null = wide_null(window_name);
  let (x, y) = match position {
    Some([x, y]) => (x, y),
    None => (CW_USEDEFAULT, CW_USEDEFAULT),
  };
  let hwnd = CreateWindowExW(
    WS_EX_APPWINDOW | WS_EX_OVERLAPPEDWINDOW,
    class_name_null.as_ptr(),
    window_name_null.as_ptr(),
    WS_OVERLAPPEDWINDOW,
    x,
    y,
    width,
    height,
    null_mut(),
    null_mut(),
    get_process_handle(),
    create_param,
  );
  if hwnd.is_null() {
    Err(get_last_error())
  } else {
    Ok(hwnd)
  }
}

/// Gets a message from the thread's message queue.
///
/// The message can be for any window from this thread,
/// or it can be a non-window message as well.
///
/// See [`GetMessageW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew)
pub fn get_any_message() -> Result<MSG, Win32Error> {
  let mut msg = MSG::default();
  let output = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
  if output == -1 {
    Err(get_last_error())
  } else {
    Ok(msg)
  }
}

/// Translates virtual-key messages into character messages.
///
/// The character messages go into your thread's message queue,
/// and you'll see them if you continue to consume messages.
///
/// **Returns:**
/// * `true` if the message was `WM_KEYDOWN`, `WM_KEYUP`, `WM_SYSKEYDOWN`, or
///   `WM_SYSKEYUP`.
/// * `true` for any other message type that generated a character message.
/// * otherwise `false`
///
/// See [`TranslateMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-translatemessage)
pub fn translate_message(msg: &MSG) -> bool {
  0 != unsafe { TranslateMessage(msg) }
}

/// Sets the "userdata" pointer of the window (`GWLP_USERDATA`).
///
/// **Returns:** The previous userdata pointer.
///
/// [`SetWindowLongPtrW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw)
pub unsafe fn set_window_userdata<T>(
  hwnd: HWND, ptr: *mut T,
) -> Result<*mut T, Win32Error> {
  set_last_error(Win32Error(0));
  let out = SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as LONG_PTR);
  if out == 0 {
    // if output is 0, it's only a "real" error if the last_error is non-zero
    let last_error = get_last_error();
    if last_error.0 != 0 {
      Err(last_error)
    } else {
      Ok(out as *mut T)
    }
  } else {
    Ok(out as *mut T)
  }
}

/// Gets the "userdata" pointer of the window (`GWLP_USERDATA`).
///
/// **Returns:** The userdata pointer.
///
/// [`GetWindowLongPtrW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowlongptrw)
pub unsafe fn get_window_userdata<T>(hwnd: HWND) -> Result<*mut T, Win32Error> {
  set_last_error(Win32Error(0));
  let out = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
  if out == 0 {
    // if output is 0, it's only a "real" error if the last_error is non-zero
    let last_error = get_last_error();
    if last_error.0 != 0 {
      Err(last_error)
    } else {
      Ok(out as *mut T)
    }
  } else {
    Ok(out as *mut T)
  }
}

/// Indicates to the system that a thread has made a request to terminate
/// (quit).
///
/// The exit code becomes the `wparam` of the [`WM_QUIT`] message your message
/// loop eventually gets.
///
/// [`PostQuitMessage`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postquitmessage)
pub fn post_quit_message(exit_code: c_int) {
  unsafe { PostQuitMessage(exit_code) }
}

/// Prepares the specified window for painting.
///
/// On success: you get back both the [`HDC`] and [`PAINTSTRUCT`]
/// that you'll need for future painting calls (including [`EndPaint`]).
///
/// [`BeginPaint`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-beginpaint)
pub unsafe fn begin_paint(
  hwnd: HWND,
) -> Result<(HDC, PAINTSTRUCT), Win32Error> {
  let mut ps = PAINTSTRUCT::default();
  let hdc = BeginPaint(hwnd, &mut ps);
  if hdc.is_null() {
    Err(get_last_error())
  } else {
    Ok((hdc, ps))
  }
}

/// See [`GetSysColor`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getsyscolor)
pub enum SysColor {
  _3dDarkShadow = 21,
  _3dLight = 22,
  ActiveBorder = 10,
  ActiveCaption = 2,
  AppWorkspace = 12,
  /// Button face, also "3D face" color.
  ButtonFace = 15,
  /// Button highlight, also "3D highlight" color.
  ButtonHighlight = 20,
  /// Button shadow, also "3D shadow" color.
  ButtonShadow = 16,
  ButtonText = 18,
  CaptionText = 9,
  /// Desktop background color
  Desktop = 1,
  GradientActiveCaption = 27,
  GradientInactiveCaption = 28,
  GrayText = 17,
  Highlight = 13,
  HighlightText = 14,
  HotLight = 26,
  InactiveBorder = 11,
  InactiveCaption = 3,
  InactiveCaptionText = 19,
  InfoBackground = 24,
  InfoText = 23,
  Menu = 4,
  MenuHighlight = 29,
  MenuBar = 30,
  MenuText = 7,
  ScrollBar = 0,
  Window = 5,
  WindowFrame = 6,
  WindowText = 8,
}

/// Fills a rectangle with the given system color.
///
/// When filling the specified rectangle, this does **not** include the
/// rectangle's right and bottom sides. GDI fills a rectangle up to, but not
/// including, the right column and bottom row, regardless of the current
/// mapping mode.
///
/// [`FillRect`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-fillrect)
pub unsafe fn fill_rect_with_sys_color(
  hdc: HDC, rect: &RECT, color: SysColor,
) -> Result<(), ()> {
  if FillRect(hdc, rect, (color as u32 + 1) as HBRUSH) != 0 {
    Ok(())
  } else {
    Err(())
  }
}

/// See [`EndPaint`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-endpaint)
pub unsafe fn end_paint(hwnd: HWND, ps: &PAINTSTRUCT) {
  EndPaint(hwnd, ps);
}

/// Performs [`begin_paint`] / [`end_paint`] around your closure.
pub unsafe fn do_some_painting<F, T>(hwnd: HWND, f: F) -> Result<T, Win32Error>
where
  F: FnOnce(HDC, bool, RECT) -> Result<T, Win32Error>,
{
  let (hdc, ps) = begin_paint(hwnd)?;
  let output = f(hdc, ps.fErase != 0, ps.rcPaint);
  end_paint(hwnd, &ps);
  output
}

/// See [`ChoosePixelFormat`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-choosepixelformat)
pub unsafe fn choose_pixel_format(
  hdc: HDC, ppfd: &PIXELFORMATDESCRIPTOR,
) -> Result<c_int, Win32Error> {
  let index = ChoosePixelFormat(hdc, ppfd);
  if index != 0 {
    Ok(index)
  } else {
    Err(get_last_error())
  }
}
