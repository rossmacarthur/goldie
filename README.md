# goldie

[![Crates.io Version](https://img.shields.io/crates/v/goldie.svg)](https://crates.io/crates/goldie)
[![Docs.rs Latest](https://img.shields.io/badge/docs.rs-latest-blue.svg)](https://docs.rs/goldie)
[![Build Status](https://img.shields.io/github/workflow/status/rossmacarthur/goldie/build/trunk)](https://github.com/rossmacarthur/goldie/actions?query=workflow%3Abuild)

Simple golden file testing for Rust.

```rust
goldie::assert!(text);
```

## 🚀 Getting started

Add `goldie` to your project as a dev dependency.

```sh
cargo add goldie --dev
```

In your test function assert the contents using `goldie::assert!`. The golden
filename will be automatically determined based on the test file and test
function name. Run tests with `GOLDIE_UPDATE=true` to automatically update
golden files.

```rust
#[test]
fn example() {
    let text = { /* ... run the test ... */ }

    // assert that the contents of ./testdata/example.golden
    // are equal to `text`
    goldie::assert!(text)
}
```

Templated golden files are also supported using `goldie::assert_template!`.
Something implementing `serde::Serialize` needs to be provided as context in
order to render the template. Values are rendered using
[upon](https://github.com/rossmacarthur/upon) e.g. `{{ value.field }}`.
You cannot use  `GOLDIE_UPDATE=true` to automatically update templated golden
files.

```rust
#[test]
fn example() {
    let text = { /* ... run the test ... */ }

    // assert that the contents of ./testdata/example.golden
    // are equal to `text` after rendering with `ctx`.
    let ctx = upon::value!{ value: "Hello World!" };
    goldie::assert_template!(&ctx, text)
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
