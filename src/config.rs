use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub version: String,
    pub fzf: FzfConfig,
    pub auto_discovery: AutoDiscoveryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FzfConfig {
    pub height: String,
    pub layout: String,
    pub preview_window: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoDiscoveryConfig {
    pub enabled: bool,
    pub paths: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            fzf: FzfConfig::default(),
            auto_discovery: AutoDiscoveryConfig::default(),
        }
    }
}

impl Default for FzfConfig {
    fn default() -> Self {
        Self {
            height: "40%".to_string(),
            layout: "reverse".to_string(),
            preview_window: "right:60%".to_string(),
        }
    }
}

impl Default for AutoDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            paths: Vec::new(),
        }
    }
}

/// Returns the config directory: `~/.config/worktree-manager`
pub fn config_dir() -> PathBuf {
    let base = directories::BaseDirs::new()
        .expect("failed to determine home directory")
        .config_dir()
        .to_path_buf();
    base.join("worktree-manager")
}

/// Returns the config file path: `~/.config/worktree-manager/config.yaml`
pub fn config_path() -> PathBuf {
    config_dir().join("config.yaml")
}

/// Loads config from disk. Returns default config if file doesn't exist.
pub fn load() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;

    let config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse config file: {}", path.display()))?;

    Ok(config)
}

/// Saves config to disk. Creates parent directories if needed.
pub fn save(config: &Config) -> Result<()> {
    let path = config_path();
    let dir = config_dir();

    // Create parent directory if it doesn't exist
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create config directory: {}", dir.display()))?;
    }

    let content = serde_yaml::to_string(config).context("failed to serialize config to YAML")?;

    fs::write(&path, content)
        .with_context(|| format!("failed to write config file: {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = Config::default();
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.fzf.height, "40%");
        assert_eq!(config.fzf.layout, "reverse");
        assert_eq!(config.fzf.preview_window, "right:60%");
        assert!(config.auto_discovery.enabled);
        assert!(config.auto_discovery.paths.is_empty());
    }

    #[test]
    fn config_serializes_to_yaml() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("version:"));
        assert!(yaml.contains("fzf:"));
        assert!(yaml.contains("auto_discovery:"));
    }

    #[test]
    fn config_deserializes_from_yaml() {
        let yaml = r#"
version: "1.0.0"
fzf:
  height: "50%"
  layout: reverse
  preview_window: "right:70%"
auto_discovery:
  enabled: false
  paths:
    - /home/user/projects
    - /home/user/work
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.fzf.height, "50%");
        assert_eq!(config.fzf.preview_window, "right:70%");
        assert!(!config.auto_discovery.enabled);
        assert_eq!(config.auto_discovery.paths.len(), 2);
    }

    #[test]
    fn config_dir_returns_path() {
        let dir = config_dir();
        assert!(dir.to_string_lossy().contains("worktree-manager"));
    }

    #[test]
    fn config_path_returns_yaml_file() {
        let path = config_path();
        assert!(path.to_string_lossy().ends_with("config.yaml"));
    }

    #[test]
    fn load_returns_default_when_file_missing() {
        // Test that load() succeeds whether config exists or not
        let config = load().unwrap();
        assert_eq!(config.version, "1.0.0");
        // Don't assert on paths - user may have configured them
    }
}
