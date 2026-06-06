use std::fs;

use cc_switch::{
    ResolvedPaths, collect_codex_profiles, read_codex_before_name, read_codex_current_name,
    use_before_codex_profile, use_codex_profile, use_next_codex_profile,
};
use tempfile::TempDir;

#[test]
fn collect_codex_profiles_returns_sorted_names() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("zeta", "model = \"zeta\"\n", "{\"token\":\"zeta\"}");
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"openai\"}");
    sandbox.write_codex_profile("alpha", "model = \"alpha\"\n", "{\"token\":\"alpha\"}");
    fs::create_dir_all(sandbox.paths.codex_profiles_dir.join("missing-config")).unwrap();
    let missing_auth_dir = sandbox.paths.codex_profiles_dir.join("missing-auth");
    fs::create_dir_all(&missing_auth_dir).unwrap();
    fs::write(
        missing_auth_dir.join("config.toml"),
        "model = \"missing\"\n",
    )
    .unwrap();
    fs::write(
        sandbox.paths.codex_profiles_dir.join("notes.txt"),
        "ignore me",
    )
    .unwrap();

    let profiles = collect_codex_profiles(&sandbox.paths).unwrap();
    let names = profiles
        .into_iter()
        .map(|profile| profile.name)
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["alpha", "openai", "zeta"]);
}

#[test]
fn use_codex_profile_copies_config_and_records_current() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"next\"}");
    fs::write(
        &sandbox.paths.codex_target_config_path,
        "model = \"old\"\nprovider = \"legacy\"\n",
    )
    .unwrap();
    fs::write(
        &sandbox.paths.codex_target_auth_path,
        "{\"token\":\"secret\"}",
    )
    .unwrap();

    use_codex_profile(&sandbox.paths, "openai").unwrap();

    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_config_path).unwrap(),
        "model = \"gpt-5\"\n"
    );
    assert_eq!(
        read_codex_current_name(&sandbox.paths).unwrap().as_deref(),
        Some("openai")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{\"token\":\"next\"}"
    );

    let mut backup_names = fs::read_dir(&sandbox.paths.codex_backups_dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    backup_names.sort();
    assert_eq!(backup_names.len(), 2);
    assert!(
        backup_names
            .iter()
            .any(|name| name.starts_with("config.toml.") && name.ends_with(".bak"))
    );
    assert!(
        backup_names
            .iter()
            .any(|name| name.starts_with("auth.json.") && name.ends_with(".bak"))
    );
}

#[test]
fn use_next_codex_profile_uses_current_record_and_wraps() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"openai\"}");
    sandbox.write_codex_profile("xxxcom", "model = \"mirror\"\n", "{\"token\":\"xxx\"}");
    fs::write(&sandbox.paths.codex_current_path, "openai").unwrap();
    fs::write(
        &sandbox.paths.codex_target_config_path,
        "model = \"gpt-5\"\n",
    )
    .unwrap();
    fs::write(&sandbox.paths.codex_target_auth_path, "{\"token\":\"old\"}").unwrap();

    use_next_codex_profile(&sandbox.paths).unwrap();
    assert_eq!(
        read_codex_current_name(&sandbox.paths).unwrap().as_deref(),
        Some("xxxcom")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_config_path).unwrap(),
        "model = \"mirror\"\n"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{\"token\":\"xxx\"}"
    );

    use_next_codex_profile(&sandbox.paths).unwrap();
    assert_eq!(
        read_codex_current_name(&sandbox.paths).unwrap().as_deref(),
        Some("openai")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_config_path).unwrap(),
        "model = \"gpt-5\"\n"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{\"token\":\"old\"}"
    );
}

#[test]
fn use_next_codex_profile_falls_back_to_first_when_current_is_unknown() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"openai\"}");
    sandbox.write_codex_profile("xxxcom", "model = \"mirror\"\n", "{\"token\":\"xxx\"}");
    fs::write(&sandbox.paths.codex_current_path, "missing").unwrap();

    use_next_codex_profile(&sandbox.paths).unwrap();

    assert_eq!(
        read_codex_current_name(&sandbox.paths).unwrap().as_deref(),
        Some("openai")
    );
}

#[test]
fn use_codex_profile_prunes_backups_per_codex_target_file() {
    let mut sandbox = Sandbox::new();
    sandbox.paths.max_backup_files = 2;
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"next\"}");
    fs::write(&sandbox.paths.codex_target_config_path, "model = \"old\"\n").unwrap();
    fs::write(&sandbox.paths.codex_target_auth_path, "{\"token\":\"old\"}").unwrap();

    for name in [
        "config.toml.20260531-134500.bak",
        "config.toml.20260531-134501.bak",
        "auth.json.20260531-134500.bak",
        "auth.json.20260531-134501.bak",
    ] {
        fs::write(sandbox.paths.codex_backups_dir.join(name), "old").unwrap();
    }

    use_codex_profile(&sandbox.paths, "openai").unwrap();

    let mut backup_names = fs::read_dir(&sandbox.paths.codex_backups_dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    backup_names.sort();

    assert_eq!(backup_names.len(), 4);
    assert!(!backup_names.contains(&"config.toml.20260531-134500.bak".to_string()));
    assert!(!backup_names.contains(&"auth.json.20260531-134500.bak".to_string()));
    assert!(backup_names.contains(&"config.toml.20260531-134501.bak".to_string()));
    assert!(backup_names.contains(&"auth.json.20260531-134501.bak".to_string()));
    assert!(
        backup_names
            .iter()
            .any(|name| name.starts_with("config.toml.") && name.ends_with(".bak"))
    );
    assert!(
        backup_names
            .iter()
            .any(|name| name.starts_with("auth.json.") && name.ends_with(".bak"))
    );
}

#[test]
fn use_codex_profile_requires_auth_json_in_profile() {
    let sandbox = Sandbox::new();
    let profile_dir = sandbox.paths.codex_profiles_dir.join("openai");
    fs::create_dir_all(&profile_dir).unwrap();
    fs::write(profile_dir.join("config.toml"), "model = \"gpt-5\"\n").unwrap();

    let error = use_codex_profile(&sandbox.paths, "openai")
        .unwrap_err()
        .to_string();
    assert!(error.contains("Codex profile auth not found"));
}

#[test]
fn use_codex_profile_syncs_modified_active_files_back_to_current_profile() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"openai\"}");
    sandbox.write_codex_profile("xxxcom", "model = \"mirror\"\n", "{\"token\":\"xxx\"}");

    use_codex_profile(&sandbox.paths, "openai").unwrap();
    fs::write(
        &sandbox.paths.codex_target_config_path,
        "# local edit\nmodel = \"gpt-5\"\n",
    )
    .unwrap();
    fs::write(
        &sandbox.paths.codex_target_auth_path,
        "{\n  \"token\": \"edited\"\n}\n",
    )
    .unwrap();

    use_codex_profile(&sandbox.paths, "xxxcom").unwrap();

    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_profile_path("openai")).unwrap(),
        "# local edit\nmodel = \"gpt-5\"\n"
    );
    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_auth_path("openai")).unwrap(),
        "{\n  \"token\": \"edited\"\n}\n"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_config_path).unwrap(),
        "model = \"mirror\"\n"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{\"token\":\"xxx\"}"
    );
}

#[test]
fn use_codex_profile_syncs_only_changed_config_file() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"openai\"}");
    sandbox.write_codex_profile("xxxcom", "model = \"mirror\"\n", "{\"token\":\"xxx\"}");

    use_codex_profile(&sandbox.paths, "openai").unwrap();
    fs::write(
        &sandbox.paths.codex_target_config_path,
        "model = \"gpt-5\"\nprovider = \"local\"\n",
    )
    .unwrap();

    use_codex_profile(&sandbox.paths, "xxxcom").unwrap();

    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_profile_path("openai")).unwrap(),
        "model = \"gpt-5\"\nprovider = \"local\"\n"
    );
    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_auth_path("openai")).unwrap(),
        "{\"token\":\"openai\"}"
    );
}

#[test]
fn use_codex_profile_syncs_only_changed_auth_file() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"openai\"}");
    sandbox.write_codex_profile("xxxcom", "model = \"mirror\"\n", "{\"token\":\"xxx\"}");

    use_codex_profile(&sandbox.paths, "openai").unwrap();
    fs::write(
        &sandbox.paths.codex_target_auth_path,
        "{\"token\":\"edited\"}",
    )
    .unwrap();

    use_codex_profile(&sandbox.paths, "xxxcom").unwrap();

    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_profile_path("openai")).unwrap(),
        "model = \"gpt-5\"\n"
    );
    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_auth_path("openai")).unwrap(),
        "{\"token\":\"edited\"}"
    );
}

#[test]
fn use_codex_profile_skips_sync_when_current_record_is_missing_or_invalid() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"openai\"}");
    sandbox.write_codex_profile("xxxcom", "model = \"mirror\"\n", "{\"token\":\"xxx\"}");
    fs::write(&sandbox.paths.codex_current_path, "missing").unwrap();
    fs::write(
        &sandbox.paths.codex_target_config_path,
        "model = \"locally edited\"\n",
    )
    .unwrap();
    fs::write(
        &sandbox.paths.codex_target_auth_path,
        "{\"token\":\"edited\"}",
    )
    .unwrap();

    use_codex_profile(&sandbox.paths, "xxxcom").unwrap();

    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_profile_path("openai")).unwrap(),
        "model = \"gpt-5\"\n"
    );
    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_auth_path("openai")).unwrap(),
        "{\"token\":\"openai\"}"
    );
    assert_eq!(
        read_codex_current_name(&sandbox.paths).unwrap().as_deref(),
        Some("xxxcom")
    );
}

#[test]
fn use_codex_profile_aborts_when_active_auth_json_is_invalid() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"openai\"}");
    sandbox.write_codex_profile("xxxcom", "model = \"mirror\"\n", "{\"token\":\"xxx\"}");

    use_codex_profile(&sandbox.paths, "openai").unwrap();
    fs::write(&sandbox.paths.codex_target_auth_path, "{ invalid").unwrap();

    let error = use_codex_profile(&sandbox.paths, "xxxcom")
        .unwrap_err()
        .to_string();

    assert!(error.contains("Invalid JSON") || error.contains("expected"));
    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_auth_path("openai")).unwrap(),
        "{\"token\":\"openai\"}"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{ invalid"
    );
    assert_eq!(
        read_codex_current_name(&sandbox.paths).unwrap().as_deref(),
        Some("openai")
    );
}

#[test]
fn use_codex_profile_skips_missing_active_file_during_sync_back() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"openai\"}");
    sandbox.write_codex_profile("xxxcom", "model = \"mirror\"\n", "{\"token\":\"xxx\"}");

    use_codex_profile(&sandbox.paths, "openai").unwrap();
    fs::remove_file(&sandbox.paths.codex_target_auth_path).unwrap();
    fs::write(
        &sandbox.paths.codex_target_config_path,
        "model = \"edited\"\n",
    )
    .unwrap();

    use_codex_profile(&sandbox.paths, "xxxcom").unwrap();

    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_profile_path("openai")).unwrap(),
        "model = \"edited\"\n"
    );
    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_auth_path("openai")).unwrap(),
        "{\"token\":\"openai\"}"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{\"token\":\"xxx\"}"
    );
}

#[test]
fn use_before_codex_profile_switches_to_previous_profile_and_toggles() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("openai", "model = \"gpt-5\"\n", "{\"token\":\"openai\"}");
    sandbox.write_codex_profile("xxxcom", "model = \"mirror\"\n", "{\"token\":\"xxx\"}");

    use_codex_profile(&sandbox.paths, "openai").unwrap();
    use_codex_profile(&sandbox.paths, "xxxcom").unwrap();

    assert_eq!(
        read_codex_before_name(&sandbox.paths).unwrap().as_deref(),
        Some("openai")
    );

    use_before_codex_profile(&sandbox.paths).unwrap();

    assert_eq!(
        read_codex_current_name(&sandbox.paths).unwrap().as_deref(),
        Some("openai")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_config_path).unwrap(),
        "model = \"gpt-5\"\n"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{\"token\":\"openai\"}"
    );
    assert_eq!(
        read_codex_before_name(&sandbox.paths).unwrap().as_deref(),
        Some("xxxcom")
    );

    use_before_codex_profile(&sandbox.paths).unwrap();

    assert_eq!(
        read_codex_current_name(&sandbox.paths).unwrap().as_deref(),
        Some("xxxcom")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_config_path).unwrap(),
        "model = \"mirror\"\n"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{\"token\":\"xxx\"}"
    );
    assert_eq!(
        read_codex_before_name(&sandbox.paths).unwrap().as_deref(),
        Some("openai")
    );
}

#[test]
fn use_before_codex_profile_without_history_skips_without_changing_targets() {
    let sandbox = Sandbox::new();
    fs::write(
        &sandbox.paths.codex_target_config_path,
        "model = \"active\"\n",
    )
    .unwrap();
    fs::write(
        &sandbox.paths.codex_target_auth_path,
        "{\"token\":\"active\"}",
    )
    .unwrap();

    use_before_codex_profile(&sandbox.paths).unwrap();

    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_config_path).unwrap(),
        "model = \"active\"\n"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{\"token\":\"active\"}"
    );
    assert_eq!(read_codex_before_name(&sandbox.paths).unwrap(), None);
}

#[test]
fn use_before_codex_profile_with_incomplete_history_profile_skips_without_error() {
    let sandbox = Sandbox::new();
    let partial_dir = sandbox.paths.codex_profiles_dir.join("partial");
    fs::create_dir_all(&partial_dir).unwrap();
    fs::write(partial_dir.join("config.toml"), "model = \"partial\"\n").unwrap();
    fs::write(&sandbox.paths.codex_before_path, "partial").unwrap();
    fs::write(
        &sandbox.paths.codex_target_config_path,
        "model = \"active\"\n",
    )
    .unwrap();
    fs::write(
        &sandbox.paths.codex_target_auth_path,
        "{\"token\":\"active\"}",
    )
    .unwrap();

    use_before_codex_profile(&sandbox.paths).unwrap();

    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_config_path).unwrap(),
        "model = \"active\"\n"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{\"token\":\"active\"}"
    );
    assert_eq!(read_codex_before_name(&sandbox.paths).unwrap(), None);
    assert!(!sandbox.paths.codex_before_path.is_file());
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
                max_backup_files: 5,
            },
            _temp_dir: temp_dir,
        }
    }

    fn write_codex_profile(&self, name: &str, content: &str, auth: &str) {
        let profile_dir = self.paths.codex_profiles_dir.join(name);
        fs::create_dir_all(&profile_dir).unwrap();
        fs::write(profile_dir.join("config.toml"), content).unwrap();
        fs::write(profile_dir.join("auth.json"), auth).unwrap();
    }
}
