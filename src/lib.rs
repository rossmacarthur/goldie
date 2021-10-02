use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;
use tinytemplate::TinyTemplate;

#[derive(Debug)]
pub struct Goldie {
    /// The path to the golden file.
    golden_path: PathBuf,
    /// Whether to update the golden file if it doesn't match.
    update: bool,
}

impl Goldie {
    /// Construct a new golden file tester.
    ///
    /// Where
    /// - `source_path` is path to the source file that the test resides in.
    /// - `function_path` is the full path to the function. e.g.
    ///   `mycrate::module::tests::function_name`.
    pub fn new(source_path: impl AsRef<Path>, function_path: impl AsRef<str>) -> Self {
        let source_path = source_path.as_ref();
        let function_path = function_path.as_ref();
        Self::_new(source_path, function_path)
    }

    fn _new(source_path: &Path, function_path: &str) -> Self {
        let function_name = function_path.rsplitn(2, "::").next().unwrap();
        let module = source_path.file_stem().unwrap();
        let golden_path = {
            let mut p = source_path.parent().unwrap().join("testdata");
            if module != "lib" && module != "mod" {
                p = p.join(module)
            }
            p.join(function_name).with_extension("golden")
        };
        let update = matches!(
            env::var("GOLDIE_UPDATE").ok().as_deref(),
            Some("1" | "true")
        );
        Self {
            golden_path,
            update,
        }
    }

    #[track_caller]
    pub fn assert(&self, expected: impl AsRef<str>) -> Result<()> {
        if self.update {
            let dir = self.golden_path.parent().unwrap();
            fs::create_dir_all(dir)?;
            fs::write(&self.golden_path, expected.as_ref())?;
        } else {
            let actual = fs::read_to_string(&self.golden_path).with_context(|| {
                format!(
                    "failed to read golden file `{}`",
                    self.golden_path.display()
                )
            })?;
            pretty_assertions::assert_eq!(
                actual,
                expected.as_ref(),
                "golden file `{}` does not match",
                self.golden_path
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

        let contents = fs::read_to_string(&self.golden_path).with_context(|| {
            format!(
                "failed to read golden file `{}`",
                self.golden_path.display()
            )
        })?;
        tt.add_template("golden", &contents).with_context(|| {
            format!(
                "failed to compile golden file template `{}`",
                self.golden_path.display()
            )
        })?;
        let actual = tt.render("golden", &ctx).with_context(|| {
            format!(
                "failed to render golden file template `{}`",
                self.golden_path.display()
            )
        })?;

        pretty_assertions::assert_eq!(
            actual,
            expected.as_ref(),
            "golden file `{}` does not match",
            self.golden_path
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

#[doc(hidden)]
#[macro_export]
macro_rules! _new_goldie {
    () => {{
        use ::std::path::Path;
        use ::std::{concat, env, file};
        let source_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/", file!()));
        let function_path = $crate::item_path!();
        $crate::Goldie::new(source_path, function_path)
    }};
}

/// Returns the fully qualified path to the current item.
///
/// Goldie uses this to get the name of the test function.
///
/// # Examples
///
/// ```no_run
/// mod example {
///     fn function() {
///         let path = goldie::item_path!();
///         assert!(path.ends_with("::example::function"));
///     }
/// }
/// ```
#[macro_export]
macro_rules! item_path {
    () => {{
        fn type_name_of<T>(_: T) -> &'static str {
            ::std::any::type_name::<T>()
        }
        fn f() {}
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

#[cfg(test)]
mod tests {
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
}
