//! 配置文件解析（`config.toml`）。
//!
//! `config.toml` 位于运行时配置目录。目前仅支持覆盖 Claude Code
//! 目标配置文件路径（默认 `~/.claude/settings.json`）。
//!
//! 示例：
//! ```toml
//! [claude]
//! settings_path = "~/.claude/settings.json"
//!
//! [backups]
//! max_files = 5
//! ```

use serde::Deserialize;

/// `config.toml` 的磁盘表示。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub claude: ClaudeConfig,
    #[serde(default)]
    pub backups: BackupsConfig,
}

/// `[claude]` 段。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClaudeConfig {
    /// 可选：覆盖 Claude Code 目标 settings.json 路径。
    pub settings_path: Option<String>,
}

/// `[backups]` 段。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BackupsConfig {
    /// 可选：最多保留多少个自动备份文件。
    pub max_files: Option<usize>,
}

/// 解析后供应用使用的配置。从 `ConfigFile` 扁平化而来。
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 用户指定的目标配置文件路径。`None` 表示使用默认路径。
    pub target_settings_path: Option<String>,
    /// 自动备份最多保留多少个文件。`None` 表示使用默认值。
    pub max_backup_files: Option<usize>,
}

impl From<ConfigFile> for AppConfig {
    fn from(value: ConfigFile) -> Self {
        Self {
            target_settings_path: value.claude.settings_path,
            max_backup_files: value.backups.max_files,
        }
    }
}
