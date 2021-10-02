# goldie

[![Crates.io Version](https://img.shields.io/crates/v/goldie.svg)](https://crates.io/crates/goldie)
[![Docs.rs Latest](https://img.shields.io/badge/docs.rs-latest-blue.svg)](https://docs.rs/goldie)

Simple golden file testing for Rust.

```rust
goldie::assert!(text);
```

## 🚀 Getting started

Add the following to your Cargo manifest.

```toml
[dev-dependencies]
goldie = "0.1"
```

In your test function assert the contents using `goldie::assert!`. The golden
filename will be automatically determined based on the test file and test
function name. Run tests with `GOLDIE_UPDATE=true` to automatically update
golden files.
```rust
// file: module.rs

#[test]
fn example() {
    let text = { /* ... run the test ... */ }

    // assert that the contents of ./testdata/module/example.golden
    // are equal to `text`
    goldie::assert!(text)
}
```

Templated golden files are also supported using `goldie::assert_template!`.
Something implementing `serde::Serialize` needs to be provided as context in
order to render the template. Values are rendered using
[TinyTemplate](https://github.com/bheisler/TinyTemplate) e.g. `{value.field}`.
You cannot use  `GOLDIE_UPDATE=true` to automatically update templated golden
files.
```rust
// file: module.rs

use serde_json::json;

#[test]
fn example() {
    let text = { /* ... run the test ... */ }

    // first render the golden file ./testdata/module/example.golden
    // with the provided `ctx` then assert that the contents are
    // equal to the result.
    let ctx = json!({"value": "Hello World!"});
    goldie::assert_template!(&ctx, text)
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
