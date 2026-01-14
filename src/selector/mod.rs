use std::rc::Rc;

use thiserror::Error;

use crate::completion::CompletionContext;

#[derive(Error, Debug)]
pub enum SelectorError {
    #[error("Selector execution error: {0}")]
    ExecutionError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Cancelled")]
    Cancelled,
    #[error("No candidates")]
    NoCandidates,
    #[error("Error setting Ctrl-C handler")]
    SettingCtrlCHandler(#[from] ctrlc::Error),
}

#[derive(Debug, Clone)]
pub struct SelectorConfig {
    pub ctx: Rc<CompletionContext>,
    pub prompt: String,
    pub height: String,
    pub header: Option<String>,
}

pub trait Selector {
    fn select_one(
        &self,
        candidates: &[String],
        current_word: &str,
        config: &SelectorConfig,
    ) -> Result<Option<String>, SelectorError>;
}

// Re-export implementations
pub mod dialoguer;

