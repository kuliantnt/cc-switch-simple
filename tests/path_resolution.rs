use std::fs;

use directories::BaseDirs;
use tempfile::TempDir;

use cc_switch::paths::{default_target_settings_path, load_config, resolve_user_path};

#[test]
fn resolve_user_path_expands_home_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().join("home");
    let config_dir = temp_dir.path().join(".cc-switch-simple");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&config_dir).unwrap();

    let resolved = resolve_user_path("~/custom/settings.json", &config_dir, &home_dir).unwrap();
    assert_eq!(resolved, home_dir.join("custom").join("settings.json"));
}

#[test]
fn resolve_user_path_uses_config_dir_for_relative_paths() {
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().join("home");
    let config_dir = temp_dir.path().join(".cc-switch-simple");
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&config_dir).unwrap();

    let resolved = resolve_user_path("claude/settings.json", &config_dir, &home_dir).unwrap();
    assert_eq!(resolved, config_dir.join("claude").join("settings.json"));
}

#[test]
fn load_config_reads_custom_target_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.toml");
    fs::write(
        &config_file,
        r#"
[claude]
settings_path = "~/Library/Application Support/Claude/settings.json"
"#,
    )
    .unwrap();

    let config = load_config(&config_file).unwrap();
    assert_eq!(
        config.target_settings_path.as_deref(),
        Some("~/Library/Application Support/Claude/settings.json")
    );
    assert_eq!(config.max_backup_files, None);
}

#[test]
fn load_config_reads_backup_limit() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.toml");
    fs::write(
        &config_file,
        r#"
[backups]
max_files = 7
"#,
    )
    .unwrap();

    let config = load_config(&config_file).unwrap();
    assert_eq!(config.max_backup_files, Some(7));
}

#[test]
fn load_config_rejects_zero_backup_limit() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.toml");
    fs::write(
        &config_file,
        r#"
[backups]
max_files = 0
"#,
    )
    .unwrap();

    let error = load_config(&config_file).unwrap_err().to_string();
    assert!(error.contains("backups.max_files must be greater than 0"));
}

#[test]
fn default_target_settings_path_points_to_home_claude_dir() {
    let base_dirs = BaseDirs::new().unwrap();
    let target = default_target_settings_path(&base_dirs);
    assert!(target.ends_with(".claude/settings.json"));
}
