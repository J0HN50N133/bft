use thiserror::Error;
use brush_parser::{tokenize_str, Token};

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

pub fn parse_shell_line(input: &str, cursor_pos: usize) -> Result<ParsedLine, ParseError> {
    if input.trim().is_empty() {
        return Ok(ParsedLine::new(vec![], vec![], cursor_pos, 0));
    }

    let tokens = tokenize_str(input)
        .map_err(|e| ParseError::TokenizationError(e.to_string()))?;

    let mut words = Vec::new();
    let mut raw_words = Vec::new();
    let mut current_word_index = 0;
    
    let mut found_cursor = false;
    let mut last_end = 0;

    for token in tokens.iter() {
        let (raw, loc) = match token {
            Token::Operator(s, l) => (s, l),
            Token::Word(s, l) => (s, l),
        };

        if loc.start.index > last_end {
            let gap = &input[last_end..loc.start.index];
            if gap.chars().any(char::is_whitespace) {
                // If there's a whitespace gap, we *might* insert an empty word if the cursor is here
                // OR if the gap is "significant" enough to separate words even without cursor.
                // In bash, "ls -la" -> words=["ls", "-la"].
                // "ls  -la" -> words=["ls", "-la"] (extra space doesn't make extra word unless cursor is there?).
                // Actually, COMP_WORDS generally splits by IFS. "ls  -la" -> "ls", "-la".
                // But if cursor is in the middle of spaces, we need a word there to complete.
                
                if !found_cursor && cursor_pos >= last_end && cursor_pos < loc.start.index {
                    words.push(String::new());
                    raw_words.push(String::new());
                    current_word_index = words.len() - 1;
                    found_cursor = true;
                }
            }
        }

        words.push(unquote_string(raw));
        raw_words.push(raw.clone());

        if !found_cursor {
            if cursor_pos >= loc.start.index && cursor_pos <= loc.end.index {
                current_word_index = words.len() - 1;
                found_cursor = true;
            }
        }
        
        last_end = loc.end.index;
    }

    if !found_cursor {
        // Cursor after all tokens
        let tail_start = last_end;
        if tail_start < input.len() {
            let tail = &input[tail_start..];
            if tail.chars().any(char::is_whitespace) {
                words.push(String::new());
                raw_words.push(String::new());
                current_word_index = words.len() - 1;
            } else if cursor_pos > tail_start {
                 // Cursor is in tail (trailing whitespace or empty)
                 words.push(String::new());
                 raw_words.push(String::new());
                 current_word_index = words.len() - 1;
            } else {
                 // Cursor exactly at end of last token?
                 // Should have been caught by loop (<= loc.end.index)
                 // But loop uses last_end which is exclusive end.
                 // If cursor_pos == last_end, it was matched in loop.
                 current_word_index = words.len().saturating_sub(1);
            }
        } else if cursor_pos > tail_start {
             // Cursor past end of string?
             words.push(String::new());
             raw_words.push(String::new());
             current_word_index = words.len() - 1;
        } else {
             // Exact match at end
             current_word_index = words.len().saturating_sub(1);
        }
    }

    Ok(ParsedLine::new(words, raw_words, cursor_pos, current_word_index))
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
}
