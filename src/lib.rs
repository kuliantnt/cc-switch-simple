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

pub fn use_profile(paths: &ResolvedPaths, name: &str) -> Result<()> {
    ensure_runtime_dirs(paths)?;
    validate_profile_name(name)?;

    let profile_path = paths.profile_path(name);
    if !profile_path.is_file() {
        bail!("Profile not found: {}", profile_path.display());
    }

    let target_json = canonical_json_from_file(&profile_path)?;
    if paths.target_settings_path.is_file() {
        let current_json = canonical_json_from_file(&paths.target_settings_path)?;
        if current_json == target_json {
            println!("Already on profile: {}", name);
            return Ok(());
        }

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

pub fn doctor(paths: &ResolvedPaths) -> Result<()> {
    let mut issues = Vec::new();

    let target_parent = paths
        .target_settings_path
        .parent()
        .ok_or_else(|| anyhow!("Target settings path has no parent directory"))?;

    println!("Config dir: {}", paths.config_dir.display());
    println!("Profiles dir: {}", paths.profiles_dir.display());
    println!("Backups dir: {}", paths.backups_dir.display());
    println!("Config file: {}", paths.config_file_path.display());
    println!("Target settings: {}", paths.target_settings_path.display());
    println!();

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileEntry {
    pub name: String,
    pub path: PathBuf,
}

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

    profiles.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(profiles)
}

pub fn detect_current_profile_name(paths: &ResolvedPaths) -> Result<Option<String>> {
    let profiles = collect_profiles(paths)?;
    let index = detect_current_profile_index(paths, &profiles)?;
    Ok(index.map(|value| profiles[value].name.clone()))
}

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

pub fn create_backup(paths: &ResolvedPaths, now: OffsetDateTime) -> Result<PathBuf> {
    ensure_runtime_dirs(paths)?;

    if !paths.target_settings_path.is_file() {
        bail!(
            "Target settings file not found: {}",
            paths.target_settings_path.display()
        );
    }

    canonical_json_from_file(&paths.target_settings_path)?;

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

pub fn next_profile_index(current: Option<usize>, profile_count: usize) -> Option<usize> {
    if profile_count == 0 {
        return None;
    }

    Some(match current {
        Some(index) => (index + 1) % profile_count,
        None => 0,
    })
}

pub fn ensure_runtime_dirs(paths: &ResolvedPaths) -> Result<()> {
    for dir in [&paths.config_dir, &paths.profiles_dir, &paths.backups_dir] {
        fs::create_dir_all(dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    }
    Ok(())
}

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

pub fn write_profile_to_target(source: &Path, target: &Path) -> Result<()> {
    let content =
        fs::read(source).with_context(|| format!("Failed to read {}", source.display()))?;
    canonical_json_from_slice(&content)?;

    let target_parent = target
        .parent()
        .ok_or_else(|| anyhow!("Target settings path has no parent directory"))?;
    fs::create_dir_all(target_parent)
        .with_context(|| format!("Failed to create {}", target_parent.display()))?;

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp_name = format!(".cc-switch-{}-{nonce}.tmp", std::process::id());
    let temp_path = target_parent.join(temp_name);

    fs::write(&temp_path, &content)
        .with_context(|| format!("Failed to write {}", temp_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Failed to chmod {}", temp_path.display()))?;
    }

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

pub fn canonical_json_from_file(path: &Path) -> Result<String> {
    let content = fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    canonical_json_from_slice(&content).with_context(|| format!("Invalid JSON: {}", path.display()))
}

pub fn canonical_json_from_slice(content: &[u8]) -> Result<String> {
    let value: Value = serde_json::from_slice(content)?;
    canonical_json_value(&value)
}

pub fn canonical_json_value(value: &Value) -> Result<String> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_string(value).map_err(Into::into)
        }
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
