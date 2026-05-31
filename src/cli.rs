//! CLI 参数定义（clap derive 模式）。
//!
//! 子命令：
//! - `list`    列出所有可用 profile，当前匹配到的用 `*` 标记
//! - `current` 显示当前目标配置文件匹配到的 profile 名称
//! - `use`     切换到指定名称的 profile
//! - `next`    按文件名排序轮换到下一个 profile
//! - `doctor`  诊断：检查目录、配置路径、JSON 有效性

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "cc-switch",
    version,
    about = "Cross-platform Claude Code profile switcher",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 列出所有可用 profile，当前匹配到的用 `*` 标记。
    List,
    /// 显示目标配置文件当前匹配的 profile 名称。
    Current,
    /// 切换到指定名称的 profile（先备份再写入）。
    Use { name: String },
    /// 按文件名排序轮换到下一个 profile（先备份再写入）。
    Next,
    /// 切换 Codex 预设（`config.toml` + `auth.json`）。
    Cx {
        #[command(subcommand)]
        command: CodexCommands,
    },
    /// 诊断：检查运行时目录、配置路径、JSON 有效性。
    Doctor,
}

#[derive(Debug, Subcommand)]
pub enum CodexCommands {
    /// 列出所有可用的 Codex 预设。
    List,
    /// 显示当前选中的 Codex 预设。
    Current,
    /// 切换到指定名称的 Codex 预设（同时写入 `config.toml` 和 `auth.json`）。
    Use { name: String },
    /// 按名称排序轮换到下一个 Codex 预设。
    Next,
}
