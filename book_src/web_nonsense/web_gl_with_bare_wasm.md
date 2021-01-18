
# Web GL with bare Wasm

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
If you have your own favorite way to spin up a local server that can serve static files that's fine.
If you don't already have such a thing (I didn't), then you can try [devserver](https://crates.io/crates/devserver).

> cargo install devserver

