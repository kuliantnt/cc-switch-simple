//! 配置文件解析（`config.toml`）。
//!
//! `config.toml` 位于运行时配置目录。目前仅支持覆盖 Claude Code
//! 目标配置文件路径（默认 `~/.claude/settings.json`）。
//!
//! 示例：
//! ```toml
//! [claude]
//! settings_path = "~/.claude/settings.json"
//! ```

use serde::Deserialize;

/// `config.toml` 的磁盘表示。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub claude: ClaudeConfig,
}

/// `[claude]` 段。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClaudeConfig {
    /// 可选：覆盖 Claude Code 目标 settings.json 路径。
    pub settings_path: Option<String>,
}

/// 解析后供应用使用的配置。从 `ConfigFile` 扁平化而来。
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 用户指定的目标配置文件路径。`None` 表示使用默认路径。
    pub target_settings_path: Option<String>,
}

impl From<ConfigFile> for AppConfig {
    fn from(value: ConfigFile) -> Self {
        Self {
            target_settings_path: value.claude.settings_path,
        }
    }
}
