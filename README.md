
# Triangle From Scratch

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

## Licensing and Contribution

### License

This project's content is licensed as `Zlib OR Apache-2.0 OR MIT`.

Additionally, I have been told by some people that source code licenses aren't "appropriate" for non-code works, such as tutorials.
If you would like, you can use the non-code portions of the project under the [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) license.
(I personally think that once a tutorial is *about* source code it all becomes very blurry indeed,
but I'm not a lawyer, and I'd rather people have more options than less options.)

Any contributions to the project must be submitted under the same license terms.

### What's A Good Contribution?

That's an excellent question, and I'm so happy you asked.

Even though the core of the series focuses on Win32, I'd love to eventually cover all three desktop operating systems (Win/Mac/Linux).
Similarly, I'd love to support both of the main mobile operating systems (Android/iOS),
and even embedded devices would be within scope.

The only real limitation is that I don't know much about all of these various platforms.
If **you** know about one of the platforms that we don't yet have articles about, and you want to write something, please get in touch.
The easiest ways would be to open up a tracking issue, or start a draft PR if you've already got something started.
