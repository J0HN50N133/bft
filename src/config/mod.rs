use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorType {
    Dialoguer,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub fzf_tmux_height: Option<String>,
    pub fzf_default_opts: String,
    pub fzf_completion_opts: String,
    pub auto_common_prefix: bool,
    pub auto_common_prefix_part: bool,
    pub prompt: String,
    pub completion_sep: String,
    pub no_empty_cmd_completion: bool,
    pub selector_type: SelectorType,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fzf_tmux_height: Some("40%".to_string()),
            fzf_default_opts: String::new(),
            fzf_completion_opts: String::new(),
            auto_common_prefix: true,
            auto_common_prefix_part: false,
            prompt: "> ".to_string(),
            completion_sep: "\x01".to_string(),
            no_empty_cmd_completion: false,
            selector_type: SelectorType::Dialoguer,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let fzf_tmux_height = env::var("FZF_TMUX_HEIGHT").ok();
        let fzf_default_opts = env::var("FZF_DEFAULT_OPTS").unwrap_or_default();
        let fzf_completion_opts = env::var("FZF_COMPLETION_OPTS").unwrap_or_default();

        let auto_common_prefix = env::var("FZF_COMPLETION_AUTO_COMMON_PREFIX")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true); // Default to true as per common behavior, though bash script checks explicit "true"

        let auto_common_prefix_part = env::var("FZF_COMPLETION_AUTO_COMMON_PREFIX_PART")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let prompt = env::var("FZF_TAB_COMPLETION_PROMPT").unwrap_or_else(|_| "> ".to_string());

        // _FZF_COMPLETION_SEP=$'\x01' in bash script
        let completion_sep = env::var("_FZF_COMPLETION_SEP").unwrap_or_else(|_| "\x01".to_string());

        // shopt -q no_empty_cmd_completion
        // In Rust we can't check shopt directly easily, so we rely on an env var passing it through
        // or just a config flag. For now, let's assume it might be passed or default false.
        // We'll add an env var for it to allow configuration.
        let no_empty_cmd_completion = env::var("FZF_NO_EMPTY_CMD_COMPLETION")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let selector_type = env::var("FZF_TAB_COMPLETION_SELECTOR")
            .map(|v| match v.to_lowercase().as_str() {
                "dialoguer" => SelectorType::Dialoguer,
                _ => SelectorType::Dialoguer, // Default to dialoguer
            })
            .unwrap_or(SelectorType::Dialoguer);

        Self {
            fzf_tmux_height,
            fzf_default_opts,
            fzf_completion_opts,
            auto_common_prefix,
            auto_common_prefix_part,
            prompt,
            completion_sep,
            no_empty_cmd_completion,
            selector_type,
        }
    }
}
