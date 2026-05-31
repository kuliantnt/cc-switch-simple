use std::fs;

use cc_switch::{
    ResolvedPaths, backup_file_name, collect_profiles, detect_current_profile_index,
    next_profile_index,
};
use tempfile::TempDir;
use time::macros::datetime;

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
        let target_settings_path = temp_dir.path().join(".claude").join("settings.json");

        fs::create_dir_all(&profiles_dir).unwrap();
        fs::create_dir_all(&backups_dir).unwrap();
        fs::create_dir_all(target_settings_path.parent().unwrap()).unwrap();

        Self {
            paths: ResolvedPaths {
                config_dir: config_dir.clone(),
                config_file_path: config_dir.join("config.toml"),
                profiles_dir,
                backups_dir,
                target_settings_path,
            },
            _temp_dir: temp_dir,
        }
    }

    fn write_profile(&self, name: &str, json: &str) {
        fs::write(self.paths.profiles_dir.join(format!("{name}.json")), json).unwrap();
    }
}
