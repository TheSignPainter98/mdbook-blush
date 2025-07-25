# mdbook-blush

A preprocessor for [mdBook][mdbook] which adds the ‘blush’ notation for small-caps.

When applied, words surrounded by `=` signs will be formatted as small-caps.
Small-cap annotations work within words, but not between words, i.e. `=q=o=s=` looks like `QoS`.

## Installation

Type and run

```bash
cargo install mdbook-blush
```

This will build `mdbook-blush` from source.

## Integrating `mdbook-blush`

Type and run the following command with the path to your book.

```bash
mdbook-blush install path/to/book
```

You will see that a new `[preprocessor.blush]` table has been added to your `book.toml`, the `[output.html]` table has been amended to include the newly-written `blush.css` file.

The blush notation is now available for use.

## License and Author.

This project is [licensed under GPLv3](LICENSE).

This project was originally written by Ed Jones.

[mdbook]: https://github.com/rust-lang/mdBook
