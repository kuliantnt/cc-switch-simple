//! 路径解析模块。
//!
//! 负责确定运行时所有目录和文件的位置：
//!
//! | 路径 | 说明 |
//! |------|------|
//! | `config_dir` | 运行时配置根目录 |
//! | `config_file_path` | `config.toml` 路径 |
//! | `profiles_dir` | profile JSON 存放目录 |
//! | `backups_dir` | 自动备份目录 |
//! | `target_settings_path` | Claude Code 的 `settings.json` |
//!
//! 路径解析规则（`resolve_user_path`）：
//! 1. `~` 或 `~/...` → 相对于用户 home 目录
//! 2. 绝对路径 → 直接使用
//! 3. 相对路径 → 相对于 `config_dir` 解析

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use directories::BaseDirs;

use crate::{AppConfig, ConfigFile};

pub const DEFAULT_MAX_BACKUP_FILES: usize = 5;

/// 一次解析得到的全部运行时路径。
#[derive(Debug, Clone)]
pub struct ResolvedPaths {
    /// 运行时配置根目录。
    pub config_dir: PathBuf,
    /// `config.toml` 完整路径。
    pub config_file_path: PathBuf,
    /// profile JSON 存放目录。
    pub profiles_dir: PathBuf,
    /// 自动备份存放目录。
    pub backups_dir: PathBuf,
    /// Claude Code 目标 `settings.json` 路径。
    pub target_settings_path: PathBuf,
    /// 自动备份最多保留多少个文件。
    pub max_backup_files: usize,
}

impl ResolvedPaths {
    /// 根据 profile 名称构造对应的 JSON 文件路径。
    pub fn profile_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{name}.json"))
    }
}

/// 路径解析器。持有用户目录信息，一次构造可多次调用 `resolve()`。
#[derive(Debug, Clone)]
pub struct PathResolver {
    base_dirs: BaseDirs,
}

impl PathResolver {
    /// 从系统用户目录创建解析器。
    pub fn new() -> Result<Self> {
        let base_dirs =
            BaseDirs::new().ok_or_else(|| anyhow!("Unable to resolve user directories"))?;
        Ok(Self { base_dirs })
    }

    /// 使用指定的 `BaseDirs` 创建解析器（主要用于测试）。
    pub fn from_base_dirs(base_dirs: BaseDirs) -> Self {
        Self { base_dirs }
    }

    /// 解析全部运行时路径。
    ///
    /// 流程：
    /// 1. 确定配置根目录
    /// 2. 读取 `config.toml`（可选）
    /// 3. 如果 `config.toml` 中指定了 `settings_path`，按规则解析；
    ///    否则使用默认路径 `~/.claude/settings.json`
    pub fn resolve(&self) -> Result<ResolvedPaths> {
        let config_dir = default_config_dir(&self.base_dirs);
        let config_file_path = config_dir.join("config.toml");
        let app_config = load_config(&config_file_path)?;

        let default_target = default_target_settings_path(&self.base_dirs);
        let target_settings_path = match app_config.target_settings_path {
            Some(path) => resolve_user_path(&path, &config_dir, self.base_dirs.home_dir())?,
            None => default_target,
        };
        let max_backup_files = app_config
            .max_backup_files
            .unwrap_or(DEFAULT_MAX_BACKUP_FILES);

        Ok(ResolvedPaths {
            profiles_dir: config_dir.join("profiles"),
            backups_dir: config_dir.join("backups"),
            config_dir,
            config_file_path,
            target_settings_path,
            max_backup_files,
        })
    }
}

/// 默认运行时配置目录。
///
/// - Linux/macOS: `~/.cc-switch-simple/`
/// - Windows: `{FOLDERID_RoamingAppData}\cc-switch-simple\`
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

/// Claude Code 默认目标配置文件路径：`~/.claude/settings.json`。
pub fn default_target_settings_path(base_dirs: &BaseDirs) -> PathBuf {
    base_dirs.home_dir().join(".claude").join("settings.json")
}

/// 加载 `config.toml`。文件不存在时返回默认空配置，不报错。
pub fn load_config(path: &Path) -> Result<AppConfig> {
    if !path.is_file() {
        return Ok(AppConfig {
            target_settings_path: None,
            max_backup_files: None,
        });
    }

    let raw =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let file: ConfigFile =
        toml::from_str(&raw).with_context(|| format!("Failed to parse {}", path.display()))?;
    let app_config: AppConfig = file.into();
    if matches!(app_config.max_backup_files, Some(0)) {
        bail!("Configured backups.max_files must be greater than 0");
    }
    Ok(app_config)
}

/// 解析用户在 `config.toml` 中指定的路径。
///
/// 规则：
/// - `~` 或 `~/...` / `~\\...` → 展开为 home 目录
/// - 绝对路径 → 直接使用
/// - 相对路径 → 相对 `config_dir` 解析
/// - 空字符串 → 报错
pub fn resolve_user_path(raw: &str, config_dir: &Path, home_dir: &Path) -> Result<PathBuf> {
    let path = raw.trim();
    if path.is_empty() {
        return Err(anyhow!("Configured settings_path cannot be empty"));
    }

    // 精确匹配 `~`（仅 home 目录本身）
    if path == "~" {
        return Ok(home_dir.to_path_buf());
    }

    // `~/...` 或 `~\...`（Windows 也接受 `~\`）
    if let Some(rest) = path.strip_prefix("~/").or_else(|| path.strip_prefix("~\\")) {
        return Ok(home_dir.join(rest));
    }

    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        return Ok(candidate);
    }

    // 相对路径：相对于 config_dir 解析
    Ok(config_dir.join(candidate))
}
