
[GitHub Repo](https://github.com/rust-tutorials/triangle-from-scratch)

# Triangle From Scratch: Introduction

This is an educational series about drawing a triangle without using any outside crates.

Specifically, the rules are:

1) We can only put a crate into the `[dependencies]` section of `Cargo.toml` if it's a crate that we wrote ourselves, as part of this project.
2) We **can** still use Rust's standard library. Since all Rust programs can import from the standard library without a `[dependencies]` entry, it's fair game.

Without any external crates, we'll have to write our own operating system bindings.
It's not difficult code to write, there's just a lot of background details you need to understand first.
That's where most of our focus will go, on learning how that works.
There's a lot less focus spent on the literal "triangle drawing" part, which is usually fairly easy.

Expected subjects include:

* Reading OS documentation (which usually assumes you're programming in C).
* Understanding the C header files that describe the OS's public API.
* Writing appropriate "raw" Rust bindings to that public API.
* Creating ergonomic wrapper functions to make the API easily used with the rest of Rust.
* Having those wrapper functions be fully safe (in the Rust sense) when possible, or at least making them as error-proof as we can.

**Reminder:** The "absolutely no dependencies" thing is for demonstration purposes only.
If you actually want to draw a triangle within a reasonable amount of development time, please do feel free to use dependencies.
Depending on what you need to do, there's generally many good crates available.
