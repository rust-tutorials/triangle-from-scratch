
# Web GL with bare Wasm

I should give a big thanks to [kettle11](https://github.com/kettle11),
who made the [hello_triangle_wasm_rust](https://github.com/kettle11/hello_triangle_wasm_rust) example for me.

Also, I should probably have an extra reminder at the top of this lesson:
This is the "doing it all yourself" style.
Much of the "Rust for Wasm" ecosystem uses a crate called [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/introduction.html).
In the same way that, if you "just want to open a window" you would often reach for `winit` or `sdl2` or something,
if you "just want to show something in the browser" you'll often use `wasm-bindgen` (and the crates that go with it).
People will at least *expect* that you're using `wasm-bindgen` if you get lost and need to ask someone for help.
They've got a book of their own, with many many examples, so have a look there if that's what you wanna do.

## Toolchain Setup

Before we even begin, we'll need to take a few extra steps to have the right compiler and tools available.

In addition to having Rust installed, we need to install the `wasm32-unknown-unknown` target:

> rustup target add wasm32-unknown-unknown

In addition, you may wish to obtain the `wasm-opt` tool from their [GitHub repo](https://github.com/WebAssembly/binaryen),
though it's not required.

You also might wish to obtain the `wasm-strip` tool from [The WebAssembly Binary Toolkit](https://github.com/WebAssembly/wabt) (WABT).
It lets you strip debugging symbols and such from the program, reducing the size by quite a bit.
You can also do this without an extra tool via a Nightly `rustc` flag.

Once you've compiled your program to wasm you'll also need some way to display it.

**Unfortunately, you can't simply open a local file in your browser using a `file://` address.**

This is fine for a plain HTML file,
but browsers (rightly) get more paranoid every day,
so they don't support wasm execution in pages loaded through a file address.
If you don't already have such a thing (I didn't), then you can try [devserver](https://crates.io/crates/devserver).

> cargo install devserver

If you already have your own favorite way to spin up a local server that can serve static files, that's fine too.

## Separate Folder

This will have a few non-standard bits of setup,
so I'm going to put it in a `web_crate/` directory.

First it needs its own `Cargo.toml` file:
```toml
[package]
name = "triangle-from-scratch-web-crate"
version = "0.1.0"
authors = ["Lokathor <zefria@gmail.com>"]
edition = "2018"
license = "Zlib OR Apache-2.0 OR MIT"
```

And to make a wasm library we need to tell Rust that the [crate-type](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-crate-type-field)
will be `cdylib`:
```toml
[lib]
crate-type = ["cdylib"]
```

Personally I also like to turn on [link-time optimization](https://doc.rust-lang.org/cargo/reference/profiles.html#lto) with release builds,
not because it's required,
but just because I'm willing to spend some extra compile time to get a performance edge.
The winner here is "thin",
which provides almost all the benefit for a minimal amount of additional time and memory taken to compile.

```toml
[profile.release]
lto = "thin"
```

Now we're set.

## The Wasm Library

As you can sorta already see, our "program" isn't actually going to be built as an executable.
Instead, it's going to be built as a C-compatible library that the JavaScript of the webpage will load and use.
This means that instead of writing a `main.rs` with an optional `lib.rs`,
we put 100% of the code into `lib.rs` right from the start.

```rust
// lib.rs

#[no_mangle]
pub extern "C" fn start() {
  // nothing yet!
}
```

Note the use of the [no_mangle](https://doc.rust-lang.org/reference/abi.html#the-no_mangle-attribute) attribute.
This totally disables the usual name mangling that Rust does.
It allows for the function to be called by external code that doesn't know Rust's special naming scheme, which is good,
but there can only be a single function with a given name anywhere.
In other words, if some other function named `start` with no mangling existed *anywhere* in our project,
or in any of our dependencies, then we'd get a compilation error.
That's why name mangling is on by default.

Also note that we have to declare that our `start` function uses the `extern "C"` ABI,
this will give us the correct calling convention when communicating between JavaScript and Wasm.

When JavaScript loads up our wasm module, the `start` function will be called.
This will allow our program to do whatever it wants to do,
similar to the `main` function in a normal program.

## The Web Page

Okay now we need a webpage for the user to display and have the wasm go.

I'm absolutely not a web development person,
but I know just enough to throw some HTML together by hand:

```html
<html>

<body>
  <canvas width="800" height="600" id="my_canvas"></canvas>
</body>

</html>
```

Next we start the local server and go to the page.
```
D:\dev\triangle-from-scratch>cd web_crate

D:\dev\triangle-from-scratch\web_crate>devserver

Serving [D:\dev\triangle-from-scratch\web_crate\] at [ https://localhost:8080 ] or [ http://localhost:8080 ]
Automatic reloading is enabled!
Stop with Ctrl+C
```
And it says "Hello." in the middle of the page.
We'll just leave that open in one console and it'll automatically reload files as necessary.

Now we build our wasm module (note the `--target` argument):
```
D:\dev\triangle-from-scratch\web_crate>cargo build --release --target wasm32-unknown-unknown
   Compiling triangle-from-scratch-web-crate v0.1.0 (D:\dev\triangle-from-scratch\web_crate)
    Finished release [optimized] target(s) in 1.16s
```

which makes a file: `target/wasm32-unknown-unknown/release/triangle_from_scratch_web_crate.wasm`

(If we hadn't used the `--release` flag, then it'd be in `target/wasm32-unknown-unknown/debug/` instead.)

Now we have to alter our page to load the wasm via a script:
```html
<html>

<body>
  <canvas width="800" height="600" id="my_canvas"></canvas>
  <script>
    var importObject = {};

    const mod_path = 'target/wasm32-unknown-unknown/release/triangle_from_scratch_web_crate.wasm';
    WebAssembly.instantiateStreaming(fetch(mod_path), importObject)
      .then(results => {
        results.instance.exports.start();
      });
  </script>
</body>

</html>
```

What's going on here?
Well, you should sure read the [Loading and running WebAssembly code](https://developer.mozilla.org/en-US/docs/WebAssembly/Loading_and_running)
tutorial on the Mozilla Developer Network (MDN) page.

* First we call [WebAssembly.instantiateStreaming()](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/instantiateStreaming)
  * The first argument is whatever will give us the wasm stream.
    In this case, a call to [fetch](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API).
  * The second argument is the "import object", which lets us provide things to the wasm code.
    At the moment we don't provide anything to the wasm, so we use an empty object.
* This gives a `Promise<ResultObject>`, so we use the `then` method to do something to the results.
  It's similar to Rust's async/await and Future stuff.
  Except it's not quite the same, they tell me.
  I don't really know JavaScript, but I'm kinda just nodding and smiling as we go.
* When acting on the results,
  `results.module` is the [web assembly module](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Module)
  and `results.instance` is the [web assembly instance](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Instance).
  The module isn't too helpful to us right now,
  but by using the instance we can call our `start` function (or any other non-mangled public function).

## Make The Wasm Do Something

It's not too exciting for nothing to happen.
Let's have the wasm clear the canvas to a non-white color.

First we expand the script on the web page.
What we need to do is give the wasm code some functions to let it interact with the outside world.
```html
  <script>
    var gl;
    var canvas;

    function setupCanvas() {
      console.log("Setting up the canvas...");
      let canvas = document.getElementById("my_canvas");
      gl = canvas.getContext("webgl");
      if (!gl) {
        console.log("Failed to get a WebGL context for the canvas!");
        return;
      }
    }

    function clearToBlue() {
      gl.clearColor(0.1, 0.1, 0.9, 1.0);
      gl.clear(gl.COLOR_BUFFER_BIT);
    }

    var importObject = {
      env: {
        setupCanvas: setupCanvas,
        clearToBlue: clearToBlue,
      }
    };

    const mod_path = 'target/wasm32-unknown-unknown/release/triangle_from_scratch_web_crate.wasm';
    WebAssembly.instantiateStreaming(fetch(mod_path), importObject)
      .then(results => {
        results.instance.exports.start();
      });
  </script>
```

Now our `importObject` has an `env` field.
Each function declared in here will be accessible to the wasm as an external function.
One of them sets up the canvas and WebGL context.
The other clears the canvas to a nice blue color.

Now we can call these from the Rust code:
```rust
mod js {
  extern "C" {
    pub fn setupCanvas();
    pub fn clearToBlue();
  }
}

#[no_mangle]
pub extern "C" fn start() {
  unsafe {
    js::setupCanvas();
    js::clearToBlue();
  }
}
```

And we'll see a blue canvas!

Note that JavaScript convention doesn't use `snake_case` naming,
they use `camelCase` naming.
The naming style isn't significant to the compiler, it's just a convention.

## Workflow Tweaks

When we want to rebuild our wasm module we have to use the whole
`cargo build --release --target wasm32-unknown-unknown`
each time.
Horrible.
Let's make a [.cargo/config.toml](https://doc.rust-lang.org/cargo/reference/config.html)
file in our `web_stuff` crate folder.
Then we can set the default build target to be for wasm:
```toml
[build]
target = "wasm32-unknown-unknown"
```
Now `cargo build` and `cargo build --release` will pick the `wasm32-unknown-unknown` target by default.

Also, here is where we can easily pass the flag for `rustc` to strip the symbols from the output:
```toml
[build]
target = "wasm32-unknown-unknown"
rustflags = ["-Zstrip=symbols"]
```
The `-Z` part means that it's an unstable flag, so we can only do it with Nightly.
If you want to strip the symbols but stick to Stable Rust you'll have to get the
`wasm-strip` tool from the [wabt](https://github.com/WebAssembly/wabt)
toolkit that I mentioned before.
Stripping the symbols just makes the output smaller, so there's less to send over the network.
In a small example like ours, it changes the final output size from 308 bytes to 161 bytes.
Our code isn't doing too much, in terms of instructions,
so just putting in the debug symbols is a hefty percentage of the overall bytes taken.
We'll have another look when our program is doing a bit more to see if it's still a big difference.

Also, it's a little annoying to have to manually rebuild our wasm when the HTML pages reloads automatically.
To fix this, we can get `cargo-watch`

> cargo install cargo-watch

And then run a cargo-watch instance to automatically rebuild the code as necessary:
```
cargo watch -c -x "build --release"
```
The `-c` clears the terminal each time the watch restarts so that you never look at old output by accident.

The `-x "build --release"` executes "cargo build --release" each time `cargo-watch` detects a change.

Now we will always have both the latest HTML *and* wasm in our browser page.

## Drawing A Triangle

We need a little more wasm/js interaction than what we have right now to actually draw a triangle.

If you want a larger WebGL tutorial you should check out [the one on MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial).
WebGL is based on OpenGL ES 2.0,
which is based on OpenGL 2.0,
so if you already know about GL stuff, this will probably look very familiar.

For now, we'll mostly skip over the WebGL explanations themselves.
Instead we'll focus on the interoperation guts that let our wasm interact with WebGL.

### The Rust Code

What we want is for our rust code to look something like this:
```rust
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
```

I don't want to cover too many details of how WebGL works right now because we're mostly focusing on the Wasm stuff,
but here are the broad steps:
* Initialize the canvas
* Bind a buffer as the ARRAY_BUFFER and then place our vertex data into it.
* Bind a buffer as the ELEMENT_ARRAY_BUFFER and then give it our index data.
* Create a vertex shader
* Create a fragment shader
* Create a program, connect the two shaders, then link, then use.
* Get the location of the vertex_position attribute,
  enable that location,
  and then point the location at the correct position within our vertex array.
* Clear the screen to our background color.
* Draw the triangle.

If you're used to OpenGL,
or even to graphics programming using some other API,
this should all feel quite familiar.

To support our `start` function we need to have quite a few more `extern` declarations,
and also `const` declarations:
```rust
pub type GLenum = u32;
pub type GLbitmask = u32;
pub type GLuint = u32;
pub type GLint = i32;
pub type GLsizei = i32;
// Note(kettle11): GLintptr should be an i64, but those can't be properly passed
// between Wasm and Javascript, so for now just use an i32.
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
```

This is pretty normal stuff, except the `JsObject` thing.
What's going on there?

Well, we can't pass a whole javascript object over the C FFI.
What even is a javascript object, anyway?
I dunno, some sort of hash... thing... with fields.
It doesn't matter.
The point is that it's a type that you *can't* pass over the C FFI.
That's mostly fine, except that we need to communicate with GL about them.

What we'll do is store all our javascript objects in a list out in javascript-land,
and then in the WASM we just use the *index values* into that list to name the javascript objects when we need to.

### The JavaScript Code

On the javascript side of things, we mostly add a bunch of boring functions,
but a few are interesting.

First we set up a few more variables we'll use.
We have the `gl` and `canvas` from before,
but now we'll need to make the javascript and wasm memory interact,
and we'll also need to track javascript objects that the wasm knows about.
Since we need to transfer strings between wasm and javascript,
we'll need a [TextDecoder](https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder).
```html
<html>

<body>
  <canvas width="800" height="600" id="my_canvas"></canvas>
  <script>
    var gl;
    var canvas;
    var wasm_memory;
    var js_objects = [null];

    const decoder = new TextDecoder();
```

The canvas setup is basically the same as before,
```js
    function setupCanvas() {
      console.log("Setting up the canvas.");
      let canvas = document.getElementById("my_canvas");
      gl = canvas.getContext("webgl");
      if (!gl) {
        console.log("Failed to get a WebGL context for the canvas!");
        return;
      }
    }
```

But making our `importObject` is quite a few functions this time:
```js
    var importObject = {
      env: {
        setupCanvas: setupCanvas,

        attachShader: function (program, shader) {
          gl.attachShader(js_objects[program], js_objects[shader]);
        },
        bindBuffer: function (target, id) {
          gl.bindBuffer(target, js_objects[id]);
        },
        bufferDataF32: function (target, data_ptr, data_length, usage) {
          const data = new Float32Array(wasm_memory.buffer, data_ptr, data_length);
          gl.bufferData(target, data, usage);
        },
        bufferDataU16: function (target, data_ptr, data_length, usage) {
          const data = new Uint16Array(wasm_memory.buffer, data_ptr, data_length);
          gl.bufferData(target, data, usage);
        },
        clear: function (mask) {
          gl.clear(mask)
        },
        clearColor: function (r, g, b, a) {
          gl.clearColor(r, g, b, a);
        },
        compileShader: function (shader) {
          gl.compileShader(js_objects[shader]);
        },
        createBuffer: function () {
          return js_objects.push(gl.createBuffer()) - 1;
        },
        createProgram: function () {
          return js_objects.push(gl.createProgram()) - 1;
        },
        createShader: function (shader_type) {
          return js_objects.push(gl.createShader(shader_type)) - 1;
        },
        drawElements: function (mode, count, type, offset) {
          gl.drawElements(mode, count, type, offset);
        },
        enableVertexAttribArray: function (index) {
          gl.enableVertexAttribArray(index)
        },
        getAttribLocation: function (program, pointer, length) {
          const string_data = new Uint8Array(wasm_memory.buffer, pointer, length);
          const string = decoder.decode(string_data);
          return gl.getAttribLocation(js_objects[program], string);
        },
        linkProgram: function (program) {
          gl.linkProgram(js_objects[program]);
        },
        shaderSource: function (shader, pointer, length) {
          const string_data = new Uint8Array(wasm_memory.buffer, pointer, length);
          const string = decoder.decode(string_data);
          gl.shaderSource(js_objects[shader], string);
        },
        useProgram: function (program) {
          gl.useProgram(js_objects[program]);
        },
        vertexAttribPointer: function (index, size, type, normalized, stride, offset) {
          gl.vertexAttribPointer(index, size, type, normalized, stride, offset);
        },
      }
    };
```

#### Slices

Of note is this function:
```js
bufferDataF32: function (target, data_ptr, data_length, usage) {
  const data = new Float32Array(wasm_memory.buffer, data_ptr, data_length);
  gl.bufferData(target, data, usage);
},
```
To access a memory slice living in the wasm memory,
first the wasm code passes a pointer and length out to javascript.
Then the javascript uses that to make an array object.
In this case, a [Float32Array](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Float32Array).
We have a similar function for `u16` as well.

Strings take an extra step:
once you've made a `Uint8Array` we have to use our decoder to convert that into javascript's natural string type.

#### Objects

When we need to get an object from WebGL and pass it along to wasm,
we just push it into our list and tell wasm the index of the object.
The [push](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/push)
method on arrays returns the new length, so we just subtract 1 and that'll be the index of the newly added object.
```js
createBuffer: function () {
  return js_objects.push(gl.createBuffer()) - 1;
},
```

This is *not* the most extensible system.
We can't ever delete and objects with this basic setup.
If we remove an element from our list, all the slots after would have the wrong index.

To allow for deletion, we'd need to change deleted elements to `null`,
and then when a new object is requested we'd scan the list looking for the first `null` (other than at index 0),
and put the object at that position.
If we don't find any open spots in the list, *only then* do we push it onto the end of the list.

Alternately, we could store all of our objects in a [Map](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map).
This would let us simply assign an object to a key,
and when we're done with it we'd delete the key.

I'm not a javascript expert, in fact I'm barely a javascript beginner,
so I don't know which of those would be better in the long term.

Any time we need to let the wasm "use" an object,
it passes the index of the object and we look it up from our list:
```js
attachShader: function (program, shader) {
  gl.attachShader(js_objects[program], js_objects[shader]);
},
```

#### Wasm Startup

Finally, we need to change one more thing about the startup code.

After we get the `results` back,
we have to assign the exported memory to our `wasm_memory` value.
This lets all the other functions manipulate is when they need to.
```js
const mod_path = 'target/wasm32-unknown-unknown/release/triangle_from_scratch_web_crate.wasm';
WebAssembly.instantiateStreaming(fetch(mod_path), importObject)
  .then(results => {
    console.log("Wasm instance created.");
    // assign the memory to be usable by the other functions
    wasm_memory = results.instance.exports.memory;
    // start the wasm
    results.instance.exports.start();
  });
```

## And Now There's A Triangle!

We've finally got our triangle on the page!

A static image isn't the best,
so in the next lesson we'll cover how to get some user interaction.
