use std::env;
use std::path::Path;

use super::{Context, Module, RootModuleConfig, SegmentConfig};
use crate::command::execute;
use crate::configs::python::PythonConfig;

/// Creates a module with the current Python version
///
/// Will display the Python version if any of the following criteria are met:
///     - Current directory contains a `.python-version` file
///     - Current directory contains a `requirements.txt` file
///     - Current directory contains a `pyproject.toml` file
///     - Current directory contains a file with the `.py` extension
///     - Current directory contains a `Pipfile` file
///     - Current directory contains a `tox.ini` file
pub fn module<'a>(context: &'a Context) -> Option<Module<'a>> {
    let is_py_project = context
        .try_begin_scan()?
        .set_files(&[
            "requirements.txt",
            ".python-version",
            "pyproject.toml",
            "Pipfile",
            "tox.ini",
        ])
        .set_extensions(&["py"])
        .is_match();

    let is_venv = env::var("VIRTUAL_ENV").ok().is_some();

    if !is_py_project && !is_venv {
        return None;
    }

    let mut module = context.new_module("python");
    let config: PythonConfig = PythonConfig::try_load(module.config);

    module.set_style(config.style);
    module.create_segment("symbol", &config.symbol);

    if config.pyenv_version_name {
        let python_version = get_pyenv_version()?;
        module.create_segment("pyenv_prefix", &config.pyenv_prefix);
        module.create_segment("version", &SegmentConfig::new(&python_version.trim()));
    } else {
        let python_version = get_python_version()?;
        let formatted_version = format_python_version(&python_version);
        module.create_segment("version", &SegmentConfig::new(&formatted_version));

        if let Some(virtual_env) = get_python_virtual_env() {
            module.create_segment(
                "virtualenv",
                &SegmentConfig::new(&format!(" ({})", virtual_env)),
            );
        };
    };

    Some(module)
}

fn get_pyenv_version() -> Option<String> {
    execute("pyenv version-name")
}

fn get_python_version() -> Option<String> {
    execute("python --version")
}

fn format_python_version(python_stdout: &str) -> String {
    format!("v{}", python_stdout.trim_start_matches("Python ").trim())
}

fn get_python_virtual_env() -> Option<String> {
    env::var("VIRTUAL_ENV").ok().and_then(|venv| {
        Path::new(&venv)
            .file_name()
            .map(|filename| String::from(filename.to_str().unwrap_or("")))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::utils::test::render_module;
    use ansi_term::Color;
    use std::fs::File;
    use std::io;
    use tempfile;

    #[test]
    fn test_format_python_version() {
        let input = "Python 3.7.2";
        assert_eq!(format_python_version(input), "v3.7.2");
    }

    #[test]
    fn folder_with_python_version() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join(".python-version"))?.sync_all()?;

        let actual = render_module("python", dir.path());
        let expected = format!("via {} ", Color::Yellow.bold().paint("🐍 v3.7.4"));
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn folder_with_requirements_txt() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join("requirements.txt"))?.sync_all()?;

        let actual = render_module("python", dir.path());
        let expected = format!("via {} ", Color::Yellow.bold().paint("🐍 v3.7.4"));
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn folder_with_pyproject_toml() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join("pyproject.toml"))?.sync_all()?;

        let actual = render_module("python", dir.path());
        let expected = format!("via {} ", Color::Yellow.bold().paint("🐍 v3.7.4"));
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn folder_with_pipfile() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join("Pipfile"))?.sync_all()?;

        let actual = render_module("python", dir.path());
        let expected = format!("via {} ", Color::Yellow.bold().paint("🐍 v3.7.4"));
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn folder_with_tox() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join("tox.ini"))?.sync_all()?;

        let actual = render_module("python", dir.path());
        let expected = format!("via {} ", Color::Yellow.bold().paint("🐍 v3.7.4"));
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn folder_with_py_file() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join("main.py"))?.sync_all()?;

        let actual = render_module("python", dir.path());
        let expected = format!("via {} ", Color::Yellow.bold().paint("🐍 v3.7.4"));
        assert_eq!(expected, actual);
        Ok(())
    }

    // #[test]
    // fn with_virtual_env() -> io::Result<()> {
    //     let dir = tempfile::tempdir()?;
    //     File::create(dir.path().join("main.py"))?.sync_all()?;
    //     let output = render_module("python", dir.path())
    //         .env("VIRTUAL_ENV", "/foo/bar/my_venv")
    //         .arg("--path")
    //         .arg(dir.path())
    //         .output()?;
    //     let actual = String::from_utf8(output.stdout).unwrap();

    //     let expected = format!("via {} ", Color::Yellow.bold().paint("🐍 v3.7.4 (my_venv)"));
    //     assert_eq!(expected, actual);
    //     Ok(())
    // }

    // #[test]
    // fn with_active_venv() -> io::Result<()> {
    //     let dir = tempfile::tempdir()?;
    //     let output = render_module("python", dir.path())
    //         .env("VIRTUAL_ENV", "/foo/bar/my_venv")
    //         .arg("--path")
    //         .arg(dir.path())
    //         .output()?;
    //     let actual = String::from_utf8(output.stdout).unwrap();

    //     let expected = format!("via {} ", Color::Yellow.bold().paint("🐍 v3.7.4 (my_venv)"));
    //     assert_eq!(expected, actual);
    //     Ok(())
    // }
}
