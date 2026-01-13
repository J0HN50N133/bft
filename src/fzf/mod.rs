use thiserror::Error;
use fzf_wrapped::{FzfBuilder, Border, Layout};

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
    pub completion_sep: String,
    pub options: Vec<String>,
}

impl Default for FzfConfig {
    fn default() -> Self {
        Self {
            height: "40%".to_string(),
            prompt: "> ".to_string(),
            layout: Layout::Default,
            border: Border::None,
            completion_sep: "\x01".to_string(),
            options: Vec::new(),
        }
    }
}

pub fn select_with_fzf(candidates: &[String], current_word: &str, config: &FzfConfig) -> Result<Option<String>, FzfError> {
    if candidates.is_empty() {
        return Ok(None);
    }

    let mut formatted_candidates = Vec::with_capacity(candidates.len());
    let sep = &config.completion_sep;
    let len = current_word.len();

    for cand in candidates {
        let (prefix, suffix) = if len <= cand.len() {
            cand.split_at(len)
        } else {
            (cand.as_str(), "")
        };

        let formatted = format!(
            "{}{}{}{}{}{}{}",
            cand,
            sep,
            "\x1b[37m", prefix, "\x1b[0m",
            sep,
            suffix
        );
        formatted_candidates.push(formatted);
    }

    let mut builder = FzfBuilder::default();
    builder.layout(config.layout)
        .border(config.border)
        .prompt(config.prompt.clone());

    let mut custom_args = config.options.clone();
    custom_args.push("--ansi".to_string());
    custom_args.push(format!("-d{}", sep));
    custom_args.push("--nth=2".to_string());
    custom_args.push("--with-nth=2,3".to_string());
    custom_args.push(format!("--height={}", config.height));
    custom_args.push("--reverse".to_string());

    builder.custom_args(custom_args);

    let fzf = builder.build()?;
    
    let output = fzf_wrapped::run_with_output(fzf, formatted_candidates);

    if let Some(selection) = output {
        if let Some(idx) = selection.find(sep) {
            Ok(Some(selection[..idx].to_string()))
        } else {
            Ok(Some(selection))
        }
    } else {
        Ok(None)
    }
}

pub fn calculate_fzf_height(_cursor_line: usize, _total_lines: usize) -> String {
    "40%".to_string()
}
