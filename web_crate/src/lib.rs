pub type GLenum = u32;
pub type GLbitmask = u32;
pub type GLuint = u32;
pub type GLint = i32;
pub type GLsizei = i32;
// Note(kettle11): GLintptr should be an i64, but Wasm can't pass those, so for
// now just use an i32.
pub type GLintptr = i32;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct JSObject(u32);
impl JSObject {
  pub const fn null() -> Self {
    JSObject(0)
  }
}

use constants::*;
mod constants {
  //! Values taken from the [WebGL Constants page](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Constants).
  //!
  //! All names here have the `GL_` prefix added.

  use super::{GLbitmask, GLenum};

  pub const GL_ARRAY_BUFFER: GLenum = 0x8892;
  pub const GL_ELEMENT_ARRAY_BUFFER: GLenum = 0x8893;
  pub const GL_FLOAT: GLenum = 0x1406;
  pub const GL_FRAGMENT_SHADER: GLenum = 0x8B30;
  pub const GL_STATIC_DRAW: GLenum = 0x88E4;
  pub const GL_TRIANGLES: GLenum = 0x0004;
  pub const GL_UNSIGNED_SHORT: GLenum = 0x1403;
  pub const GL_VERTEX_SHADER: GLenum = 0x8B31;

  pub const GL_COLOR_BUFFER_BIT: GLbitmask = 0x00004000;
}

mod js {
  //! Holds our `extern "C"` declarations for javascript interactions.

  use super::*;

  extern "C" {
    pub fn setupCanvas();

    //

    pub fn attachShader(program: JSObject, shader: JSObject);
    pub fn bindBuffer(target: GLenum, id: JSObject);
    pub fn bufferDataF32(
      target: GLenum, data_ptr: *const f32, data_length: usize, usage: GLenum,
    );
    pub fn bufferDataU16(
      target: GLenum, data_ptr: *const u16, data_length: usize, usage: GLenum,
    );
    pub fn clear(mask: GLbitmask);
    pub fn clearColor(r: f32, g: f32, b: f32, a: f32);
    pub fn compileShader(program: JSObject);
    pub fn createBuffer() -> JSObject;
    pub fn createProgram() -> JSObject;
    pub fn createShader(shader_type: GLenum) -> JSObject;
    pub fn drawElements(
      mode: GLenum, count: GLsizei, type_: GLenum, offset: GLintptr,
    );
    pub fn enableVertexAttribArray(index: GLuint);
    pub fn getAttribLocation(
      program: JSObject, name: *const u8, name_length: usize,
    ) -> GLuint;
    pub fn linkProgram(program: JSObject);
    pub fn shaderSource(
      shader: JSObject, source: *const u8, source_length: usize,
    );
    pub fn useProgram(program: JSObject);
    pub fn vertexAttribPointer(
      index: GLuint, size: GLint, type_: GLenum, normalized: bool,
      stride: GLsizei, pointer: GLintptr,
    );
  }
}

#[no_mangle]
pub extern "C" fn start() {
  unsafe {
    js::setupCanvas();

    let vertex_data = [-0.2_f32, 0.5, 0.0, -0.5, -0.4, 0.0, 0.5, -0.1, 0.0];
    let vertex_buffer = js::createBuffer();
    js::bindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
    js::bufferDataF32(
      GL_ARRAY_BUFFER,
      vertex_data.as_ptr(),
      vertex_data.len(),
      GL_STATIC_DRAW,
    );

    let index_data = [0_u16, 1, 2];
    let index_buffer = js::createBuffer();
    js::bindBuffer(GL_ELEMENT_ARRAY_BUFFER, index_buffer);
    js::bufferDataU16(
      GL_ELEMENT_ARRAY_BUFFER,
      index_data.as_ptr(),
      index_data.len(),
      GL_STATIC_DRAW,
    );

    let vertex_shader_text = "
      attribute vec3 vertex_position;
      void main(void) {
        gl_Position = vec4(vertex_position, 1.0);
      }";
    let vertex_shader = js::createShader(GL_VERTEX_SHADER);
    js::shaderSource(
      vertex_shader,
      vertex_shader_text.as_bytes().as_ptr(),
      vertex_shader_text.len(),
    );
    js::compileShader(vertex_shader);

    let fragment_shader_text = "
      void main() {
        gl_FragColor = vec4(1.0, 0.5, 0.313, 1.0);
      }";
    let fragment_shader = js::createShader(GL_FRAGMENT_SHADER);
    js::shaderSource(
      fragment_shader,
      fragment_shader_text.as_bytes().as_ptr(),
      fragment_shader_text.len(),
    );
    js::compileShader(fragment_shader);

    let shader_program = js::createProgram();
    js::attachShader(shader_program, vertex_shader);
    js::attachShader(shader_program, fragment_shader);
    js::linkProgram(shader_program);
    js::useProgram(shader_program);

    let name = "vertex_position";
    let attrib_location = js::getAttribLocation(
      shader_program,
      name.as_bytes().as_ptr(),
      name.len(),
    );
    assert!(attrib_location != GLuint::MAX);
    js::enableVertexAttribArray(attrib_location);
    js::vertexAttribPointer(attrib_location, 3, GL_FLOAT, false, 0, 0);

    js::clearColor(0.37, 0.31, 0.86, 1.0);
    js::clear(GL_COLOR_BUFFER_BIT);
    js::drawElements(GL_TRIANGLES, 3, GL_UNSIGNED_SHORT, 0);
  }
}
