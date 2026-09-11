use std::path::{Path, PathBuf};

use crate::args::BackendKind;

pub const BUILT_IN_BACKEND: BackendKind = BackendKind::Cranelift;
const CONFIG_FILE_NAME: &str = "config";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FurnaceConfig {
    pub backend: BackendKind,
}

pub fn resolve_backend(explicit: Option<BackendKind>) -> Result<BackendKind, String> {
    if let Some(backend) = explicit {
        return Ok(backend);
    }
    let path = config_path()?;
    resolve_backend_from(&path, None)
}

pub fn save_backend(backend: BackendKind) -> Result<PathBuf, String> {
    let path = config_path()?;
    save_backend_to(&path, backend)?;
    Ok(path)
}

fn config_path() -> Result<PathBuf, String> {
    if let Some(directory) = std::env::var_os("FURNACE_CONFIG_DIR") {
        if directory.is_empty() {
            return Err("FURNACE_CONFIG_DIR cannot be empty".into());
        }
        return Ok(PathBuf::from(directory).join(CONFIG_FILE_NAME));
    }

    #[cfg(windows)]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(not(windows))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")));

    base.map(|directory| directory.join("furnace").join(CONFIG_FILE_NAME))
        .ok_or_else(|| "cannot determine Furnace configuration directory".into())
}

fn resolve_backend_from(path: &Path, explicit: Option<BackendKind>) -> Result<BackendKind, String> {
    if let Some(backend) = explicit {
        return Ok(backend);
    }
    Ok(load_from(path)?.map_or(BUILT_IN_BACKEND, |config| config.backend))
}

fn load_from(path: &Path) -> Result<Option<FurnaceConfig>, String> {
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "cannot read Furnace configuration '{}': {}",
                path.display(),
                error
            ))
        }
    };
    parse_config(&contents).map(Some).map_err(|message| {
        format!(
            "malformed Furnace configuration '{}': {}",
            path.display(),
            message
        )
    })
}

fn parse_config(contents: &str) -> Result<FurnaceConfig, String> {
    let mut backend = None;
    for (index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("invalid syntax at line {}", index + 1))?;
        if key.trim() != "backend" {
            return Err(format!(
                "unknown setting '{}' at line {}",
                key.trim(),
                index + 1
            ));
        }
        if backend.is_some() {
            return Err(format!("duplicate backend setting at line {}", index + 1));
        }
        let value = value.trim();
        let name = value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .ok_or_else(|| format!("backend must be a quoted string at line {}", index + 1))?;
        backend = Some(BackendKind::parse(name).ok_or_else(|| {
            format!(
                "unknown backend '{}'; available backends: native, cranelift",
                name
            )
        })?);
    }

    backend
        .map(|backend| FurnaceConfig { backend })
        .ok_or_else(|| "backend setting is required".into())
}

fn save_backend_to(path: &Path, backend: BackendKind) -> Result<(), String> {
    let directory = path
        .parent()
        .ok_or_else(|| "configuration path has no parent directory".to_string())?;
    std::fs::create_dir_all(directory).map_err(|error| {
        format!(
            "cannot create Furnace configuration directory '{}': {}",
            directory.display(),
            error
        )
    })?;
    std::fs::write(path, format!("backend = \"{}\"\n", backend.name())).map_err(|error| {
        format!(
            "cannot write Furnace configuration '{}': {}",
            path.display(),
            error
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_config_uses_existing_cranelift_default() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(
            resolve_backend_from(&temp.path().join("missing"), None).unwrap(),
            BackendKind::Cranelift
        );
    }

    #[test]
    fn native_and_cranelift_persist_across_separate_loads() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config");
        save_backend_to(&path, BackendKind::Native).unwrap();
        assert_eq!(
            load_from(&path).unwrap().unwrap().backend,
            BackendKind::Native
        );
        assert_eq!(
            load_from(&path).unwrap().unwrap().backend,
            BackendKind::Native
        );

        save_backend_to(&path, BackendKind::Cranelift).unwrap();
        assert_eq!(
            load_from(&path).unwrap().unwrap().backend,
            BackendKind::Cranelift
        );
    }

    #[test]
    fn explicit_override_wins_even_when_saved_config_is_malformed() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config");
        std::fs::write(&path, "this is broken").unwrap();
        assert_eq!(
            resolve_backend_from(&path, Some(BackendKind::Native)).unwrap(),
            BackendKind::Native
        );
    }

    #[test]
    fn malformed_and_unknown_saved_backends_are_errors() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config");
        std::fs::write(&path, "this is broken").unwrap();
        assert!(load_from(&path)
            .unwrap_err()
            .contains("malformed Furnace configuration"));
        std::fs::write(&path, "backend = \"llvm\"\n").unwrap();
        assert!(load_from(&path)
            .unwrap_err()
            .contains("unknown backend 'llvm'"));
    }

    #[test]
    fn save_reports_configuration_directory_failures() {
        let temp = tempfile::tempdir().unwrap();
        let blocker = temp.path().join("not-a-directory");
        std::fs::write(&blocker, "file").unwrap();
        let error = save_backend_to(&blocker.join("config"), BackendKind::Native).unwrap_err();
        assert!(error.contains("cannot create Furnace configuration directory"));
    }
}
