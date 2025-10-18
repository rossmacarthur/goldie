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
//! In your test function assert the contents using `goldie::assert!`. The golden
//! filename will be automatically determined based on the test file and test
//! function name. Run tests with `GOLDIE_UPDATE=true` to automatically update
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
//! Templated golden files are also supported using `goldie::assert_template!`.
//! Something implementing `serde::Serialize` needs to be provided as context in
//! order to render the template. Values are rendered using
//! [upon](https://github.com/rossmacarthur/upon) e.g. `{{ value.field }}`.
//! You cannot use  `GOLDIE_UPDATE=true` to automatically update templated golden
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

#[cfg(test)]
mod tests;

use std::env;
use std::fmt::Display;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[cfg(feature = "color")]
use pretty_assertions::assert_eq;

use anyhow::{Context, Result};
#[cfg(any(feature = "template", feature = "json"))]
use serde::Serialize;

/// Assert the golden file matches.
#[macro_export]
macro_rules! assert {
    ($actual:expr) => {{
        let g = $crate::builder!().build();
        if let Err(err) = g.assert($actual) {
            ::std::panic!("{}", err);
        }
    }};
}

/// Assert the golden file matches the debug output.
#[macro_export]
macro_rules! assert_debug {
    ($actual:expr) => {{
        let g = $crate::builder!().build();
        if let Err(err) = g.assert_debug($actual) {
            ::std::panic!("{}", err);
        }
    }};
}

/// Assert the templated golden file matches.
#[macro_export]
macro_rules! assert_template {
    ($ctx:expr, $actual:expr) => {{
        let g = $crate::builder!().build();
        if let Err(err) = g.assert_template($ctx, $actual) {
            ::std::panic!("{}", err);
        }
    }};
}

/// Assert the JSON golden file matches.
#[macro_export]
macro_rules! assert_json {
    ($actual:expr) => {{
        let g = $crate::builder!().build();
        if let Err(err) = g.assert_json($actual) {
            ::std::panic!("{}", err);
        }
    }};
}

/// Constructs a new goldie instance.
#[doc(hidden)]
#[macro_export]
macro_rules! builder {
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
    source_manifest_dir: PathBuf,

    /// file!() in the test, e.g. src/module/tests/file.rs
    source_file: PathBuf,

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

#[derive(Debug)]
pub struct Goldie {
    /// The path to the golden file.
    golden_file: PathBuf,
    /// Whether to update the golden file if it doesn't match.
    update: bool,
}

impl Builder {
    #[doc(hidden)]
    pub fn new(
        source_manifest_dir: &'static str,
        source_file: &'static str,
        function_path: &'static str,
    ) -> Self {
        Self {
            source_manifest_dir: PathBuf::from(source_manifest_dir),
            source_file: PathBuf::from(source_file),
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

        // Convert a source file path like "src/a/mod.rs" into "src/a"
        let source_file: PathBuf = source_file
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
            let mut d = source_manifest_dir.clone();
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
    pub fn assert(&self, actual: impl AsRef<str>) -> Result<()> {
        if self.update {
            let dir = self.golden_file.parent().unwrap();
            fs::create_dir_all(dir)?;
            fs::write(&self.golden_file, actual.as_ref())?;
        } else {
            let expected = fs::read_to_string(&self.golden_file)
                .with_context(|| self.error("failed to read golden file"))?;
            assert_eq!(
                actual.as_ref(),
                expected,
                "\n\ngolden file `{}` does not match",
                self.golden_file()
            );
        }
        Ok(())
    }

    #[track_caller]
    pub fn assert_debug(&self, actual: impl std::fmt::Debug) -> Result<()> {
        self.assert(format!("{actual:#?}"))
    }

    #[cfg(feature = "template")]
    #[track_caller]
    pub fn assert_template(&self, ctx: impl Serialize, actual: impl AsRef<str>) -> Result<()> {
        use std::sync::LazyLock as Lazy;
        static ENGINE: Lazy<upon::Engine> = Lazy::new(|| {
            upon::Engine::with_syntax(upon::Syntax::builder().expr("{{", "}}").build())
        });

        let contents = fs::read_to_string(&self.golden_file)
            .with_context(|| self.error("failed to read golden file"))?;
        let expected = ENGINE
            .compile(&contents)
            .with_context(|| self.error("failed to compile golden file template"))?
            .render(&ENGINE, &ctx)
            .to_string()
            .with_context(|| self.error("failed to render golden file template"))?;

        assert_eq!(
            actual.as_ref(),
            expected,
            "\n\ngolden file `{}` does not match",
            self.golden_file()
        );

        Ok(())
    }

    #[cfg(feature = "json")]
    #[track_caller]
    pub fn assert_json(&self, actual: impl Serialize) -> Result<()> {
        if self.update {
            let dir = self.golden_file.parent().unwrap();
            fs::create_dir_all(dir)?;
            fs::write(
                &self.golden_file,
                serde_json::to_string_pretty(&actual).unwrap(),
            )?;
        } else {
            let contents = fs::read_to_string(&self.golden_file)
                .with_context(|| self.error("failed to read golden file"))?;
            let expected: serde_json::Value =
                serde_json::from_str(&contents).with_context(|| self.error("bad JSON"))?;
            let actual: serde_json::Value = serde_json::to_value(&actual)?;

            assert_eq!(
                actual,
                expected,
                "\n\ngolden file `{}` does not match",
                self.golden_file(),
            );
        }

        Ok(())
    }

    fn golden_file(&self) -> impl Display + '_ {
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
    fn error(&self, msg: &str) -> String {
        use yansi::Paint;
        format!(
            "\n\n{}: {}\nrun with {} to regenerate the golden file\n\n",
            msg.red(),
            self.golden_file(),
            "GOLDIE_UPDATE=1".blue().bold(),
        )
    }

    #[cfg(not(feature = "color"))]
    fn error(&self, msg: &str) -> String {
        format!(
            "\n\n{}: {}\nrun with GOLDIE_UPDATE=1 to regenerate the golden file\n\n",
            msg,
            self.golden_file(),
        )
    }
}
