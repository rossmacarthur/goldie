use super::*;

use serde::Serialize;

#[test]
fn goldie_golden_file() {
    let manifest_dir = "/repo";
    let tests = [
        (
            ("src/lib.rs", "crate::tests::func"),
            "/repo/src/testdata/func.golden",
        ),
        (
            ("src/lib.rs", "crate::tests::a::func"),
            "/repo/src/testdata/a/func.golden",
        ),
        (
            ("src/tests.rs", "crate::tests::func"),
            "/repo/src/testdata/func.golden",
        ),
        (
            ("src/tests.rs", "crate::tests::a::func"),
            "/repo/src/testdata/a/func.golden",
        ),
        (
            ("src/a.rs", "crate::a::tests::func"),
            "/repo/src/a/testdata/func.golden",
        ),
        (
            ("src/a.rs", "crate::a::tests::b::func"),
            "/repo/src/a/testdata/b/func.golden",
        ),
        (
            ("src/a/tests.rs", "crate::a::tests::func"),
            "/repo/src/a/testdata/func.golden",
        ),
        (
            ("src/a/tests.rs", "crate::a::tests::b::func"),
            "/repo/src/a/testdata/b/func.golden",
        ),
        (
            ("src/a/mod.rs", "crate::a::tests::func"),
            "/repo/src/a/testdata/func.golden",
        ),
        (
            ("src/a/mod.rs", "crate::a::tests::b::func"),
            "/repo/src/a/testdata/b/func.golden",
        ),
        (
            ("src/a/tests.rs", "crate::a::tests::func"),
            "/repo/src/a/testdata/func.golden",
        ),
        (
            ("src/a/tests.rs", "crate::a::tests::b::func"),
            "/repo/src/a/testdata/b/func.golden",
        ),
        (
            ("src/a/b/tests.rs", "crate::a::b::tests::func"),
            "/repo/src/a/b/testdata/func.golden",
        ),
        (
            ("src/a/b/tests.rs", "crate::a::b::tests::c::func"),
            "/repo/src/a/b/testdata/c/func.golden",
        ),
        (
            ("src/bin/a.rs", "crate::tests::func"),
            "/repo/src/bin/a/testdata/func.golden",
        ),
        (
            ("src/bin/a.rs", "crate::tests::b::func"),
            "/repo/src/bin/a/testdata/b/func.golden",
        ),
        (
            ("tests/a.rs", "crate::func"),
            "/repo/tests/a/testdata/func.golden",
        ),
        (
            ("tests/a.rs", "crate::b::func"),
            "/repo/tests/a/testdata/b/func.golden",
        ),
        (
            ("tests/a/b.rs", "crate::b::func"),
            "/repo/tests/a/b/testdata/func.golden",
        ),
        (
            ("tests/a/b.rs", "crate::b::c::func"),
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
fn goldie_assert() {
    crate::assert!("testing...\n");
}

#[test]
fn goldie_assert_debug() {
    #[allow(dead_code)]
    #[derive(Debug)]
    struct User {
        name: &'static str,
        surname: &'static str,
    }

    let u = User {
        name: "Steve",
        surname: "Harrington",
    };

    crate::assert_debug!(&u);
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

#[test]
fn goldie_assert_json() {
    #[derive(Serialize)]
    struct User {
        name: &'static str,
        surname: &'static str,
    }

    let u = User {
        name: "Steve",
        surname: "Harrington",
    };

    crate::assert_json!(&u);
}
