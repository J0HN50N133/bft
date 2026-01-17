use brush_parser::{Token, tokenize_str};
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
    pub fn new(
        words: Vec<String>,
        raw_words: Vec<String>,
        cursor_position: usize,
        current_word_index: usize,
    ) -> Self {
        Self {
            words,
            raw_words,
            cursor_position,
            current_word_index,
        }
    }
}

fn byte_to_char_index(s: &str, byte_idx: usize) -> usize {
    s.char_indices()
        .take_while(|(idx, _)| *idx < byte_idx)
        .count()
}

pub fn parse_shell_line(input: &str, cursor_pos: usize) -> Result<ParsedLine, ParseError> {
    if input.trim().is_empty() {
        return Ok(ParsedLine::new(vec![], vec![], cursor_pos, 0));
    }

    let tokens = tokenize_str(input).map_err(|e| ParseError::TokenizationError(e.to_string()))?;

    let mut words = Vec::new();
    let mut raw_words = Vec::new();
    let mut current_word_index = 0;

    let cursor_char_pos = byte_to_char_index(input, cursor_pos);
    let mut found_cursor = false;
    let mut last_end_char = 0;

    for token in tokens.iter() {
        let (raw, loc) = match token {
            Token::Operator(s, l) => (s, l),
            Token::Word(s, l) => (s, l),
        };

        let start_char = byte_to_char_index(input, loc.start.index);
        let end_char = byte_to_char_index(input, loc.end.index);

        if start_char > last_end_char {
            if !found_cursor && cursor_char_pos >= last_end_char && cursor_char_pos < start_char {
                words.push(String::new());
                raw_words.push(String::new());
                current_word_index = words.len() - 1;
                found_cursor = true;
            }
        }

        words.push(unquote_string(raw));
        raw_words.push(raw.clone());

        if !found_cursor && cursor_char_pos >= start_char && cursor_char_pos <= end_char {
            current_word_index = words.len() - 1;
            found_cursor = true;
        }

        last_end_char = end_char;
    }

    if !found_cursor {
        let input_char_len = input.chars().count();
        if last_end_char < input_char_len {
            let tail_chars: Vec<char> = input.chars().skip(last_end_char).collect();
            if tail_chars.iter().any(|c| c.is_whitespace()) {
                if cursor_char_pos > last_end_char {
                    words.push(String::new());
                    raw_words.push(String::new());
                    current_word_index = words.len() - 1;
                } else {
                    current_word_index = words.len().saturating_sub(1);
                }
            } else {
                current_word_index = words.len().saturating_sub(1);
            }
        } else if cursor_char_pos > last_end_char {
            words.push(String::new());
            raw_words.push(String::new());
            current_word_index = words.len() - 1;
        } else {
            current_word_index = words.len().saturating_sub(1);
        }
    }

    Ok(ParsedLine::new(
        words,
        raw_words,
        cursor_pos,
        current_word_index,
    ))
}

pub fn unquote_string(s: &str) -> String {
    brush_parser::unquote_str(s).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let input = "ls -la";
        let parsed = parse_shell_line(input, 2).unwrap();
        assert_eq!(parsed.words, vec!["ls", "-la"]);
        assert_eq!(parsed.current_word_index, 0);

        let parsed = parse_shell_line(input, 3).unwrap();
        assert_eq!(parsed.current_word_index, 1);
        assert_eq!(parsed.words[1], "-la");
    }

    #[test]
    fn test_parse_gap() {
        let input = "ls  -la";
        let parsed = parse_shell_line(input, 3).unwrap();
        assert_eq!(parsed.words, vec!["ls", "", "-la"]);
        assert_eq!(parsed.current_word_index, 1);
        assert_eq!(parsed.words[1], "");
    }

    #[test]
    fn test_parse_quoted() {
        let input = "echo 'hello world'";
        let parsed = parse_shell_line(input, 10).unwrap();
        assert_eq!(parsed.words, vec!["echo", "hello world"]);
        assert_eq!(parsed.raw_words, vec!["echo", "'hello world'"]);
        assert_eq!(parsed.current_word_index, 1);
    }

    #[test]
    fn test_parse_trailing_space() {
        let input = "ls ";
        let parsed = parse_shell_line(input, 3).unwrap();
        assert_eq!(parsed.words, vec!["ls", ""]);
        assert_eq!(parsed.current_word_index, 1);
    }

    #[test]
    fn test_adjacent_tokens() {
        let input = "echo \"a\"\"b\"";
        let parsed = parse_shell_line(input, 9).unwrap();
        assert_eq!(parsed.words, vec!["echo", "ab"]);
    }

    #[test]
    fn test_parse_chinese() {
        let input = "ls 中文";
        let parsed = parse_shell_line(input, 9).unwrap();
        assert_eq!(parsed.words, vec!["ls", "中文"]);
        assert_eq!(parsed.current_word_index, 1);
    }

    #[test]
    fn test_parse_mixed_utf8() {
        let input = "git checkout feature-中文";
        let parsed = parse_shell_line(input, 10).unwrap();
        assert_eq!(parsed.words, vec!["git", "checkout", "feature-中文"]);
        assert_eq!(parsed.current_word_index, 1);
    }
}
