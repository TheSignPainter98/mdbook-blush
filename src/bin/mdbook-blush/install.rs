use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{anyhow, Context, Result};
use indoc::indoc;
use toml_edit::{Array, DocumentMut, Item, Table};

use crate::args::InstallCmd;

pub(crate) fn run_install_command(cmd: InstallCmd) -> ExitCode {
    match run_install_command_impl(cmd) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

pub fn run_install_command_impl(cmd: InstallCmd) -> Result<()> {
    let config = InstallConfig::from(cmd);
    edit_book_toml(&config).context("cannot edit book.toml")?;
    write_css(&config).context("cannot install css")?;
    Ok(())
}

struct InstallConfig {
    book_root_dir: PathBuf,
    css_path: PathBuf,
}

impl From<InstallCmd> for InstallConfig {
    fn from(cmd: InstallCmd) -> Self {
        let InstallCmd {
            book_root_dir,
            css_dir,
        } = cmd;
        let css_path = css_dir.join("blush.css");
        Self {
            book_root_dir,
            css_path,
        }
    }
}

fn edit_book_toml(config: &InstallConfig) -> Result<()> {
    let InstallConfig {
        book_root_dir,
        css_path,
    } = config;
    let mut changed = false;

    let book_path = book_root_dir.join("book.toml");
    let mut book_toml = fs::read_to_string(&book_path)
        .with_context(|| anyhow!("Cannot read {}", book_path.display()))?
        .parse::<DocumentMut>()?;

    let output_table = book_toml
        .entry("output")
        .or_insert_with(|| {
            changed = true;
            implicit_table()
        })
        .as_table_mut()
        .ok_or_else(|| anyhow!("`output` entry must be a table"))?;
    let html_table = output_table
        .entry("html")
        .or_insert_with(|| {
            changed = true;
            implicit_table()
        })
        .as_table_mut()
        .ok_or_else(|| anyhow!("`output.html` entry must be a table"))?;
    let additional_css_array = html_table
        .entry("additional-css")
        .or_insert_with(|| {
            changed = true;
            Array::new().into()
        })
        .as_array_mut()
        .ok_or_else(|| anyhow!("`output.html.additional-css` must be an array"))?;
    if !additional_css_array.iter().any(|entry| {
        entry
            .as_str()
            .is_some_and(|entry_str| entry_str == css_path.as_os_str())
    }) {
        changed = true;
        additional_css_array.push(css_path.to_string_lossy().as_ref());
    }

    let preprocessor_table = book_toml
        .entry("preprocessor")
        .or_insert_with(|| {
            changed = true;
            implicit_table()
        })
        .as_table_mut()
        .ok_or_else(|| anyhow!("`preprocessor` entry must be a table"))?;
    let blush_item = preprocessor_table.entry("blush").or_insert_with(|| {
        changed = true;
        Item::Table(Table::new())
    });
    if !blush_item.is_table() {
        eprintln!("Warning: preprocessor.blush is not a table");
    }

    if changed {
        fs::write(&book_path, book_toml.to_string())
            .with_context(|| anyhow!("Cannot write {}", book_path.display()))?;
    }

    Ok(())
}

fn implicit_table() -> Item {
    let mut table = Table::new();
    table.set_implicit(true);
    Item::Table(table)
}

fn write_css(cmd: &InstallConfig) -> Result<()> {
    let InstallConfig {
        book_root_dir,
        css_path,
    } = cmd;
    write_file(
        book_root_dir.join(css_path),
        indoc! {"
            .small-caps {
                font-variant: small-caps;
            }
        "},
    )?;
    Ok(())
}

pub(crate) fn write_file(path: impl AsRef<Path>, content: impl AsRef<str>) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| anyhow!("cannot create parent directory of {}", path.display()))?;
    }

    fs::write(path, content.as_ref())
        .with_context(|| anyhow!("cannot write to {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use googletest::assert_that;
    use googletest::expect_that;
    use googletest::matchers::{all, contains_substring, eq};
    use insta::assert_snapshot;

    use super::*;

    #[googletest::test]
    fn default() {
        let tempdir = tempfile::tempdir().unwrap();

        let book_toml_path = tempdir.path().join("book.toml");
        write_file(&book_toml_path, "").unwrap();

        let exit_code = run_install_command(InstallCmd {
            book_root_dir: tempdir.path().to_owned(),
            css_dir: PathBuf::from("theme/css"),
        });
        assert_that!(exit_code, eq(ExitCode::SUCCESS));

        let book_toml_content = fs::read_to_string(&book_toml_path).unwrap();
        expect_that!(
            book_toml_content,
            all! {
                contains_substring("[preprocessor.blush]"),
                contains_substring("[output.html]"),
                contains_substring("additional-css = ["),
                contains_substring("theme/css/blush.css"),
            }
        );
        assert_snapshot!(book_toml_content);

        let blush_css_content =
            fs::read_to_string(tempdir.path().join("theme/css").join("blush.css")).unwrap();
        expect_that!(blush_css_content, contains_substring(".small-caps"));
        assert_snapshot!(blush_css_content);

        // Repeat installation has no additional effect.
        let exit_code = run_install_command(InstallCmd {
            book_root_dir: tempdir.path().to_owned(),
            css_dir: PathBuf::from("theme/css"),
        });
        assert_that!(exit_code, eq(ExitCode::SUCCESS));

        let book_toml_content = fs::read_to_string(&book_toml_path).unwrap();
        expect_that!(
            book_toml_content.matches("[preprocessor.blush]").count(),
            eq(1)
        );
        expect_that!(
            book_toml_content.matches("theme/css/blush.css").count(),
            eq(1)
        );
    }
}
