
# Triangle From Scratch

This is a series on how to draw a triangle without using any outside crates.

> Specifically, we can only put a crate into the `[dependencies]`
> section of `Cargo.toml` if it's a crate that we wrote ourselves,
> as part of this project.

This means two things:

1) We **can** still use Rust's Standard Library.
2) We'll have to write all our own operating system bindings.

Which means that the project will end up being a lot *less* about the specific
final step of drawing a triangle with GL (or whatever graphics API), and a lot
*more* about how to do general things like:

* Reading OS documentation (which usually assumes you're programming in C).
* Understanding the C header files that describe the OS's public API.
* Writing appropriate "raw" Rust bindings to that public API.
* Creating ergonomic wrapper functions to make the API easily used with the rest of Rust.
* Having those wrapper functions be fully safe (in the Rust sense) when possible,
  or at least making them as error-proof as we can.

Drawing a triangle just gives us a long term goal that's easy to understand.
