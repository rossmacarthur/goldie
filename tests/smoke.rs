use std::io;
use std::path::Path;

use anyhow::Context as _;
use serde::Serialize;

use goldie::Builder;

#[test]
fn goldie_golden_file() {
    let manifest_dir = "/repo";
    let tests = [
        (
            ("src/lib.rs", "goldie::tests::func"),
            "/repo/src/testdata/func.golden",
        ),
        (
            ("src/lib.rs", "goldie::tests::a::func"),
            "/repo/src/testdata/a/func.golden",
        ),
        (
            ("src/tests.rs", "goldie::tests::func"),
            "/repo/src/testdata/func.golden",
        ),
        (
            ("src/tests.rs", "goldie::tests::a::func"),
            "/repo/src/testdata/a/func.golden",
        ),
        (
            ("src/a.rs", "goldie::a::tests::func"),
            "/repo/src/a/testdata/func.golden",
        ),
        (
            ("src/a.rs", "goldie::a::tests::b::func"),
            "/repo/src/a/testdata/b/func.golden",
        ),
        (
            ("src/a/tests.rs", "goldie::a::tests::func"),
            "/repo/src/a/testdata/func.golden",
        ),
        (
            ("src/a/tests.rs", "goldie::a::tests::b::func"),
            "/repo/src/a/testdata/b/func.golden",
        ),
        (
            ("src/a/mod.rs", "goldie::a::tests::func"),
            "/repo/src/a/testdata/func.golden",
        ),
        (
            ("src/a/mod.rs", "goldie::a::tests::b::func"),
            "/repo/src/a/testdata/b/func.golden",
        ),
        (
            ("src/a/tests.rs", "goldie::a::tests::func"),
            "/repo/src/a/testdata/func.golden",
        ),
        (
            ("src/a/tests.rs", "goldie::a::tests::b::func"),
            "/repo/src/a/testdata/b/func.golden",
        ),
        (
            ("src/a/b/tests.rs", "goldie::a::b::tests::func"),
            "/repo/src/a/b/testdata/func.golden",
        ),
        (
            ("src/a/b/tests.rs", "goldie::a::b::tests::c::func"),
            "/repo/src/a/b/testdata/c/func.golden",
        ),
        (
            ("src/bin/a.rs", "goldie::tests::func"),
            "/repo/src/bin/a/testdata/func.golden",
        ),
        (
            ("src/bin/a.rs", "goldie::tests::b::func"),
            "/repo/src/bin/a/testdata/b/func.golden",
        ),
        (
            ("tests/a.rs", "goldie::func"),
            "/repo/tests/a/testdata/func.golden",
        ),
        (
            ("tests/a.rs", "goldie::b::func"),
            "/repo/tests/a/testdata/b/func.golden",
        ),
        (
            ("tests/a/b.rs", "goldie::b::func"),
            "/repo/tests/a/b/testdata/func.golden",
        ),
        (
            ("tests/a/b.rs", "goldie::b::c::func"),
            "/repo/tests/a/b/testdata/c/func.golden",
        ),
    ];

    for ((source_file, function_path), exp) in tests {
        let g = Builder::new(manifest_dir, source_file, function_path).build();
        assert_eq!(
            g.golden_file,
            Path::new(exp),
            "source_file: {source_file}, function_path: {function_path}",
        );
    }
}

#[test]
fn goldie_new_name() {
    goldie::new!()
        .name("not_new_name.golden")
        .build()
        .assert("testing...\n");
}

#[test]
fn goldie_new_name_prefix() {
    goldie::new!()
        .name_prefix("custom-")
        .build()
        .assert("testing...\n");
}

#[test]
fn goldie_new_name_prefix_dir() {
    goldie::new!()
        .name_prefix("custom/")
        .build()
        .assert("testing...\n");
}

#[test]
fn goldie_new_name_suffix() {
    goldie::new!()
        .name_suffix("-custom")
        .build()
        .assert("testing...\n");
}

#[test]
fn goldie_new_name_suffix_dir() {
    goldie::new!()
        .name_suffix("/custom")
        .build()
        .assert("testing...\n");
}

#[test]
fn goldie_new_golden_dir() {
    goldie::new!()
        .golden_dir(Path::new(file!()).parent().unwrap().join("mytestdata"))
        .build()
        .assert("testing...\n");
}

#[test]
fn goldie_new_extension() {
    goldie::new!()
        .extension("txt")
        .build()
        .assert("testing...\n");
}

#[test]
fn goldie_assert() {
    goldie::assert!("testing...\n");
}

#[test]
fn goldie_assert_alt() {
    let err: io::Result<()> = Err(io::Error::other("failed to frobnicate"));
    let err = err.context("while shaving the yak");
    goldie::assert_alt!(err.unwrap_err());
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct User {
    name: &'static str,
    surname: &'static str,
}

#[test]
fn goldie_assert_debug() {
    let u = User {
        name: "Steve",
        surname: "Harrington",
    };
    goldie::assert_debug!(&u);
}

#[test]
fn goldie_assert_debug_alt() {
    let u = User {
        name: "Steve",
        surname: "Harrington",
    };
    goldie::assert_debug_alt!(&u);
}

#[test]
fn goldie_assert_template() {
    #[derive(Serialize)]
    struct Context {
        test: &'static str,
    }
    let ctx = Context { test: "testing..." };
    goldie::assert_template!(&ctx, "Such testing...\n");
}

#[test]
fn goldie_assert_json() {
    let u = User {
        name: "Steve",
        surname: "Harrington",
    };

    goldie::assert_json!(&u);
}
