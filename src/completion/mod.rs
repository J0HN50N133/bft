use thiserror::Error;
use crate::parser::ParsedLine;

#[derive(Error, Debug)]
pub enum CompletionError {
    #[error("No completer found for command: {0}")]
    NoCompleter(String),
    #[error("Bash completion error: {0}")]
    BashError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub words: Vec<String>,
    pub current_word_idx: usize,
    pub line: String,
    pub point: usize,
    pub command: String,
    pub current_word: String,
    pub previous_word: Option<String>,
}

impl CompletionContext {
    pub fn from_parsed(parsed: &ParsedLine, line: String, point: usize) -> Self {
        let command = parsed.words.first().cloned().unwrap_or_default();
        let current_word = parsed.words.get(parsed.current_word_index).cloned().unwrap_or_default();
        let previous_word = if parsed.current_word_index > 0 {
            parsed.words.get(parsed.current_word_index - 1).cloned()
        } else {
            None
        };

        Self {
            words: parsed.words.clone(),
            current_word_idx: parsed.current_word_index,
            line,
            point,
            command,
            current_word,
            previous_word,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompletionOptions {
    pub filenames: bool,
    pub noquote: bool,
    pub nospace: bool,
    pub bashdefault: bool,
    pub default: bool,
    pub dirnames: bool,
    pub plusdirs: bool,
    pub nosort: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionSpec {
    pub function: Option<String>,
    pub wordlist: Option<String>,
    pub glob_pattern: Option<String>,
    pub command: Option<String>,
    pub filter: Option<String>,
    pub prefix: String,
    pub suffix: String,
    pub options: CompletionOptions,
}
