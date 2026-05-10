//! Simple golden file testing for Rust.
//!
//! ```text
//! goldie::assert!(text);
//! ```
//!
//! # 🚀 Getting started
//!
//! Add `goldie` to your project as a dev dependency.
//!
//! ```sh
//! cargo add goldie --dev
//! ```
//!
//! In your test function assert the contents using `goldie::assert!`. The
//! golden filename will be automatically determined based on the test file and
//! test function name. Run tests with `GOLDIE_UPDATE=1` to automatically update
//! golden files.
//!
//! ```rust,no_run
//! #[test]
//! fn example() {
//!     let text = { /* ... run the test ... */ };
//!
//!     // assert that the contents of ./testdata/example.golden
//!     // are equal to `text`
//!     goldie::assert!(text)
//! }
//! ```
//!
//! # Usage
//!
//! ### Golden file location
//!
//! By default golden files are stored in a `testdata` directory next to the
//! source test module. For example if your test is in `src/a/tests.rs` then
//! the golden files will be stored in `src/a/testdata/`. This is configurable
//! by using the [`Builder`]. For example:
//!
//! ```rust,no_run
//! # let text = "";
//! goldie::new!()
//!     .name("custom_name")
//!     .build()
//!     .assert(text);
//! ```
//!
//! ### [`assert!`]
//!
//! Compares the provided value with the contents of a golden file. The value
//! must implement `Display`. If they do not match the test will fail. If the
//! environment variable `GOLDIE_UPDATE=1` is set then the golden file will be
//! updated.
//!
//! ### [`assert_alt!`]
//!
//! Compares the provided value with the contents of a golden file. The value
//! must implement `Display`. The alternate formatting (`{:#}`) is used. If they
//! do not match the test will fail. If the environment variable
//! `GOLDIE_UPDATE=1` is set then the golden file will be updated.
//!
//! ### [`assert_debug!`]
//!
//! Compares the provided value with the contents of a golden file. The value
//! must implement `Debug`. If they do not match the test will fail. If the
//! environment variable `GOLDIE_UPDATE=1` is set then the golden file will be
//! updated.
//!
//! ### [`assert_debug_alt!`]
//!
//! Compares the provided value with the contents of a golden file. The value
//! must implement `Debug`. The alternate formatting (`{:#?}`) is used. If they
//! do not match the test will fail. If the environment variable
//! `GOLDIE_UPDATE=1` is set then the golden file will be updated.
//!
//! ### [`assert_json!`]
//!
//! Golden files containing JSON data are supported using
//! `goldie::assert_json!`. Something implementing `serde::Serialize` needs to
//! be provided as the actual value. The golden file will be pretty-printed
//! JSON. You can use `GOLDIE_UPDATE=1` to automatically update JSON golden
//! files.
//!
//! ```rust,no_run
//! #[test]
//! fn example() {
//!     #[derive(Serialize)]
//!     struct User {
//!         name: &'static str,
//!         surname: &'static str,
//!     }
//!
//!     let u = User { name: "Steve", surname: "Harrington" };
//!
//!     // assert that the contents of ./testdata/example.golden
//!    // are equal to the pretty-printed JSON representation of `u`
//!     goldie::assert_json!(&u);
//! }
//! ```
//!
//! ## [`assert_template!`]
//!
//! Templated golden files are also supported using `goldie::assert_template!`.
//! Something implementing `serde::Serialize` needs to be provided as context in
//! order to render the template. Values are rendered using
//! [upon](https://github.com/rossmacarthur/upon) e.g. `{{ value.field }}`. You
//! cannot use  `GOLDIE_UPDATE=1` to automatically update templated golden
//! files.
//!
//! ```rust,no_run
//! #[test]
//! fn example() {
//!     let text = { /* ... run the test ... */ };
//!
//!     // assert that the contents of ./testdata/example.golden
//!     // are equal to `text` after rendering with `ctx`.
//!     let ctx = upon::value!{ value: "Hello World!" };
//!     goldie::assert_template!(&ctx, text)
//! }
//! ```

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::sync::LazyLock as Lazy;
use std::sync::Mutex;

#[cfg(feature = "color")]
use pretty_assertions::assert_eq;
use serde::Deserialize;
#[cfg(any(feature = "template", feature = "json"))]
use serde::Serialize;

/// Assert the golden file matches the display output `"{}"`
#[macro_export]
macro_rules! assert {
    ($actual:expr $(,)?) => {
        $crate::new!().build().assert($actual);
    };
}

/// Assert the golden file matches the alternate display output `"{:#}"`.
#[macro_export]
macro_rules! assert_alt {
    ($actual:expr $(,)?) => {
        $crate::new!().build().assert_alt($actual);
    };
}

/// Assert the golden file matches the debug output `"{:?}"`
#[macro_export]
macro_rules! assert_debug {
    ($actual:expr $(,)?) => {
        $crate::new!().build().assert_debug($actual);
    };
}

/// Assert the golden file matches the alternate debug output `"{:#?}"`.
#[macro_export]
macro_rules! assert_debug_alt {
    ($actual:expr $(,)?) => {
        $crate::new!().build().assert_debug_alt($actual);
    };
}

/// Assert the templated golden file matches.
#[macro_export]
macro_rules! assert_template {
    ($ctx:expr, $actual:expr $(,)?) => {
        $crate::new!().build().assert_template($ctx, $actual);
    };
}

/// Assert the JSON golden file matches.
#[macro_export]
macro_rules! assert_json {
    ($actual:expr $(,)?) => {
        $crate::new!().build().assert_json($actual);
    };
}

/// Constructs a new goldie instance.
#[doc(hidden)]
#[macro_export]
macro_rules! new {
    () => {{
        let source_manifest_dir = ::std::env!("CARGO_MANIFEST_DIR");
        let source_file = ::std::file!();
        let function_path = $crate::function_path!();
        $crate::Builder::new(source_manifest_dir, source_file, function_path)
    }};
}

/// Returns the fully qualified path to the current item.
///
/// Goldie uses this to get the name of the test function.
///
/// Not public API.
#[doc(hidden)]
#[macro_export]
macro_rules! function_path {
    () => {{
        fn f() {}
        fn type_name_of_val<T>(_: T) -> &'static str {
            ::std::any::type_name::<T>()
        }
        let mut name = type_name_of_val(f).strip_suffix("::f").unwrap_or("");
        while let Some(rest) = name.strip_suffix("::{{closure}}") {
            name = rest;
        }
        name
    }};
}

/// Builder for configuring a goldie assertion.
#[derive(Debug, Clone)]
pub struct Builder {
    /// env!("CARGO_MANIFEST_DIR") in the test
    source_manifest_dir: &'static str,

    /// file!() in the test, e.g. src/module/tests/file.rs
    source_file: &'static str,

    /// _function_path!() in the test, e.g. crate::module::tests::function_name
    function_path: PathBuf,

    /// Override the directory we should put the golden file in
    golden_dir: Option<PathBuf>,

    /// Override the name of the assertion, this is usually derived from the test function name
    name: Option<String>,

    /// Add a prefix to the name
    name_prefix: Option<String>,

    /// Add a suffix to the name
    name_suffix: Option<String>,

    /// Override the extension of the golden file
    extension: Option<String>,

    /// Whether to update the golden file if it doesn't match
    update: Option<bool>,
}

/// Configuration for a golden file assertion
#[derive(Debug)]
pub struct Goldie {
    /// The path to the golden file.
    pub golden_file: PathBuf,
    /// Whether to update the golden file if it doesn't match.
    pub update: bool,
}

impl Builder {
    #[doc(hidden)]
    pub fn new(
        source_manifest_dir: &'static str,
        source_file: &'static str,
        function_path: &'static str,
    ) -> Self {
        Self {
            source_manifest_dir,
            source_file,
            function_path: function_path.split("::").collect(),
            name: None,
            name_prefix: None,
            name_suffix: None,
            extension: None,
            golden_dir: None,
            update: None,
        }
    }

    /// Override the golden file directory
    ///
    /// Defaults to a "testdata" directory next to the source test module.
    pub fn golden_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.golden_dir = Some(dir.into());
        self
    }

    /// Override the name of the assertion
    ///
    /// This defaults to the name of the test function.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add a suffix to the name of the assertion
    ///
    /// This is useful for parameterized tests. Defaults to no prefix.
    ///
    /// It is allowed to contain a path separator.
    pub fn name_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.name_prefix = Some(prefix.into());
        self
    }

    /// Add a suffix to the name of the assertion
    ///
    /// This is useful for parameterized tests. Defaults to no suffix.
    ///
    /// It is allowed to contain a path separator.
    pub fn name_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.name_suffix = Some(suffix.into());
        self
    }

    /// Override the extension of the golden file
    ///
    /// Defaults to "golden".
    pub fn extension(mut self, extension: impl Into<String>) -> Self {
        self.extension = Some(extension.into());
        self
    }

    /// Override whether to update the golden file if it doesn't match
    ///
    /// Defaults to the value of the `GOLDIE_UPDATE` environment variable.
    pub fn update(mut self, update: bool) -> Self {
        self.update = Some(update);
        self
    }

    pub fn build(self) -> Goldie {
        let Self {
            source_manifest_dir,
            source_file,
            function_path,
            name,
            name_prefix,
            name_suffix,
            extension,
            golden_dir,
            update,
        } = self;

        // first figure out the name of the test
        let mut name = name.unwrap_or_else(|| {
            function_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        });
        if let Some(prefix) = &name_prefix {
            name = format!("{prefix}{name}");
        }
        if let Some(suffix) = &name_suffix {
            name = format!("{name}{suffix}");
        }

        let skip_mod = |c: &Component| c.as_os_str() != "lib" && c.as_os_str() != "mod";
        let skip_tests = |c: &Component| c.as_os_str() != "tests";
        let workspace_dir = cargo_workspace_package_dir(source_manifest_dir);

        // Convert a source file path like "src/a/mod.rs" into "src/a"
        let source_file: PathBuf = Path::new(source_file)
            .strip_prefix(workspace_dir)
            .unwrap_or(Path::new(source_file))
            .with_extension("")
            .components()
            .filter(skip_mod)
            .collect();

        // Remove the leading crate name from the function path
        let function_path: PathBuf = function_path.components().skip(1).collect();

        // Find the longest prefix of the function path that is a suffix of the source file
        // E.g. src/a/b, a/b/tests/c/func will find
        //      src, a/b, tests/c/func
        let (source_file, remaining_mods, function_path) =
            match_paths(&source_file, &function_path);

        let golden_dir = golden_dir.unwrap_or_else(|| {
            let mut d = PathBuf::from(source_manifest_dir);
            d.push(source_file);
            d.extend(remaining_mods.components().filter(skip_tests));
            d.push("testdata");
            d
        });

        let golden_file = {
            let fpath: PathBuf = function_path.components().filter(skip_tests).collect();
            let mut f = golden_dir;
            f.push(fpath.parent().unwrap_or(Path::new("")));
            f.push(name);
            f.set_extension(extension.as_deref().unwrap_or("golden"));
            f
        };

        let update = update.unwrap_or_else(|| {
            matches!(
                env::var("GOLDIE_UPDATE").ok().as_deref(),
                Some("1" | "true")
            )
        });

        Goldie {
            golden_file,
            update,
        }
    }
}

fn match_paths(left: &Path, right: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let lc = Vec::from_iter(left.components());
    let rc = Vec::from_iter(right.components());

    // Find the max suffix of lc that is a prefix of rc
    let j = (0..=rc.len())
        .rev()
        .find(|&j| {
            let i = lc.len().saturating_sub(j);
            lc[i..] == rc[..j]
        })
        .unwrap_or(0);

    // Split by that suffix
    let i = lc.len().saturating_sub(j);
    let left = lc[..i].iter().collect();
    let common = lc[i..].iter().collect();
    let right = rc[j..].iter().collect();

    (left, common, right)
}

impl Goldie {
    #[track_caller]
    pub fn assert(&self, actual: impl fmt::Display) {
        let value = format!("{actual}");
        if self.update {
            let dir = self.golden_file.parent().unwrap();
            fs::create_dir_all(dir).expect("create golden dir");
            fs::write(&self.golden_file, &value).expect("write golden file");
        } else {
            let expected = match fs::read_to_string(&self.golden_file) {
                Ok(c) => c,
                Err(err) => {
                    panic!("{}", self.error("failed to read golden file", &err));
                }
            };
            assert_eq!(
                value,
                expected,
                "\n\ngolden file `{}` does not match",
                self.golden_file()
            );
        }
    }

    #[track_caller]
    pub fn assert_alt(&self, actual: impl fmt::Display) {
        struct Wrapper<T>(T);
        impl<T: fmt::Display> fmt::Display for Wrapper<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:#}", self.0)
            }
        }
        self.assert(Wrapper(actual))
    }

    #[track_caller]
    pub fn assert_debug(&self, actual: impl fmt::Debug) {
        struct Wrapper<T>(T);
        impl<T: fmt::Debug> fmt::Display for Wrapper<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
        self.assert(Wrapper(actual))
    }

    #[track_caller]
    pub fn assert_debug_alt(&self, actual: impl fmt::Debug) {
        struct Wrapper<T>(T);
        impl<T: fmt::Debug> fmt::Display for Wrapper<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:#?}", self.0)
            }
        }
        self.assert(Wrapper(actual))
    }

    #[cfg(feature = "template")]
    #[track_caller]
    pub fn assert_template(&self, ctx: impl Serialize, actual: impl AsRef<str>) {
        use std::sync::LazyLock as Lazy;
        static ENGINE: Lazy<upon::Engine> = Lazy::new(|| {
            upon::Engine::with_syntax(upon::Syntax::builder().expr("{{", "}}").build())
        });

        let contents = fs::read_to_string(&self.golden_file).expect("failed to read golden file");
        let expected = ENGINE
            .compile(&contents)
            .expect("failed to compile golden file template")
            .render(&ENGINE, &ctx)
            .to_string()
            .expect("failed to render golden file template");

        assert_eq!(
            actual.as_ref(),
            expected,
            "\n\ngolden file `{}` does not match",
            self.golden_file()
        );
    }

    #[cfg(feature = "json")]
    #[track_caller]
    pub fn assert_json(&self, actual: impl Serialize) {
        if self.update {
            let dir = self.golden_file.parent().unwrap();
            fs::create_dir_all(dir).expect("create golden dir");
            fs::write(
                &self.golden_file,
                serde_json::to_string_pretty(&actual).unwrap(),
            )
            .expect("write golden file");
        } else {
            let contents = match fs::read_to_string(&self.golden_file) {
                Ok(c) => c,
                Err(err) => {
                    panic!("{}", self.error("failed to read golden file", &err));
                }
            };
            let expected: serde_json::Value = match serde_json::from_str(&contents) {
                Ok(v) => v,
                Err(err) => panic!(
                    "{}",
                    self.error("failed to parse golden file as JSON", &err)
                ),
            };
            let actual: serde_json::Value =
                serde_json::to_value(&actual).expect("failed to serialize actual value to JSON");

            assert_eq!(
                actual,
                expected,
                "\n\ngolden file `{}` does not match",
                self.golden_file(),
            );
        }
    }

    fn golden_file(&self) -> impl fmt::Display + '_ {
        let path = match env::current_dir() {
            Ok(cwd) => self
                .golden_file
                .strip_prefix(cwd)
                .unwrap_or(&self.golden_file),
            Err(_) => &self.golden_file,
        };
        path.display()
    }

    #[cfg(feature = "color")]
    fn error(&self, msg: &str, err: &dyn std::error::Error) -> String {
        use yansi::Paint;
        format!(
            "{}: {}\nCaused by: {err}\n💡 run with {} to regenerate the golden file\n\n",
            msg.red(),
            self.golden_file(),
            "GOLDIE_UPDATE=1".blue().bold(),
        )
    }

    #[cfg(not(feature = "color"))]
    fn error(&self, msg: &str, err: &dyn std::error::Error) -> String {
        format!(
            "{}: {}\nCaused by: {err}\nrun with GOLDIE_UPDATE=1 to regenerate the golden file\n\n",
            msg,
            self.golden_file(),
        )
    }
}

/// Not public API.
///
/// Returns the package manifest dir relative to the Cargo workspace root.
///
/// Cargo may report `file!()` paths relative to the workspace root for
/// workspace members, e.g. `bar/src/lib.rs` instead of `src/lib.rs`. This
/// value is used to strip that package prefix before computing the golden file
/// path.
///
/// Not public API.
#[doc(hidden)]
pub fn cargo_workspace_package_dir(manifest_dir: &str) -> Arc<Path> {
    static DIRS: Lazy<Mutex<BTreeMap<String, Arc<Path>>>> =
        Lazy::new(|| Mutex::new(BTreeMap::new()));

    let mut dirs = DIRS.lock().unwrap();

    if let Some(dir) = dirs.get(manifest_dir) {
        return dir.clone();
    }

    let dir = env::var("CARGO_WORKSPACE_DIR")
        .map(|dir| {
            Path::new(manifest_dir)
                .strip_prefix(dir)
                .unwrap_or(Path::new(manifest_dir))
                .to_path_buf()
        })
        .unwrap_or_else(|_| {
            #[derive(Deserialize)]
            struct Manifest {
                workspace_root: PathBuf,
            }
            let cargo = env::var_os("CARGO");
            let cargo = cargo.as_deref().unwrap_or_else(|| OsStr::new("cargo"));
            let output = process::Command::new(cargo)
                .args(["metadata", "--format-version=1", "--no-deps"])
                .current_dir(manifest_dir)
                .output()
                .expect("failed to run `cargo metadata`");
            core::assert!(output.status.success(), "failed to fetch `cargo metadata`");
            let manifest: Manifest = serde_json::from_slice(&output.stdout).unwrap();
            PathBuf::from(manifest_dir)
                .strip_prefix(&manifest.workspace_root)
                .unwrap_or(Path::new(manifest_dir))
                .to_path_buf()
        });

    let dir: Arc<Path> = dir.into_boxed_path().into();
    dirs.insert(String::from(manifest_dir), dir.clone());

    dir
}
