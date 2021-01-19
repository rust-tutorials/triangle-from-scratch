
# Web Nonsense

People really like to run stuff in the browser.
It's very nice to end users if they can just open a web page and not have to install a whole thing.

If you want to run Rust code in a browser you compile it to [WebAssembly](https://en.wikipedia.org/wiki/WebAssembly),
or WASM for short,
which is an output target the same as compiling for windows x86_64,
or linux arm,
or any other target.

Even then, Wasm is strongly sandboxed, and it cannot directly interact with the world.
Not only do you have to bind to some external functions on the Rust side,
you have to *write those external functions yourself* in javascript.
This is a bit of a bother,
and so,
*for this one target platform*,
we'll first see how to do it ourselves,
and then we'll see how to leverage the most common crate for targeting wasm.
