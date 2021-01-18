
# Web GL with bare Wasm

Before we begin I should give a big thanks to [kettle11](https://github.com/kettle11),
who made the [hello_triangle_wasm_rust](https://github.com/kettle11/hello_triangle_wasm_rust) example for me.

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
Unfortunately, you can't simply open a local file in your browser using a `file://` address.
This is fine for a plain HTML file,
but browsers (rightly) get more paranoid every day,
so they don't support wasm execution in pages loaded through a file address.
If you don't already have such a thing (I didn't), then you can try [devserver](https://crates.io/crates/devserver).

> cargo install devserver

If you already have your own favorite way to spin up a local server that can serve static files, that's fine too.

## Separate Folder

This will have a new non-standard requirements,
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

Now also, to make a wasm library like we need we have to tell Rust that the [crate-type](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-crate-type-field)
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
<canvas width="800" height="600" id="my_canvas"></canvas>

<body> Hello. </body>

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
<canvas width="800" height="600" id="my_canvas"></canvas>

<body> Hello.
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

Also, when we want to rebuild our wasm module we have to use the whole
`cargo build --release --target wasm32-unknown-unknown`
each time.
Horrible.
Let's make a `.cargo/config.toml` file in our `web_stuff` crate folder.
Then we can set the default build target to be for wasm:
```toml
[build]
target = "wasm32-unknown-unknown"
```
Now `cargo build` and `cargo build --release` will pick the `wasm32-unknown-unknown` target by default.

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

We need a little more wasm/js interaction than that to make a triangle.

TODO
