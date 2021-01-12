#![allow(non_camel_case_types)]

use super::*;

/// From `gl.xml`
pub type GLbitfield = c_uint;

/// From `gl.xml`
pub type GLfloat = c_float;

/// See [`glClear`](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glClear.xhtml)
pub type glClear_t = Option<unsafe extern "system" fn(mask: GLbitfield)>;

/// See [`glClearColor`](https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/glClearColor.xhtml)
pub type glClearColor_t = Option<
  unsafe extern "system" fn(
    red: GLfloat,
    green: GLfloat,
    blue: GLfloat,
    alpha: GLfloat,
  ),
>;

pub const GL_COLOR_BUFFER_BIT: GLbitfield = 0x00004000;
