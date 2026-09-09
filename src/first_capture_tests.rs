//! 首次收纳 optional Codex 模型 JSON 的内部单测。
//!
//! 因依赖仅测试可见的 `set_sync_back_override` 钩子，
//! 本模块以 `#[cfg(test)]` 编译，不进入正式 library API。

use std::fs;

use tempfile::TempDir;

use crate::{ResolvedPaths, read_codex_current_name, set_sync_back_override, use_codex_profile};

struct Sandbox {
    _temp_dir: TempDir,
    paths: ResolvedPaths,
}

impl Sandbox {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".cc-switch-simple");
        let codex_profiles_dir = config_dir.join("codex");
        let codex_backups_dir = config_dir.join("backups").join("codex");
        let codex_target_dir = temp_dir.path().join(".codex");

        fs::create_dir_all(&codex_profiles_dir).unwrap();
        fs::create_dir_all(&codex_backups_dir).unwrap();
        fs::create_dir_all(&codex_target_dir).unwrap();

        Self {
            paths: ResolvedPaths {
                config_dir: config_dir.clone(),
                config_file_path: config_dir.join("config.toml"),
                profiles_dir: config_dir.join("profiles"),
                current_path: config_dir.join("current"),
                before_path: config_dir.join("before"),
                backups_dir: config_dir.join("backups"),
                target_settings_path: temp_dir.path().join(".claude").join("settings.json"),
                codex_profiles_dir: codex_profiles_dir.clone(),
                codex_current_path: codex_profiles_dir.join("current"),
                codex_before_path: codex_profiles_dir.join("before"),
                codex_backups_dir,
                codex_target_config_path: codex_target_dir.join("config.toml"),
                codex_target_auth_path: codex_target_dir.join("auth.json"),
                codex_target_models_catalog_path: codex_target_dir.join("models_catalog.json"),
                codex_target_models_path: codex_target_dir.join("models.json"),
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

fn backup_names(sandbox: &Sandbox) -> Vec<String> {
    let mut names = fs::read_dir(&sandbox.paths.codex_backups_dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    names.sort();
    names
}

#[test]
fn first_capture_models_json_saved_then_restored_on_switch_back() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("old", "model = \"old\"\n", "{\"token\":\"old\"}");
    sandbox.write_codex_profile("new", "model = \"new\"\n", "{\"token\":\"new\"}");
    let active = r#"{"models":[{"slug":"active-model"}]}"#;
    fs::write(&sandbox.paths.codex_target_models_path, active).unwrap();
    fs::write(&sandbox.paths.codex_current_path, "old").unwrap();
    fs::write(&sandbox.paths.codex_target_config_path, "model = \"old\"\n").unwrap();
    fs::write(&sandbox.paths.codex_target_auth_path, "{\"token\":\"old\"}").unwrap();

    set_sync_back_override(Some(true));
    use_codex_profile(&sandbox.paths, "new").unwrap();
    set_sync_back_override(None);

    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_models_path("old")).unwrap(),
        active
    );

    set_sync_back_override(Some(false));
    use_codex_profile(&sandbox.paths, "old").unwrap();
    set_sync_back_override(None);

    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_models_path).unwrap(),
        active
    );
}

#[test]
fn first_capture_models_catalog_saved_then_restored_on_switch_back() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("old", "model = \"old\"\n", "{\"token\":\"old\"}");
    sandbox.write_codex_profile("new", "model = \"new\"\n", "{\"token\":\"new\"}");
    let active = r#"{"models":[{"slug":"active-catalog"}]}"#;
    fs::write(&sandbox.paths.codex_target_models_catalog_path, active).unwrap();
    fs::write(&sandbox.paths.codex_current_path, "old").unwrap();
    fs::write(&sandbox.paths.codex_target_config_path, "model = \"old\"\n").unwrap();
    fs::write(&sandbox.paths.codex_target_auth_path, "{\"token\":\"old\"}").unwrap();

    set_sync_back_override(Some(true));
    use_codex_profile(&sandbox.paths, "new").unwrap();
    set_sync_back_override(None);

    assert_eq!(
        fs::read_to_string(sandbox.paths.codex_models_catalog_path("old")).unwrap(),
        active
    );

    set_sync_back_override(Some(false));
    use_codex_profile(&sandbox.paths, "old").unwrap();
    set_sync_back_override(None);

    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_models_catalog_path).unwrap(),
        active
    );
}

#[test]
fn first_capture_declined_keeps_profile_empty_and_switch_completes() {
    let sandbox = Sandbox::new();
    sandbox.write_codex_profile("old", "model = \"old\"\n", "{\"token\":\"old\"}");
    sandbox.write_codex_profile("new", "model = \"new\"\n", "{\"token\":\"new\"}");
    let active = r#"{"models":[{"slug":"active-model"}]}"#;
    fs::write(&sandbox.paths.codex_target_models_path, active).unwrap();
    fs::write(&sandbox.paths.codex_current_path, "old").unwrap();
    fs::write(&sandbox.paths.codex_target_config_path, "model = \"old\"\n").unwrap();
    fs::write(&sandbox.paths.codex_target_auth_path, "{\"token\":\"old\"}").unwrap();

    set_sync_back_override(Some(false));
    use_codex_profile(&sandbox.paths, "new").unwrap();
    set_sync_back_override(None);

    assert!(!sandbox.paths.codex_models_path("old").exists());
    assert_eq!(
        read_codex_current_name(&sandbox.paths).unwrap().as_deref(),
        Some("new")
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_config_path).unwrap(),
        "model = \"new\"\n"
    );
    assert_eq!(
        fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
        "{\"token\":\"new\"}"
    );
}

#[test]
fn invalid_active_optional_json_blocks_switch_without_partial_state() {
    for which in [0_u8, 1] {
        let sandbox = Sandbox::new();
        sandbox.write_codex_profile("old", "model = \"old\"\n", "{\"token\":\"old\"}");
        sandbox.write_codex_profile("new", "model = \"new\"\n", "{\"token\":\"new\"}");
        fs::write(&sandbox.paths.codex_current_path, "old").unwrap();
        fs::write(&sandbox.paths.codex_target_config_path, "model = \"old\"\n").unwrap();
        fs::write(&sandbox.paths.codex_target_auth_path, "{\"token\":\"old\"}").unwrap();
        let invalid = b"{ invalid json \xff";
        if which == 0 {
            fs::write(&sandbox.paths.codex_target_models_path, invalid).unwrap();
        } else {
            fs::write(&sandbox.paths.codex_target_models_catalog_path, invalid).unwrap();
        }

        let error = use_codex_profile(&sandbox.paths, "new")
            .unwrap_err()
            .to_string();

        assert!(error.contains("Invalid JSON"), "unexpected error: {error}");
        assert!(!sandbox.paths.codex_models_path("old").exists());
        assert!(!sandbox.paths.codex_models_catalog_path("old").exists());
        assert_eq!(
            read_codex_current_name(&sandbox.paths).unwrap().as_deref(),
            Some("old")
        );
        assert_eq!(
            fs::read_to_string(&sandbox.paths.codex_target_config_path).unwrap(),
            "model = \"old\"\n"
        );
        assert_eq!(
            fs::read_to_string(&sandbox.paths.codex_target_auth_path).unwrap(),
            "{\"token\":\"old\"}"
        );
        assert!(backup_names(&sandbox).is_empty());
    }
}
