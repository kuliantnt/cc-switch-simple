//! Codex 预设切换逻辑。

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use time::OffsetDateTime;

use crate::{ResolvedPaths, next_profile_index, validate_profile_name, write_bytes_to_target};

/// 一个 Codex profile 条目：名称 + `config.toml` 路径。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexProfileEntry {
    /// Profile 名称（目录名）。
    pub name: String,
    /// 对应的 `config.toml` 路径。
    pub path: PathBuf,
}

/// `cc-switch cx list`。
pub fn list_codex_profiles(paths: &ResolvedPaths) -> Result<()> {
    let profiles = collect_codex_profiles(paths)?;
    if profiles.is_empty() {
        println!(
            "No Codex profiles found in {}",
            paths.codex_profiles_dir.display()
        );
        return Ok(());
    }

    let current = read_codex_current_name(paths)?;
    println!("Available Codex profiles:");
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

/// `cc-switch cx current`。
pub fn show_codex_current(paths: &ResolvedPaths) -> Result<()> {
    let current = read_codex_current_name(paths)?;
    match current {
        Some(name) => println!("Current Codex profile: {}", name),
        None => println!("Current Codex profile: not set"),
    }

    println!(
        "Target config: {}",
        paths.codex_target_config_path.display()
    );
    print_target_status("Target config status", &paths.codex_target_config_path);
    println!("Target auth: {}", paths.codex_target_auth_path.display());
    print_target_status("Target auth status", &paths.codex_target_auth_path);

    Ok(())
}

/// `cc-switch cx use <name>`。
pub fn use_codex_profile(paths: &ResolvedPaths, name: &str) -> Result<()> {
    validate_profile_name(name)?;
    let profile_config_path = paths.codex_profile_path(name);
    let profile_auth_path = paths.codex_auth_path(name);
    if !profile_config_path.is_file() {
        bail!(
            "Codex profile config not found: {}",
            profile_config_path.display()
        );
    }
    if !profile_auth_path.is_file() {
        bail!(
            "Codex profile auth not found: {}",
            profile_auth_path.display()
        );
    }

    switch_codex_profile(paths, name, &profile_config_path, &profile_auth_path)
}

/// `cc-switch cx next`。
pub fn use_next_codex_profile(paths: &ResolvedPaths) -> Result<()> {
    let profiles = collect_codex_profiles(paths)?;
    if profiles.is_empty() {
        bail!(
            "No Codex profiles found in {}",
            paths.codex_profiles_dir.display()
        );
    }

    let current = read_codex_current_name(paths)?;
    let current_index = current
        .as_deref()
        .and_then(|name| profiles.iter().position(|profile| profile.name == name));
    let next_index = next_profile_index(current_index, profiles.len()).ok_or_else(|| {
        anyhow!(
            "No Codex profiles found in {}",
            paths.codex_profiles_dir.display()
        )
    })?;
    let next = &profiles[next_index];

    let before = current.as_deref().unwrap_or("unknown");
    println!("Current: {}", next.name);
    println!("Before: {}", before);

    let next_auth_path = paths.codex_auth_path(&next.name);
    if !next_auth_path.is_file() {
        bail!("Codex profile auth not found: {}", next_auth_path.display());
    }

    switch_codex_profile(paths, &next.name, &next.path, &next_auth_path)
}

/// 扫描 `~/.cc-switch-simple/codex/<name>/config.toml` 和 `auth.json`。
pub fn collect_codex_profiles(paths: &ResolvedPaths) -> Result<Vec<CodexProfileEntry>> {
    if !paths.codex_profiles_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    for entry in fs::read_dir(&paths.codex_profiles_dir)
        .with_context(|| format!("Failed to read {}", paths.codex_profiles_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let config_path = path.join("config.toml");
        if !config_path.is_file() {
            continue;
        }
        if !path.join("auth.json").is_file() {
            continue;
        }

        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow!("Invalid UTF-8 Codex profile name: {}", path.display()))?;
        profiles.push(CodexProfileEntry {
            name,
            path: config_path,
        });
    }

    profiles.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(profiles)
}

/// 读取当前记录的 Codex profile 名称。
pub fn read_codex_current_name(paths: &ResolvedPaths) -> Result<Option<String>> {
    if !paths.codex_current_path.is_file() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&paths.codex_current_path)
        .with_context(|| format!("Failed to read {}", paths.codex_current_path.display()))?;
    let name = raw.trim();
    if name.is_empty() {
        return Ok(None);
    }

    Ok(Some(name.to_string()))
}

fn switch_codex_profile(
    paths: &ResolvedPaths,
    name: &str,
    profile_config_path: &Path,
    profile_auth_path: &Path,
) -> Result<()> {
    ensure_codex_runtime_dirs(paths)?;

    ensure_target_file_slot(&paths.codex_target_config_path, "Codex target config")?;
    ensure_target_file_slot(&paths.codex_target_auth_path, "Codex target auth")?;

    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    if paths.codex_target_config_path.is_file() {
        let backup_path = create_codex_backup(paths, &paths.codex_target_config_path, now)?;
        println!("Config backup: {}", backup_path.display());
    } else {
        println!("Initializing new Codex config.");
    }

    if paths.codex_target_auth_path.is_file() {
        let backup_path = create_codex_backup(paths, &paths.codex_target_auth_path, now)?;
        println!("Auth backup: {}", backup_path.display());
    } else {
        println!("Initializing new Codex auth.");
    }

    let config_content = fs::read(profile_config_path)
        .with_context(|| format!("Failed to read {}", profile_config_path.display()))?;
    let auth_content = fs::read(profile_auth_path)
        .with_context(|| format!("Failed to read {}", profile_auth_path.display()))?;
    write_bytes_to_target(&config_content, &paths.codex_target_config_path)?;
    write_bytes_to_target(&auth_content, &paths.codex_target_auth_path)?;
    write_bytes_to_target(name.as_bytes(), &paths.codex_current_path)?;

    println!("Switched Codex profile: {}", name);
    println!("Updated: {}", paths.codex_target_config_path.display());
    println!("Updated: {}", paths.codex_target_auth_path.display());
    Ok(())
}

fn ensure_codex_runtime_dirs(paths: &ResolvedPaths) -> Result<()> {
    for dir in [&paths.codex_profiles_dir, &paths.codex_backups_dir] {
        fs::create_dir_all(dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    }

    for path in [
        &paths.codex_target_config_path,
        &paths.codex_target_auth_path,
    ] {
        let target_parent = path.parent().ok_or_else(|| {
            anyhow!(
                "Codex target path has no parent directory: {}",
                path.display()
            )
        })?;
        fs::create_dir_all(target_parent)
            .with_context(|| format!("Failed to create {}", target_parent.display()))?;
    }
    Ok(())
}

fn ensure_target_file_slot(path: &Path, label: &str) -> Result<()> {
    if path.exists() && !path.is_file() {
        bail!("{label} path is not a file: {}", path.display());
    }

    Ok(())
}

fn create_codex_backup(
    paths: &ResolvedPaths,
    source_path: &Path,
    now: OffsetDateTime,
) -> Result<PathBuf> {
    ensure_codex_runtime_dirs(paths)?;

    if !source_path.is_file() {
        bail!("Codex backup source not found: {}", source_path.display());
    }

    let content = fs::read(source_path)
        .with_context(|| format!("Failed to read {}", source_path.display()))?;
    let file_stem = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            anyhow!(
                "Invalid Codex backup source file name: {}",
                source_path.display()
            )
        })?;

    let mut suffix = 0_u32;
    loop {
        let file_name = codex_backup_file_name(file_stem, now, suffix)?;
        let path = paths.codex_backups_dir.join(file_name);
        if path.exists() {
            suffix += 1;
            continue;
        }

        fs::write(&path, &content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        prune_codex_backups(paths, file_stem, paths.max_backup_files)?;
        return Ok(path);
    }
}

fn prune_codex_backups(paths: &ResolvedPaths, source_name: &str, keep: usize) -> Result<()> {
    let mut backups = fs::read_dir(&paths.codex_backups_dir)
        .with_context(|| format!("Failed to read {}", paths.codex_backups_dir.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !entry.file_type().ok()?.is_file() {
                return None;
            }

            let file_name = entry.file_name();
            let file_name = file_name.to_str()?;
            let key = parse_codex_backup_sort_key(file_name)?;
            if key.0 != source_name {
                return None;
            }
            Some((key, entry.path()))
        })
        .collect::<Vec<_>>();

    if backups.len() <= keep {
        return Ok(());
    }

    backups.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, path) in backups.into_iter().skip(keep) {
        fs::remove_file(&path)
            .with_context(|| format!("Failed to remove old backup {}", path.display()))?;
    }

    Ok(())
}

fn codex_backup_file_name(file_name: &str, now: OffsetDateTime, suffix: u32) -> Result<String> {
    let format = time::macros::format_description!("[year][month][day]-[hour][minute][second]");
    let stamp = now
        .format(&format)
        .context("Failed to format backup timestamp")?;

    if suffix == 0 {
        Ok(format!("{file_name}.{stamp}.bak"))
    } else {
        Ok(format!("{file_name}.{stamp}-{suffix}.bak"))
    }
}

fn parse_codex_backup_sort_key(file_name: &str) -> Option<(String, String, u32, String)> {
    let stem = file_name.strip_suffix(".bak")?;
    let (source_name, stamp_part) = stem.rsplit_once('.')?;

    let (stamp, suffix) = match stamp_part.rsplit_once('-') {
        Some((prefix, raw_suffix))
            if prefix.len() == 15 && raw_suffix.chars().all(|ch| ch.is_ascii_digit()) =>
        {
            (prefix, raw_suffix.parse().ok()?)
        }
        _ if stamp_part.len() == 15 => (stamp_part, 0),
        _ => return None,
    };

    if !stamp.chars().enumerate().all(|(index, ch)| {
        if index == 8 {
            ch == '-'
        } else {
            ch.is_ascii_digit()
        }
    }) {
        return None;
    }

    Some((
        source_name.to_string(),
        stamp.to_string(),
        suffix,
        file_name.to_string(),
    ))
}

fn print_target_status(label: &str, path: &Path) {
    if path.is_file() {
        println!("{label}: present");
    } else {
        println!("{label}: missing");
    }
}
