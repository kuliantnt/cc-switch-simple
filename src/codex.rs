//! Codex 预设切换逻辑。

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use time::OffsetDateTime;

use crate::{
    ResolvedPaths, next_profile_index, should_sync_back, validate_profile_name,
    write_bytes_to_target,
};

/// 一个 Codex profile 条目：名称 + `config.toml` 路径。
///
/// 同目录下的 `auth.json` 必须存在，`models_catalog.json` 可选。
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
    println!(
        "Target model catalog: {}",
        paths.codex_target_models_catalog_path.display()
    );
    print_target_status(
        "Target model catalog status",
        &paths.codex_target_models_catalog_path,
    );

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

    let profile_models_catalog_path =
        optional_models_catalog_path(paths, name)?.map(|path| path.to_path_buf());
    switch_codex_profile(
        paths,
        name,
        &profile_config_path,
        &profile_auth_path,
        profile_models_catalog_path.as_deref(),
    )
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

    let next_models_catalog_path =
        optional_models_catalog_path(paths, &next.name)?.map(|path| path.to_path_buf());
    switch_codex_profile(
        paths,
        &next.name,
        &next.path,
        &next_auth_path,
        next_models_catalog_path.as_deref(),
    )
}

/// `cc-switch cx before`。
pub fn use_before_codex_profile(paths: &ResolvedPaths) -> Result<()> {
    let Some(name) = read_codex_before_name(paths)? else {
        clear_codex_before_name(paths)?;
        println!("No previous Codex profile recorded. Skipped.");
        return Ok(());
    };

    use_codex_profile(paths, &name)
}

/// 扫描 `~/.cc-switch-simple/codex/<name>/config.toml` 和 `auth.json`。
/// `models_catalog.json` 是可选文件，不影响 profile 被列出。
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

/// 读取最近一次成功切换前的 Codex profile 名称。
pub fn read_codex_before_name(paths: &ResolvedPaths) -> Result<Option<String>> {
    if !paths.codex_before_path.is_file() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&paths.codex_before_path)
        .with_context(|| format!("Failed to read {}", paths.codex_before_path.display()))?;
    let name = raw.trim();
    if name.is_empty() || validate_profile_name(name).is_err() {
        return Ok(None);
    }
    if !codex_profile_files_exist(paths, name) {
        return Ok(None);
    }

    Ok(Some(name.to_string()))
}

fn switch_codex_profile(
    paths: &ResolvedPaths,
    name: &str,
    profile_config_path: &Path,
    profile_auth_path: &Path,
    profile_models_catalog_path: Option<&Path>,
) -> Result<()> {
    ensure_codex_runtime_dirs(paths)?;

    ensure_target_file_slot(&paths.codex_target_config_path, "Codex target config")?;
    ensure_target_file_slot(&paths.codex_target_auth_path, "Codex target auth")?;
    ensure_target_file_slot(
        &paths.codex_target_models_catalog_path,
        "Codex target model catalog",
    )?;

    validate_target_auth_json(paths)?;
    validate_target_models_catalog_json(paths)?;

    let config_content = fs::read(profile_config_path)
        .with_context(|| format!("Failed to read {}", profile_config_path.display()))?;
    let auth_content = fs::read(profile_auth_path)
        .with_context(|| format!("Failed to read {}", profile_auth_path.display()))?;
    serde_json::from_slice::<serde_json::Value>(&auth_content)
        .with_context(|| format!("Invalid JSON: {}", profile_auth_path.display()))?;
    let models_catalog_content = profile_models_catalog_path
        .map(|path| {
            let content =
                fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
            serde_json::from_slice::<serde_json::Value>(&content)
                .with_context(|| format!("Invalid JSON: {}", path.display()))?;
            Ok::<_, anyhow::Error>(content)
        })
        .transpose()?;

    let previous_profile = current_codex_profile_name_for_history(paths)?;
    sync_back_current_codex_profile(paths)?;

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

    if paths.codex_target_models_catalog_path.is_file() {
        let backup_path = create_codex_backup(paths, &paths.codex_target_models_catalog_path, now)?;
        println!("Model catalog backup: {}", backup_path.display());
    }

    write_bytes_to_target(&config_content, &paths.codex_target_config_path)?;
    write_bytes_to_target(&auth_content, &paths.codex_target_auth_path)?;
    match (models_catalog_content, profile_models_catalog_path) {
        (Some(content), Some(_)) => {
            write_bytes_to_target(&content, &paths.codex_target_models_catalog_path)?;
            println!(
                "Updated: {}",
                paths.codex_target_models_catalog_path.display()
            );
        }
        (None, None) if paths.codex_target_models_catalog_path.is_file() => {
            fs::remove_file(&paths.codex_target_models_catalog_path).with_context(|| {
                format!(
                    "Failed to remove {}",
                    paths.codex_target_models_catalog_path.display()
                )
            })?;
            println!(
                "Removed stale Codex model catalog: {}",
                paths.codex_target_models_catalog_path.display()
            );
        }
        (None, None) => {}
        _ => unreachable!("model catalog content and path must be present together"),
    }
    write_bytes_to_target(name.as_bytes(), &paths.codex_current_path)?;
    record_codex_before_name(paths, previous_profile.as_deref(), name)?;

    println!("Switched Codex profile: {}", name);
    println!("Updated: {}", paths.codex_target_config_path.display());
    println!("Updated: {}", paths.codex_target_auth_path.display());
    Ok(())
}

fn sync_back_current_codex_profile(paths: &ResolvedPaths) -> Result<()> {
    let Some(name) = read_existing_codex_current_name(paths)? else {
        return Ok(());
    };

    let profile_config_path = paths.codex_profile_path(&name);
    let profile_auth_path = paths.codex_auth_path(&name);
    if !profile_config_path.is_file() || !profile_auth_path.is_file() {
        return Ok(());
    }

    let config_change = pending_sync_file(
        &paths.codex_target_config_path,
        &profile_config_path,
        CodexSyncKind::Config,
    )?;
    let auth_change = pending_sync_file(
        &paths.codex_target_auth_path,
        &profile_auth_path,
        CodexSyncKind::Auth,
    )?;
    let models_catalog_change = if paths.codex_models_catalog_path(&name).is_file() {
        pending_sync_file(
            &paths.codex_target_models_catalog_path,
            &paths.codex_models_catalog_path(&name),
            CodexSyncKind::ModelsCatalog,
        )?
    } else {
        None
    };

    if let Some(change) = auth_change {
        write_bytes_to_target(&change.content, &change.profile_path).with_context(|| {
            format!("Failed to sync current Codex auth.json for profile \"{name}\"")
        })?;
        println!("Synced current Codex profile file: {}", change.file_name);
    }

    if let Some(change) = config_change
        && should_sync_back(&format!(
            "Detected changes in current Codex profile \"{name}\". Sync back before switching? [y/N] "
        ))?
    {
        write_bytes_to_target(&change.content, &change.profile_path)?;
        println!("Synced current Codex profile file: {}", change.file_name);
        println!("Synced current Codex profile: {}", name);
    }

    if let Some(change) = models_catalog_change
        && should_sync_back(&format!(
            "Detected changes in current Codex model catalog \"{name}\". Sync back before switching? [y/N] "
        ))?
    {
        write_bytes_to_target(&change.content, &change.profile_path)?;
        println!("Synced current Codex profile file: {}", change.file_name);
        println!("Synced current Codex model catalog: {}", name);
    }

    Ok(())
}

fn validate_target_auth_json(paths: &ResolvedPaths) -> Result<()> {
    if !paths.codex_target_auth_path.is_file() {
        return Ok(());
    }

    let content = fs::read(&paths.codex_target_auth_path)
        .with_context(|| format!("Failed to read {}", paths.codex_target_auth_path.display()))?;
    serde_json::from_slice::<serde_json::Value>(&content)
        .with_context(|| format!("Invalid JSON: {}", paths.codex_target_auth_path.display()))?;
    Ok(())
}

fn validate_target_models_catalog_json(paths: &ResolvedPaths) -> Result<()> {
    if !paths.codex_target_models_catalog_path.is_file() {
        return Ok(());
    }

    let content = fs::read(&paths.codex_target_models_catalog_path).with_context(|| {
        format!(
            "Failed to read {}",
            paths.codex_target_models_catalog_path.display()
        )
    })?;
    serde_json::from_slice::<serde_json::Value>(&content).with_context(|| {
        format!(
            "Invalid JSON: {}",
            paths.codex_target_models_catalog_path.display()
        )
    })?;
    Ok(())
}

fn read_existing_codex_current_name(paths: &ResolvedPaths) -> Result<Option<String>> {
    let Some(name) = read_codex_current_name(paths)? else {
        return Ok(None);
    };
    if validate_profile_name(&name).is_err() {
        return Ok(None);
    }

    Ok(Some(name))
}

fn codex_profile_files_exist(paths: &ResolvedPaths, name: &str) -> bool {
    paths.codex_profile_path(name).is_file() && paths.codex_auth_path(name).is_file()
}

fn optional_models_catalog_path(paths: &ResolvedPaths, name: &str) -> Result<Option<PathBuf>> {
    let path = paths.codex_models_catalog_path(name);
    if path.exists() && !path.is_file() {
        bail!(
            "Codex profile model catalog is not a file: {}",
            path.display()
        );
    }

    Ok(path.is_file().then_some(path))
}

fn write_codex_before_name(paths: &ResolvedPaths, name: &str) -> Result<()> {
    write_bytes_to_target(name.as_bytes(), &paths.codex_before_path)
}

fn clear_codex_before_name(paths: &ResolvedPaths) -> Result<()> {
    if paths.codex_before_path.is_file() {
        fs::remove_file(&paths.codex_before_path)
            .with_context(|| format!("Failed to remove {}", paths.codex_before_path.display()))?;
    }

    Ok(())
}

fn current_codex_profile_name_for_history(paths: &ResolvedPaths) -> Result<Option<String>> {
    if !paths.codex_target_config_path.is_file() || !paths.codex_target_auth_path.is_file() {
        return Ok(None);
    }

    let Some(name) = read_codex_current_name(paths)? else {
        return Ok(None);
    };
    if validate_profile_name(&name).is_err() || !codex_profile_files_exist(paths, &name) {
        return Ok(None);
    }

    Ok(Some(name))
}

fn record_codex_before_name(
    paths: &ResolvedPaths,
    previous_profile: Option<&str>,
    target_profile: &str,
) -> Result<()> {
    match previous_profile {
        Some(name) if name != target_profile => write_codex_before_name(paths, name),
        Some(_) => Ok(()),
        None => clear_codex_before_name(paths),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodexSyncKind {
    Config,
    Auth,
    ModelsCatalog,
}

struct PendingCodexSync {
    file_name: &'static str,
    profile_path: PathBuf,
    content: Vec<u8>,
}

fn pending_sync_file(
    target_path: &Path,
    profile_path: &Path,
    kind: CodexSyncKind,
) -> Result<Option<PendingCodexSync>> {
    if !target_path.is_file() {
        return Ok(None);
    }

    let target_content = fs::read(target_path)
        .with_context(|| format!("Failed to read {}", target_path.display()))?;
    if matches!(kind, CodexSyncKind::Auth | CodexSyncKind::ModelsCatalog) {
        serde_json::from_slice::<serde_json::Value>(&target_content)
            .with_context(|| format!("Invalid JSON: {}", target_path.display()))?;
    }

    let profile_content = fs::read(profile_path)
        .with_context(|| format!("Failed to read {}", profile_path.display()))?;
    if matches!(kind, CodexSyncKind::Auth | CodexSyncKind::ModelsCatalog) {
        serde_json::from_slice::<serde_json::Value>(&profile_content)
            .with_context(|| format!("Invalid JSON: {}", profile_path.display()))?;
    }
    if target_content == profile_content {
        return Ok(None);
    }

    Ok(Some(PendingCodexSync {
        file_name: match kind {
            CodexSyncKind::Config => "config.toml",
            CodexSyncKind::Auth => "auth.json",
            CodexSyncKind::ModelsCatalog => "models_catalog.json",
        },
        profile_path: profile_path.to_path_buf(),
        content: target_content,
    }))
}

fn ensure_codex_runtime_dirs(paths: &ResolvedPaths) -> Result<()> {
    for dir in [&paths.codex_profiles_dir, &paths.codex_backups_dir] {
        fs::create_dir_all(dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    }

    for path in [
        &paths.codex_target_config_path,
        &paths.codex_target_auth_path,
        &paths.codex_target_models_catalog_path,
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
