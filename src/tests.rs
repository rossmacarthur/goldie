#![cfg(test)]

use super::*;

use serde::Serialize;

#[test]
fn goldie_golden_path() {
    let g = Goldie::new(
        Path::new("/full/path/to/source.rs"),
        "crate::mod::tests::function_name",
    );
    assert_eq!(
        g.golden_path,
        Path::new("/full/path/to/testdata/source/function_name.golden"),
    );
}

#[test]
fn goldie_assert() {
    crate::assert!("testing...\n");
}

#[test]
fn goldie_assert_template() {
    #[derive(Serialize)]
    struct Context {
        test: &'static str,
    }
    let ctx = Context { test: "testing..." };
    crate::assert_template!(&ctx, "Such testing...\n");
}
