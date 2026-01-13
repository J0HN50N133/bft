use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to tokenize input: {0}")]
    TokenizationError(String),
    #[error("Failed to parse tokens: {0}")]
    ParsingError(String),
    #[error("Failed to extract words from AST")]
    WordExtractionError,
    #[error("Cursor position out of bounds")]
    CursorOutOfBounds,
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub struct ParsedLine {
    pub words: Vec<String>,
    pub raw_words: Vec<String>,
    pub cursor_position: usize,
    pub current_word_index: usize,
}

impl ParsedLine {
    pub fn new(words: Vec<String>, raw_words: Vec<String>, cursor_position: usize, current_word_index: usize) -> Self {
        Self {
            words,
            raw_words,
            cursor_position,
            current_word_index,
        }
    }
}
