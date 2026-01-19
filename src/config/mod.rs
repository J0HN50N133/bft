use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SelectorType {
    #[default]
    Dialoguer,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderConfig {
    History { limit: Option<usize> },
    Carapace,
    Bash,
    EnvVar,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub selector_height: Option<String>,
    pub auto_common_prefix: bool,
    pub auto_common_prefix_part: bool,
    pub prompt: String,
    #[serde(skip, default = "default_completion_sep")]
    pub completion_sep: String,
    pub no_empty_cmd_completion: bool,
    pub selector_type: SelectorType,
    pub providers: Vec<ProviderConfig>,
}

fn default_completion_sep() -> String {
    "\x01".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            selector_height: Some("40%".to_string()),
            auto_common_prefix: true,
            auto_common_prefix_part: false,
            prompt: "> ".to_string(),
            completion_sep: default_completion_sep(),
            no_empty_cmd_completion: false,
            selector_type: SelectorType::Dialoguer,
            providers: vec![
                ProviderConfig::Bash,
                ProviderConfig::History { limit: Some(20) },
                ProviderConfig::Carapace,
                ProviderConfig::EnvVar,
            ],
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(config) = Self::from_file() {
            return config;
        }
        Self::from_env()
    }

    fn from_file() -> Option<Self> {
        let xdg_config_home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{}/.config", home)
        });

        let config_path = PathBuf::from(xdg_config_home).join("bft/config.json5");
        if config_path.exists()
            && let Ok(content) = fs::read_to_string(&config_path)
        {
            match json5::from_str(&content) {
                Ok(config) => return Some(config),
                Err(e) => {
                    log::error!("Failed to parse config file: {}", e);
                }
            }
        }
        None
    }

    pub fn from_env() -> Self {
        let selector_height = env::var("BFT_SELECTOR_HEIGHT").ok();

        let auto_common_prefix = env::var("BFT_AUTO_COMMON_PREFIX")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        let auto_common_prefix_part = env::var("BFT_AUTO_COMMON_PREFIX_PART")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let prompt = env::var("BFT_PROMPT").unwrap_or_else(|_| "> ".to_string());

        let no_empty_cmd_completion = env::var("BFT_NO_EMPTY_CMD_COMPLETION")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let selector_type = env::var("BFT_SELECTOR")
            .map(|v| match v.to_lowercase().as_str() {
                "dialoguer" => SelectorType::Dialoguer,
                _ => SelectorType::Dialoguer,
            })
            .unwrap_or(SelectorType::Dialoguer);

        Self {
            selector_height,
            auto_common_prefix,
            auto_common_prefix_part,
            prompt,
            completion_sep: default_completion_sep(),
            no_empty_cmd_completion,
            selector_type,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_empty_config() {
        let json = "{}";
        let config: Config = json5::from_str(json).unwrap();
        assert_eq!(config.prompt, "> ");
        assert_eq!(config.providers.len(), 4);
    }

    #[test]
    fn test_deserialize_partial_config() {
        let json = "{ prompt: '$ ' }";
        let config: Config = json5::from_str(json).unwrap();
        assert_eq!(config.prompt, "$ ");
        assert!(config.auto_common_prefix); // default
        assert_eq!(config.providers.len(), 4); // default
    }

    #[test]
    fn test_deserialize_providers_override() {
        let json = "{ providers: [{ type: 'bash' }] }";
        let config: Config = json5::from_str(json).unwrap();
        assert_eq!(config.providers.len(), 1);
        match config.providers[0] {
            ProviderConfig::Bash => {}
            _ => panic!("Expected Bash provider"),
        }
    }
}
