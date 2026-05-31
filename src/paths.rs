use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use directories::BaseDirs;

use crate::{AppConfig, ConfigFile};

#[derive(Debug, Clone)]
pub struct ResolvedPaths {
    pub config_dir: PathBuf,
    pub config_file_path: PathBuf,
    pub profiles_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub target_settings_path: PathBuf,
}

impl ResolvedPaths {
    pub fn profile_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{name}.json"))
    }
}

#[derive(Debug, Clone)]
pub struct PathResolver {
    base_dirs: BaseDirs,
}

impl PathResolver {
    pub fn new() -> Result<Self> {
        let base_dirs =
            BaseDirs::new().ok_or_else(|| anyhow!("Unable to resolve user directories"))?;
        Ok(Self { base_dirs })
    }

    pub fn from_base_dirs(base_dirs: BaseDirs) -> Self {
        Self { base_dirs }
    }

    pub fn resolve(&self) -> Result<ResolvedPaths> {
        let config_dir = default_config_dir(&self.base_dirs);
        let config_file_path = config_dir.join("config.toml");
        let app_config = load_config(&config_file_path)?;

        let default_target = default_target_settings_path(&self.base_dirs);
        let target_settings_path = match app_config.target_settings_path {
            Some(path) => resolve_user_path(&path, &config_dir, self.base_dirs.home_dir())?,
            None => default_target,
        };

        Ok(ResolvedPaths {
            profiles_dir: config_dir.join("profiles"),
            backups_dir: config_dir.join("backups"),
            config_dir,
            config_file_path,
            target_settings_path,
        })
    }
}

pub fn default_config_dir(base_dirs: &BaseDirs) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        return base_dirs.config_dir().join("cc-switch-simple");
    }

    #[cfg(not(target_os = "windows"))]
    {
        base_dirs.home_dir().join(".cc-switch-simple")
    }
}

pub fn default_target_settings_path(base_dirs: &BaseDirs) -> PathBuf {
    base_dirs.home_dir().join(".claude").join("settings.json")
}

pub fn load_config(path: &Path) -> Result<AppConfig> {
    if !path.is_file() {
        return Ok(AppConfig {
            target_settings_path: None,
        });
    }

    let raw =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let file: ConfigFile =
        toml::from_str(&raw).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(file.into())
}

pub fn resolve_user_path(raw: &str, config_dir: &Path, home_dir: &Path) -> Result<PathBuf> {
    let path = raw.trim();
    if path.is_empty() {
        return Err(anyhow!("Configured settings_path cannot be empty"));
    }

    if path == "~" {
        return Ok(home_dir.to_path_buf());
    }

    if let Some(rest) = path.strip_prefix("~/").or_else(|| path.strip_prefix("~\\")) {
        return Ok(home_dir.join(rest));
    }

    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        return Ok(candidate);
    }

    Ok(config_dir.join(candidate))
}
