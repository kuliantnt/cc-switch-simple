use std::fs;

use cc_switch::{
    Cli, CodexCli, CodexCommands, Commands, ResolvedPaths, backup_file_name, collect_profiles,
    create_backup, detect_current_profile_index, next_profile_index, read_before_profile_name,
    read_current_profile_name, use_before_profile, use_next_profile, use_profile,
};
use clap::Parser;
use tempfile::TempDir;
use time::macros::datetime;

#[test]
fn cx_switch_parser_accepts_direct_codex_commands() {
    let cli = CodexCli::try_parse_from(["cx-switch", "use", "deepseek"]).unwrap();

    assert!(matches!(
        cli.command,
        CodexCommands::Use { name } if name == "deepseek"
    ));
}

#[test]
fn cc_switch_keeps_nested_codex_commands() {
    let cli = Cli::try_parse_from(["cc-switch", "cx", "list"]).unwrap();

    assert!(matches!(
        cli.command,
        Commands::Cx {
            command: CodexCommands::List
        }
    ));
}

#[test]
fn collect_profiles_returns_sorted_names() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("zeta", r#"{"name":"zeta"}"#);
    sandbox.write_profile("alpha", r#"{"name":"alpha"}"#);
    sandbox.write_profile("beta", r#"{"name":"beta"}"#);
    fs::write(sandbox.paths.profiles_dir.join("notes.txt"), "ignore me").unwrap();

    let profiles = collect_profiles(&sandbox.paths).unwrap();
    let names = profiles
        .into_iter()
        .map(|profile| profile.name)
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["alpha", "beta", "zeta"]);
}

#[test]
fn next_profile_index_rotates_and_wraps() {
    assert_eq!(next_profile_index(None, 3), Some(0));
    assert_eq!(next_profile_index(Some(0), 3), Some(1));
    assert_eq!(next_profile_index(Some(2), 3), Some(0));
    assert_eq!(next_profile_index(Some(0), 1), Some(0));
    assert_eq!(next_profile_index(None, 0), None);
}

#[test]
fn backup_file_name_uses_timestamp_and_suffix() {
    let now = datetime!(2026-05-31 13:45:09 +08:00);
    assert_eq!(
        backup_file_name(now, 0).unwrap(),
        "settings-20260531-134509.json"
    );
    assert_eq!(
        backup_file_name(now, 2).unwrap(),
        "settings-20260531-134509-2.json"
    );
}

#[test]
fn create_backup_prunes_old_backups_and_keeps_latest_five() {
    let sandbox = Sandbox::new();
    fs::write(&sandbox.paths.target_settings_path, r#"{"active":true}"#).unwrap();

    for name in [
        "settings-20260531-134500.json",
        "settings-20260531-134501.json",
        "settings-20260531-134502.json",
        "settings-20260531-134503.json",
        "settings-20260531-134504.json",
    ] {
        fs::write(sandbox.paths.backups_dir.join(name), r#"{"old":true}"#).unwrap();
    }

    let backup_path = create_backup(&sandbox.paths, datetime!(2026-05-31 13:45:05 +08:00)).unwrap();

    assert_eq!(
        backup_path.file_name().and_then(|name| name.to_str()),
        Some("settings-20260531-134505.json")
    );

    let mut names = fs::read_dir(&sandbox.paths.backups_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            entry
                .file_type()
                .unwrap()
                .is_file()
                .then(|| entry.file_name().into_string().unwrap())
        })
        .collect::<Vec<_>>();
    names.sort();

    assert_eq!(
        names,
        vec![
            "settings-20260531-134501.json",
            "settings-20260531-134502.json",
            "settings-20260531-134503.json",
            "settings-20260531-134504.json",
            "settings-20260531-134505.json",
        ]
    );
}

#[test]
fn detect_current_profile_index_matches_canonical_json() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("official", r#"{"env":{"A":1,"B":2},"mcp":["a","b"]}"#);
    sandbox.write_profile("other", r#"{"env":{"A":3}}"#);
    fs::write(
        &sandbox.paths.target_settings_path,
        r#"{"mcp":["a","b"],"env":{"B":2,"A":1}}"#,
    )
    .unwrap();

    let profiles = collect_profiles(&sandbox.paths).unwrap();
    let index = detect_current_profile_index(&sandbox.paths, &profiles).unwrap();

    assert_eq!(index, Some(0));
}

#[test]
fn use_profile_records_current_profile_name() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("official", r#"{"name":"official"}"#);

    use_profile(&sandbox.paths, "official").unwrap();

    assert_eq!(
        read_current_profile_name(&sandbox.paths)
            .unwrap()
            .as_deref(),
        Some("official")
    );
}

#[test]
fn use_profile_syncs_modified_target_back_to_recorded_profile() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("official", r#"{"env":{"ANTHROPIC_API_KEY":"old"}}"#);
    sandbox.write_profile(
        "deepseek",
        r#"{"env":{"ANTHROPIC_BASE_URL":"https://example.test"}}"#,
    );

    use_profile(&sandbox.paths, "official").unwrap();
    fs::write(
        &sandbox.paths.target_settings_path,
        "{\n  \"env\": {\n    \"ANTHROPIC_API_KEY\": \"edited\"\n  }\n}\n",
    )
    .unwrap();

    use_profile(&sandbox.paths, "deepseek").unwrap();

    assert_eq!(
        fs::read_to_string(sandbox.paths.profile_path("official")).unwrap(),
        "{\n  \"env\": {\n    \"ANTHROPIC_API_KEY\": \"edited\"\n  }\n}\n"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.target_settings_path).unwrap(),
        r#"{"env":{"ANTHROPIC_BASE_URL":"https://example.test"}}"#
    );
    assert_eq!(
        read_current_profile_name(&sandbox.paths)
            .unwrap()
            .as_deref(),
        Some("deepseek")
    );
}

#[test]
fn use_profile_does_not_sync_format_only_json_changes() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("official", r#"{"env":{"A":1,"B":2}}"#);
    sandbox.write_profile("deepseek", r#"{"env":{"A":3}}"#);

    use_profile(&sandbox.paths, "official").unwrap();
    fs::write(
        &sandbox.paths.target_settings_path,
        "{\n  \"env\": {\n    \"B\": 2,\n    \"A\": 1\n  }\n}\n",
    )
    .unwrap();

    use_profile(&sandbox.paths, "deepseek").unwrap();

    assert_eq!(
        fs::read_to_string(sandbox.paths.profile_path("official")).unwrap(),
        r#"{"env":{"A":1,"B":2}}"#
    );
}

#[test]
fn use_profile_aborts_on_invalid_target_json_before_sync_or_overwrite() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("official", r#"{"name":"official"}"#);
    sandbox.write_profile("deepseek", r#"{"name":"deepseek"}"#);

    use_profile(&sandbox.paths, "official").unwrap();
    fs::write(&sandbox.paths.target_settings_path, "{ invalid").unwrap();

    let error = use_profile(&sandbox.paths, "deepseek")
        .unwrap_err()
        .to_string();

    assert!(error.contains("expected") || error.contains("Invalid JSON"));
    assert_eq!(
        fs::read_to_string(sandbox.paths.profile_path("official")).unwrap(),
        r#"{"name":"official"}"#
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.target_settings_path).unwrap(),
        "{ invalid"
    );
    assert_eq!(
        read_current_profile_name(&sandbox.paths)
            .unwrap()
            .as_deref(),
        Some("official")
    );
}

#[test]
fn next_profile_uses_current_record_when_target_was_modified() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("alpha", r#"{"name":"alpha"}"#);
    sandbox.write_profile("beta", r#"{"name":"beta"}"#);

    use_profile(&sandbox.paths, "alpha").unwrap();
    fs::write(
        &sandbox.paths.target_settings_path,
        r#"{"name":"alpha","edited":true}"#,
    )
    .unwrap();

    use_next_profile(&sandbox.paths).unwrap();

    assert_eq!(
        read_current_profile_name(&sandbox.paths)
            .unwrap()
            .as_deref(),
        Some("beta")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.target_settings_path).unwrap(),
        r#"{"name":"beta"}"#
    );
    assert_eq!(
        fs::read_to_string(sandbox.paths.profile_path("alpha")).unwrap(),
        r#"{"name":"alpha","edited":true}"#
    );
}

#[test]
fn next_profile_falls_back_to_content_matching_without_current_record() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("alpha", r#"{"name":"alpha"}"#);
    sandbox.write_profile("beta", r#"{"name":"beta"}"#);
    fs::write(&sandbox.paths.target_settings_path, r#"{"name":"alpha"}"#).unwrap();

    use_next_profile(&sandbox.paths).unwrap();

    assert_eq!(
        read_current_profile_name(&sandbox.paths)
            .unwrap()
            .as_deref(),
        Some("beta")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.target_settings_path).unwrap(),
        r#"{"name":"beta"}"#
    );
}

#[test]
fn before_profile_switches_to_previous_profile_and_toggles() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("alpha", r#"{"name":"alpha"}"#);
    sandbox.write_profile("beta", r#"{"name":"beta"}"#);

    use_profile(&sandbox.paths, "alpha").unwrap();
    use_profile(&sandbox.paths, "beta").unwrap();

    assert_eq!(
        read_before_profile_name(&sandbox.paths).unwrap().as_deref(),
        Some("alpha")
    );

    use_before_profile(&sandbox.paths).unwrap();

    assert_eq!(
        read_current_profile_name(&sandbox.paths)
            .unwrap()
            .as_deref(),
        Some("alpha")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.target_settings_path).unwrap(),
        r#"{"name":"alpha"}"#
    );
    assert_eq!(
        read_before_profile_name(&sandbox.paths).unwrap().as_deref(),
        Some("beta")
    );

    use_before_profile(&sandbox.paths).unwrap();

    assert_eq!(
        read_current_profile_name(&sandbox.paths)
            .unwrap()
            .as_deref(),
        Some("beta")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.target_settings_path).unwrap(),
        r#"{"name":"beta"}"#
    );
    assert_eq!(
        read_before_profile_name(&sandbox.paths).unwrap().as_deref(),
        Some("alpha")
    );
}

#[test]
fn before_profile_without_history_skips_without_changing_target() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("alpha", r#"{"name":"alpha"}"#);
    fs::write(&sandbox.paths.target_settings_path, r#"{"name":"active"}"#).unwrap();

    use_before_profile(&sandbox.paths).unwrap();

    assert_eq!(
        fs::read_to_string(&sandbox.paths.target_settings_path).unwrap(),
        r#"{"name":"active"}"#
    );
    assert_eq!(read_before_profile_name(&sandbox.paths).unwrap(), None);
}

#[test]
fn before_profile_with_deleted_history_profile_skips_without_error() {
    let sandbox = Sandbox::new();
    sandbox.write_profile("alpha", r#"{"name":"alpha"}"#);
    sandbox.write_profile("beta", r#"{"name":"beta"}"#);

    use_profile(&sandbox.paths, "alpha").unwrap();
    use_profile(&sandbox.paths, "beta").unwrap();
    fs::remove_file(sandbox.paths.profile_path("alpha")).unwrap();

    use_before_profile(&sandbox.paths).unwrap();

    assert_eq!(
        read_current_profile_name(&sandbox.paths)
            .unwrap()
            .as_deref(),
        Some("beta")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.target_settings_path).unwrap(),
        r#"{"name":"beta"}"#
    );
    assert_eq!(read_before_profile_name(&sandbox.paths).unwrap(), None);
    assert!(!sandbox.paths.before_path.is_file());
}

struct Sandbox {
    _temp_dir: TempDir,
    paths: ResolvedPaths,
}

impl Sandbox {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".cc-switch-simple");
        let profiles_dir = config_dir.join("profiles");
        let backups_dir = config_dir.join("backups");
        let codex_root = temp_dir.path().join(".cc-switch-simple");
        let codex_profiles_dir = codex_root.join("codex");
        let codex_backups_dir = codex_root.join("backups").join("codex");
        let target_settings_path = temp_dir.path().join(".claude").join("settings.json");
        let codex_target_dir = temp_dir.path().join(".codex");
        let codex_target_config_path = codex_target_dir.join("config.toml");
        let codex_target_auth_path = codex_target_dir.join("auth.json");
        let codex_target_models_catalog_path = codex_target_dir.join("models_catalog.json");

        fs::create_dir_all(&profiles_dir).unwrap();
        fs::create_dir_all(&backups_dir).unwrap();
        fs::create_dir_all(&codex_profiles_dir).unwrap();
        fs::create_dir_all(&codex_backups_dir).unwrap();
        fs::create_dir_all(target_settings_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&codex_target_dir).unwrap();

        Self {
            paths: ResolvedPaths {
                config_dir: config_dir.clone(),
                config_file_path: config_dir.join("config.toml"),
                profiles_dir,
                current_path: config_dir.join("current"),
                before_path: config_dir.join("before"),
                backups_dir,
                target_settings_path,
                codex_profiles_dir: codex_profiles_dir.clone(),
                codex_current_path: codex_profiles_dir.join("current"),
                codex_before_path: codex_profiles_dir.join("before"),
                codex_backups_dir,
                codex_target_config_path,
                codex_target_auth_path,
                codex_target_models_catalog_path,
                max_backup_files: 5,
            },
            _temp_dir: temp_dir,
        }
    }

    fn write_profile(&self, name: &str, json: &str) {
        fs::write(self.paths.profiles_dir.join(format!("{name}.json")), json).unwrap();
    }
}
