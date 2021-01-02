
# Triangle From Scratch

## This Is A Tutorial Repository

Feel free to look at the source code,
but the crate is not on crates.io,
and the majority of the project's "value" is in the mdbook articles that explain what's going on.

[You Can Read The Book On The GitHub Pages Site](https://rust-tutorials.github.io/triangle-from-scratch/)

## Project Organization

The `book_src/` contains the files to build the mdbook (`cargo install mdbook && mdbook build --open`).

The `examples/` folder is an "archive" of completed, runnable examples based on the mdbook chapters.

The crate's library portion (`src/lib.rs`, and sub-modules) contains the most easily reused code,
based on work shown in the the mdbook chapters.

The crate's binary portion (`src/main.rs`) is just scratch space.
This is so that you can most easily use `cargo run` to test things out when doing work.
The actual content of `main.rs` at any given moment is unimportant.
All the valuable code is kept as an `example/` file when a tutorial chapter is completed.

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
