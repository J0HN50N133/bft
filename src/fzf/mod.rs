use thiserror::Error;
use fzf_wrapped::{Layout, Border};

#[derive(Error, Debug)]
pub enum FzfError {
    #[error("FZF execution failed: {0}")]
    ExecutionError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("FZF wrapper error: {0}")]
    WrapperError(#[from] fzf_wrapped::FzfBuilderError),
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub struct FzfConfig {
    pub height: String,
    pub prompt: String,
    pub layout: Layout,
    pub border: Border,
    pub options: Vec<String>,
}

impl Default for FzfConfig {
    fn default() -> Self {
        Self {
            height: "40%".to_string(),
            prompt: "> ".to_string(),
            layout: Layout::Default,
            border: Border::None,
            options: Vec::new(),
        }
    }
}
