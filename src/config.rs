use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub claude: ClaudeConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClaudeConfig {
    pub settings_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub target_settings_path: Option<String>,
}

impl From<ConfigFile> for AppConfig {
    fn from(value: ConfigFile) -> Self {
        Self {
            target_settings_path: value.claude.settings_path,
        }
    }
}
