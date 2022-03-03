#[cfg(test)]
mod tests;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;
use tinytemplate::TinyTemplate;

#[derive(Debug)]
pub struct Goldie {
    /// The path to the golden file.
    golden_file: PathBuf,
    /// Whether to update the golden file if it doesn't match.
    update: bool,
}

impl Goldie {
    /// Construct a new golden file tester.
    ///
    /// Where
    /// - `source_file` is path to the source file that the test resides in.
    /// - `function_path` is the full path to the function. e.g.
    ///   `crate::module::tests::function_name`.
    pub fn new(source_file: impl AsRef<Path>, function_path: impl AsRef<str>) -> Self {
        Self::_new(source_file.as_ref(), function_path.as_ref())
    }

    fn _new(source_file: &Path, function_path: &str) -> Self {
        let (_, name) = function_path.rsplit_once("::").unwrap();

        let golden_file = {
            let mut p = source_file.parent().unwrap().to_owned();
            p.push("testdata");
            p.push(name);
            p.set_extension("golden");
            p
        };

        let update = matches!(
            env::var("GOLDIE_UPDATE").ok().as_deref(),
            Some("1" | "true")
        );

        Self {
            golden_file,
            update,
        }
    }

    #[track_caller]
    pub fn assert(&self, expected: impl AsRef<str>) -> Result<()> {
        if self.update {
            let dir = self.golden_file.parent().unwrap();
            fs::create_dir_all(dir)?;
            fs::write(&self.golden_file, expected.as_ref())?;
        } else {
            let actual = fs::read_to_string(&self.golden_file).with_context(|| {
                format!(
                    "failed to read golden file `{}`",
                    self.golden_file.display()
                )
            })?;
            pretty_assertions::assert_eq!(
                actual,
                expected.as_ref(),
                "golden file `{}` does not match",
                self.golden_file
                    .strip_prefix(env::current_dir()?)?
                    .display(),
            );
        }
        Ok(())
    }

    #[track_caller]
    pub fn assert_template(&self, ctx: impl Serialize, expected: impl AsRef<str>) -> Result<()> {
        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&tinytemplate::format_unescaped);

        let contents = fs::read_to_string(&self.golden_file).with_context(|| {
            format!(
                "failed to read golden file `{}`",
                self.golden_file.display()
            )
        })?;
        tt.add_template("golden", &contents).with_context(|| {
            format!(
                "failed to compile golden file template `{}`",
                self.golden_file.display()
            )
        })?;
        let actual = tt.render("golden", &ctx).with_context(|| {
            format!(
                "failed to render golden file template `{}`",
                self.golden_file.display()
            )
        })?;

        pretty_assertions::assert_eq!(
            actual,
            expected.as_ref(),
            "golden file `{}` does not match",
            self.golden_file
                .strip_prefix(env::current_dir()?)?
                .display(),
        );

        Ok(())
    }
}

/// Assert the golden file matches.
#[macro_export]
macro_rules! assert {
    ($expected:expr) => {{
        let g = $crate::_new_goldie!();
        g.assert($expected).unwrap();
    }};
}

/// Assert the templated golden file matches.
#[macro_export]
macro_rules! assert_template {
    ($ctx:expr, $expected:expr) => {{
        let g = $crate::_new_goldie!();
        g.assert_template($ctx, $expected).unwrap();
    }};
}

/// Constructs a new goldie instance.
#[doc(hidden)]
#[macro_export]
macro_rules! _new_goldie {
    () => {{
        use ::std::path::Path;
        use ::std::{concat, env, file};
        let source_file = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/", file!()));
        let function_path = $crate::_function_path!();
        $crate::Goldie::new(source_file, function_path)
    }};
}

/// Returns the fully qualified path to the current item.
///
/// Goldie uses this to get the name of the test function.
#[doc(hidden)]
#[macro_export]
macro_rules! _function_path {
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
