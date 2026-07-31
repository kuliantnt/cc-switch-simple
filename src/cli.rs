//! CLI 参数定义（clap derive 模式）。
//!
//! 子命令：
//! - `list`    列出所有可用 profile，当前匹配到的用 `*` 标记
//! - `current` 显示当前目标配置文件匹配到的 profile 名称
//! - `use`     切换到指定名称的 profile
//! - `next`    按文件名排序轮换到下一个 profile
//! - `before`  切换到最近一次成功切换前的 profile
//! - `doctor`  诊断：检查目录、配置路径、JSON 有效性
//! - `cx-switch` 直接切换 Codex 预设

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

/// `cx-switch` 的直接命令行入口。
#[derive(Debug, Parser)]
#[command(
    name = "cx-switch",
    version,
    about = "Codex profile switcher",
    arg_required_else_help = true
)]
pub struct CodexCli {
    #[command(subcommand)]
    pub command: CodexCommands,
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
    /// 切换到最近一次成功切换前的 profile。
    Before,
    /// 切换 Codex 预设（`config.toml` + `auth.json`，可选 `models_catalog.json`）。
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
    /// 切换到指定名称的 Codex 预设（同时写入配置、认证和可选模型目录）。
    Use { name: String },
    /// 按名称排序轮换到下一个 Codex 预设。
    Next,
    /// 切换到最近一次成功切换前的 Codex 预设。
    Before,
}
