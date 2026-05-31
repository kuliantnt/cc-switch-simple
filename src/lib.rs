//! cc-switch 核心逻辑。
//!
//! # 架构
//!
//! ```text
//! main.rs  →  run()  →  命令分发  →  list / current / use / next / doctor
//!                              └── PathResolver  →  ResolvedPaths
//! ```
//!
//! # Profile 匹配机制
//!
//! Profile 匹配不依赖文件名或元数据，而是直接比较 **规范化 JSON**。
//! 将 profile JSON 和目标 `settings.json` 分别规范化为稳定的字符串
//!（key 排序、无空格），然后做字符串相等比较。
//! 这样即使两个文件格式不同（缩进、key 顺序），只要语义内容一致就能匹配。
//!
//! # 写入安全性
//!
//! `write_profile_to_target` 使用 **先写临时文件再 rename** 的原子写入模式：
//! 1. 在同目录下创建 `.cc-switch-<pid>-<nonce>.tmp`
//! 2. 写入内容，Unix 上设权限 0600
//! 3. 删除目标文件（如果存在）
//! 4. `rename` 临时文件到目标路径
//!
//! 这样即使写入过程中崩溃，也只会留下一个 `.tmp` 文件，
//! 不会损坏目标 `settings.json`。

mod cli;
mod config;
pub mod paths;

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use serde_json::Value;
use time::OffsetDateTime;

pub use cli::{Cli, Commands};
pub use config::{AppConfig, ConfigFile};
pub use paths::{PathResolver, ResolvedPaths};

/// 解析 CLI 参数，解析路径，分发到对应子命令处理函数。
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let paths = PathResolver::new()?.resolve()?;

    match cli.command {
        Commands::List => list_profiles(&paths),
        Commands::Current => show_current(&paths),
        Commands::Use { name } => use_profile(&paths, &name),
        Commands::Next => use_next_profile(&paths),
        Commands::Doctor => doctor(&paths),
    }
}

/// `list` 子命令：列出所有 profile，当前匹配到的用 `*` 标记。
///
/// Profile 按文件名（去掉 `.json` 后缀）字母序排列。
pub fn list_profiles(paths: &ResolvedPaths) -> Result<()> {
    ensure_runtime_dirs(paths)?;
    let profiles = collect_profiles(paths)?;
    if profiles.is_empty() {
        println!("No profiles found in {}", paths.profiles_dir.display());
        return Ok(());
    }

    let current = detect_current_profile_name(paths)?;
    println!("Available profiles:");
    println!();

    for profile in profiles {
        if current.as_deref() == Some(profile.name.as_str()) {
            println!("* {}", profile.name);
        } else {
            println!("  {}", profile.name);
        }
    }

    Ok(())
}

/// `current` 子命令：显示当前目标文件匹配到的 profile 名称。
///
/// 匹配方式：对目标 `settings.json` 和每个 profile 做规范化 JSON 字符串比较。
pub fn show_current(paths: &ResolvedPaths) -> Result<()> {
    ensure_runtime_dirs(paths)?;

    if !paths.target_settings_path.is_file() {
        println!(
            "No target settings file at {}",
            paths.target_settings_path.display()
        );
        return Ok(());
    }

    let current = detect_current_profile_name(paths)?;
    match current {
        Some(name) => println!("Current profile: {}", name),
        None => println!("Current profile: unknown"),
    }
    println!("Target settings: {}", paths.target_settings_path.display());
    Ok(())
}

/// `use <name>` 子命令：切换到指定 profile。
///
/// 流程：
/// 1. 校验 profile 名称合法性
/// 2. 检查 profile 文件是否存在
/// 3. 如果目标已匹配该 profile，跳过
/// 4. 备份当前目标文件（如果存在）
/// 5. 写入 profile 内容到目标文件
pub fn use_profile(paths: &ResolvedPaths, name: &str) -> Result<()> {
    ensure_runtime_dirs(paths)?;
    validate_profile_name(name)?;

    let profile_path = paths.profile_path(name);
    if !profile_path.is_file() {
        bail!("Profile not found: {}", profile_path.display());
    }

    // 规范化比较：如果目标内容和 profile 一致则无需切换
    let target_json = canonical_json_from_file(&profile_path)?;
    if paths.target_settings_path.is_file() {
        let current_json = canonical_json_from_file(&paths.target_settings_path)?;
        if current_json == target_json {
            println!("Already on profile: {}", name);
            return Ok(());
        }

        // 切换前先备份当前配置
        let backup_path = create_backup(
            paths,
            OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
        )?;
        println!("Backup: {}", backup_path.display());
    } else {
        println!("Initializing new target settings file.");
    }

    write_profile_to_target(&profile_path, &paths.target_settings_path)?;
    println!("Switched to profile: {}", name);
    println!("Updated: {}", paths.target_settings_path.display());
    Ok(())
}

/// `next` 子命令：按文件名排序轮换到下一个 profile。
///
/// 如果当前配置无法匹配任何 profile，回退到第一个（index 0）。
/// 只有一个 profile 时会提示 "Only one profile available"。
pub fn use_next_profile(paths: &ResolvedPaths) -> Result<()> {
    ensure_runtime_dirs(paths)?;
    let profiles = collect_profiles(paths)?;
    if profiles.is_empty() {
        bail!("No profiles found in {}", paths.profiles_dir.display());
    }

    let current = detect_current_profile_index(paths, &profiles)?;
    let next_index = next_profile_index(current, profiles.len())
        .ok_or_else(|| anyhow!("No profiles found in {}", paths.profiles_dir.display()))?;

    let next_profile = &profiles[next_index];
    let before = current
        .map(|index| profiles[index].name.as_str())
        .unwrap_or("unknown");

    println!("Before: {}", before);
    println!("After: {}", next_profile.name);

    if paths.target_settings_path.is_file() {
        let backup_path = create_backup(
            paths,
            OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
        )?;
        println!("Backup: {}", backup_path.display());
    } else {
        println!("Initializing new target settings file.");
    }

    write_profile_to_target(&next_profile.path, &paths.target_settings_path)?;

    if profiles.len() == 1 {
        println!("Only one profile available.");
    }

    println!("Switched to profile: {}", next_profile.name);
    println!("Updated: {}", paths.target_settings_path.display());
    Ok(())
}

/// `doctor` 子命令：诊断运行时状态。
///
/// 检查项：
/// - 配置目录、profiles 目录、backups 目录是否存在
/// - 目标父目录是否存在
/// - 目标 settings.json 是否为合法 JSON
/// - 每个 profile 文件是否为合法 JSON
///
/// 有任何 error 级别的问题时，以非零 exit code 退出。
pub fn doctor(paths: &ResolvedPaths) -> Result<()> {
    let mut issues = Vec::new();

    let target_parent = paths
        .target_settings_path
        .parent()
        .ok_or_else(|| anyhow!("Target settings path has no parent directory"))?;

    // 打印所有路径，方便用户确认
    println!("Config dir: {}", paths.config_dir.display());
    println!("Profiles dir: {}", paths.profiles_dir.display());
    println!("Backups dir: {}", paths.backups_dir.display());
    println!("Config file: {}", paths.config_file_path.display());
    println!("Target settings: {}", paths.target_settings_path.display());
    println!();

    // 检查三个运行时目录
    for (label, path) in [
        ("config dir", &paths.config_dir),
        ("profiles dir", &paths.profiles_dir),
        ("backups dir", &paths.backups_dir),
    ] {
        if path.exists() {
            if path.is_dir() {
                println!("[ok] {} exists", label);
            } else {
                println!("[error] {} is not a directory", label);
                issues.push(format!("{} is not a directory", label));
            }
        } else {
            println!("[warn] {} does not exist yet", label);
        }
    }

    // 检查目标文件父目录（通常为 ~/.claude/）
    if target_parent.exists() {
        if target_parent.is_dir() {
            println!("[ok] target parent exists");
        } else {
            println!("[error] target parent is not a directory");
            issues.push("target parent is not a directory".to_string());
        }
    } else {
        println!("[warn] target parent does not exist");
    }

    // 检查目标 settings.json
    if paths.target_settings_path.exists() {
        if paths.target_settings_path.is_file() {
            match canonical_json_from_file(&paths.target_settings_path) {
                Ok(_) => println!("[ok] target settings file is valid JSON"),
                Err(error) => {
                    println!("[error] target settings file is invalid JSON: {error}");
                    issues.push("target settings file is invalid JSON".to_string());
                }
            }
        } else {
            println!("[error] target settings path is not a file");
            issues.push("target settings path is not a file".to_string());
        }
    } else {
        println!("[warn] target settings file does not exist yet");
    }

    // 逐个检查 profile 文件
    let profiles = collect_profiles(paths)?;
    println!("[ok] detected {} profile(s)", profiles.len());
    for profile in &profiles {
        match canonical_json_from_file(&profile.path) {
            Ok(_) => println!("[ok] profile {} is valid JSON", profile.name),
            Err(error) => {
                println!("[error] profile {} is invalid JSON: {error}", profile.name);
                issues.push(format!("profile {} is invalid JSON", profile.name));
            }
        }
    }

    if issues.is_empty() {
        return Ok(());
    }

    let mut message = String::from("Doctor found issues:");
    for issue in issues {
        let _ = write!(message, "\n- {}", issue);
    }
    bail!(message)
}

/// 一个 profile 条目：名称 + 文件路径。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileEntry {
    /// Profile 名称（文件名去掉 `.json` 后缀）。
    pub name: String,
    /// Profile JSON 文件完整路径。
    pub path: PathBuf,
}

/// 扫描 `profiles_dir`，收集所有 `.json` 文件，按名称字母序排列。
///
/// 目录不存在时返回空列表，不报错。
pub fn collect_profiles(paths: &ResolvedPaths) -> Result<Vec<ProfileEntry>> {
    if !paths.profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    for entry in fs::read_dir(&paths.profiles_dir)
        .with_context(|| format!("Failed to read {}", paths.profiles_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        // 只处理 .json 文件
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| anyhow!("Invalid UTF-8 profile name: {}", path.display()))?
            .to_string();
        profiles.push(ProfileEntry { name, path });
    }

    // 按名称字母序排列，保证 list / next 行为可预测
    profiles.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(profiles)
}

/// 检测当前目标文件匹配的 profile 名称。
pub fn detect_current_profile_name(paths: &ResolvedPaths) -> Result<Option<String>> {
    let profiles = collect_profiles(paths)?;
    let index = detect_current_profile_index(paths, &profiles)?;
    Ok(index.map(|value| profiles[value].name.clone()))
}

/// 在 profile 列表中查找与当前目标文件内容匹配的 index。
///
/// 使用规范化 JSON 字符串比较，因此不要求格式完全一致，
/// 只要求 JSON 语义内容相同。
///
/// 返回 `None` 表示：目标文件不存在，或内容不匹配任何已知 profile。
pub fn detect_current_profile_index(
    paths: &ResolvedPaths,
    profiles: &[ProfileEntry],
) -> Result<Option<usize>> {
    if !paths.target_settings_path.is_file() {
        return Ok(None);
    }

    let current = canonical_json_from_file(&paths.target_settings_path)?;
    for (index, profile) in profiles.iter().enumerate() {
        if let Ok(candidate) = canonical_json_from_file(&profile.path) {
            if candidate == current {
                return Ok(Some(index));
            }
        }
    }

    Ok(None)
}

/// 备份当前目标文件到 `backups/` 目录。
///
/// 文件命名格式：`settings-YYYYMMDD-HHMMSS.json`
/// 同一秒内多次备份会追加后缀：`settings-YYYYMMDD-HHMMSS-1.json`
///
/// 备份前会先验证目标文件是合法 JSON，避免备份损坏文件。
pub fn create_backup(paths: &ResolvedPaths, now: OffsetDateTime) -> Result<PathBuf> {
    ensure_runtime_dirs(paths)?;

    if !paths.target_settings_path.is_file() {
        bail!(
            "Target settings file not found: {}",
            paths.target_settings_path.display()
        );
    }

    // 先验证 JSON 有效性，避免备份损坏文件
    canonical_json_from_file(&paths.target_settings_path)?;

    // 同一秒内多次备份时递增 suffix 避免冲突
    let mut suffix = 0_u32;
    loop {
        let file_name = backup_file_name(now, suffix)?;
        let path = paths.backups_dir.join(file_name);
        if path.exists() {
            suffix += 1;
            continue;
        }

        fs::copy(&paths.target_settings_path, &path).with_context(|| {
            format!(
                "Failed to back up {} to {}",
                paths.target_settings_path.display(),
                path.display()
            )
        })?;
        return Ok(path);
    }
}

/// 生成备份文件名。
///
/// - suffix=0: `settings-20260531-203045.json`
/// - suffix>0: `settings-20260531-203045-1.json`
pub fn backup_file_name(now: OffsetDateTime, suffix: u32) -> Result<String> {
    let format = time::macros::format_description!("[year][month][day]-[hour][minute][second]");
    let stamp = now
        .format(&format)
        .context("Failed to format backup timestamp")?;

    if suffix == 0 {
        Ok(format!("settings-{stamp}.json"))
    } else {
        Ok(format!("settings-{stamp}-{suffix}.json"))
    }
}

/// 计算下一个 profile 的索引。
///
/// - `current` 为 `Some(i)` → 返回 `(i+1) % profile_count`
/// - `current` 为 `None`（无法识别当前配置）→ 回退到 0
/// - `profile_count` 为 0 → 返回 `None`
pub fn next_profile_index(current: Option<usize>, profile_count: usize) -> Option<usize> {
    if profile_count == 0 {
        return None;
    }

    Some(match current {
        Some(index) => (index + 1) % profile_count, // 循环轮换
        None => 0,                                  // 无法识别时回退到第一个
    })
}

/// 确保运行时所需的三个目录都存在（递归创建）。
pub fn ensure_runtime_dirs(paths: &ResolvedPaths) -> Result<()> {
    for dir in [&paths.config_dir, &paths.profiles_dir, &paths.backups_dir] {
        fs::create_dir_all(dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    }
    Ok(())
}

/// 校验 profile 名称合法性。
///
/// 规则：
/// - 不能为空
/// - 不能以 `.` 开头（防止隐藏文件）
/// - 只能包含 ASCII 字母数字、`.`、`_`、`-`
pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Profile name cannot be empty");
    }

    if name.starts_with('.') {
        bail!("Profile name cannot start with '.'");
    }

    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        bail!("Invalid profile name: {name}");
    }

    Ok(())
}

/// 将 profile 内容写入目标 settings.json。
///
/// 使用 **原子写入** 策略，避免写入过程中崩溃损坏目标文件：
///
/// 1. 在同目录下创建临时文件 `.cc-switch-<pid>-<nonce>.tmp`
/// 2. 写入内容
/// 3. Unix 上设置权限为 `0600`（仅 owner 可读写）
/// 4. 删除原目标文件
/// 5. `rename` 临时文件到目标路径（同文件系统内是原子操作）
///
/// 如果中途崩溃，最多留下一个 `.tmp` 垃圾文件，目标文件不受影响。
pub fn write_profile_to_target(source: &Path, target: &Path) -> Result<()> {
    let content =
        fs::read(source).with_context(|| format!("Failed to read {}", source.display()))?;

    // 写入前验证 JSON 有效性
    canonical_json_from_slice(&content)?;

    let target_parent = target
        .parent()
        .ok_or_else(|| anyhow!("Target settings path has no parent directory"))?;
    fs::create_dir_all(target_parent)
        .with_context(|| format!("Failed to create {}", target_parent.display()))?;

    // 生成唯一临时文件名：.cc-switch-<进程ID>-<纳秒时间戳>.tmp
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp_name = format!(".cc-switch-{}-{nonce}.tmp", std::process::id());
    let temp_path = target_parent.join(temp_name);

    fs::write(&temp_path, &content)
        .with_context(|| format!("Failed to write {}", temp_path.display()))?;

    // Unix 上确保 settings.json 权限为 0600（可能含 API key）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Failed to chmod {}", temp_path.display()))?;
    }

    // 删除旧文件后再 rename，保证原子替换
    if target.exists() {
        fs::remove_file(target)
            .with_context(|| format!("Failed to remove {}", target.display()))?;
    }

    fs::rename(&temp_path, target).with_context(|| {
        format!(
            "Failed to move {} to {}",
            temp_path.display(),
            target.display()
        )
    })?;
    Ok(())
}

/// 读取文件并返回规范化 JSON 字符串。
pub fn canonical_json_from_file(path: &Path) -> Result<String> {
    let content = fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    canonical_json_from_slice(&content).with_context(|| format!("Invalid JSON: {}", path.display()))
}

/// 将 JSON 字节切片解析为规范化字符串。
pub fn canonical_json_from_slice(content: &[u8]) -> Result<String> {
    let value: Value = serde_json::from_slice(content)?;
    canonical_json_value(&value)
}

/// 将 `serde_json::Value` 序列化为 **规范化 JSON 字符串**。
///
/// 规范化规则：
/// - Object 的 key 按字母序排列
/// - 无多余空格、换行
/// - 数组保持原始顺序
///
/// 这样两个语义相同但格式不同的 JSON 会产生相同的字符串，
/// 可以直接用 `==` 比较来判断 profile 是否匹配。
pub fn canonical_json_value(value: &Value) -> Result<String> {
    match value {
        // 标量类型直接序列化
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_string(value).map_err(Into::into)
        }
        // 数组：保持元素顺序，递归规范化每个元素
        Value::Array(items) => {
            let mut output = String::from("[");
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&canonical_json_value(item)?);
            }
            output.push(']');
            Ok(output)
        }
        // 对象：key 排序后递归规范化每个 value
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();

            let mut output = String::from("{");
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key)?);
                output.push(':');
                output.push_str(&canonical_json_value(&map[*key])?);
            }
            output.push('}');
            Ok(output)
        }
    }
}
