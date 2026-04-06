use crate::theme::{Theme, ThemeConfig};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(default = "default_autosave_interval")]
    pub autosave_interval_secs: u64,

    #[serde(default)]
    pub lsp: HashMap<String, LspServerConfig>,

    #[serde(default)]
    pub theme: Option<ThemeConfig>,
}

impl Settings {
    pub fn resolve_theme(&self) -> Theme {
        match &self.theme {
            Some(config) => Theme::resolve(config),
            None => Theme::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LspServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

fn default_autosave_interval() -> u64 {
    300 // 5 minutes
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autosave_interval_secs: default_autosave_interval(),
            lsp: HashMap::new(),
            theme: None,
        }
    }
}

impl Settings {
    pub fn load(dir: &Path) -> Self {
        let file_path = dir.join("settings.json");
        match std::fs::read_to_string(&file_path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let s = Settings::default();
        assert_eq!(s.autosave_interval_secs, 300);
        assert!(s.lsp.is_empty());
    }

    #[test]
    fn test_load_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let s = Settings::load(dir.path());
        assert_eq!(s.autosave_interval_secs, 300);
    }

    #[test]
    fn test_load_custom_autosave() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.json"),
            r#"{ "autosave_interval_secs": 60 }"#,
        )
        .unwrap();
        let s = Settings::load(dir.path());
        assert_eq!(s.autosave_interval_secs, 60);
    }

    #[test]
    fn test_load_lsp_config() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.json"),
            r#"{
                "autosave_interval_secs": 300,
                "lsp": {
                    "rs": { "command": "rust-analyzer", "args": [] },
                    "py": { "command": "pylsp", "args": ["--check"] }
                }
            }"#,
        )
        .unwrap();
        let s = Settings::load(dir.path());
        assert_eq!(s.lsp.len(), 2);
        assert_eq!(s.lsp["rs"].command, "rust-analyzer");
        assert_eq!(s.lsp["py"].args, vec!["--check"]);
    }

    #[test]
    fn test_load_malformed_json() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("settings.json"), "not json").unwrap();
        let s = Settings::load(dir.path());
        assert_eq!(s.autosave_interval_secs, 300); // falls back to default
    }

    #[test]
    fn test_load_zero_autosave_disables() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("settings.json"),
            r#"{ "autosave_interval_secs": 0 }"#,
        )
        .unwrap();
        let s = Settings::load(dir.path());
        assert_eq!(s.autosave_interval_secs, 0);
    }
}
