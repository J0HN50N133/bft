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

    let tokens = match tokenize_str(input) {
        Ok(t) => t,
        Err(_) => return Ok(fallback_parse(input, cursor_pos)),
    };

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

        if start_char > last_end_char
            && !found_cursor
            && cursor_char_pos >= last_end_char
            && cursor_char_pos < start_char
        {
            words.push(String::new());
            raw_words.push(String::new());
            current_word_index = words.len() - 1;
            found_cursor = true;
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

fn fallback_parse(input: &str, cursor_pos: usize) -> ParsedLine {
    let mut words = Vec::new();
    let mut indices = Vec::new();
    let mut current_word_index = 0;

    // Simple split by whitespace, keeping track of indices
    let mut current_idx = 0;
    for (i, part) in input.split_whitespace().enumerate() {
        let start = input[current_idx..].find(part).unwrap() + current_idx;
        let end = start + part.len();

        words.push(part.to_string());
        indices.push((start, end));

        if cursor_pos >= start && cursor_pos <= end {
            current_word_index = i;
        }

        current_idx = end;
    }

    // Handle cursor at the end or in whitespace
    if words.is_empty() {
        words.push(String::new());
        current_word_index = 0;
    } else if cursor_pos > indices.last().unwrap().1 {
        words.push(String::new());
        current_word_index = words.len() - 1;
    } else if cursor_pos < indices.first().unwrap().0 {
        // Should act as if before the first word, but we usually attach to the closest?
        // Or insert empty at start? Let's just say index 0.
        current_word_index = 0;
    } else {
        // Check if cursor is between words
        let mut found = false;
        for (i, (start, end)) in indices.iter().enumerate() {
            if cursor_pos >= *start && cursor_pos <= *end {
                current_word_index = i;
                found = true;
                break;
            }
        }
        if !found {
            // Cursor in whitespace between words.
            // We need to decide if we are at the end of previous or start of next.
            // But usually this means we are typing a new word.
            // Logic similar to main parser:
            // If we are strictly AFTER a word and BEFORE another, we are in a new word slot.
            for (i, (_, end)) in indices.iter().enumerate() {
                if i + 1 < indices.len() {
                    let next_start = indices[i + 1].0;
                    if cursor_pos > *end && cursor_pos < next_start {
                        // insert empty word
                        // But we can't easily insert into `words` and adjust indices in this simplified view without reconstructing.
                        // For fallback, simpler might be: match to the *previous* word if cursor is touching it,
                        // otherwise match to *next* word?
                        // Or just assume we are appending to the previous one?
                        // Let's rely on standard split logic:
                        // "ls  -la" -> ["ls", "-la"]. Cursor at 3 (between).
                        // We should probably behave like we are on "-la" (index 1) or a new word?
                        // The main parser inserts an empty string.

                        // Let's refine the fallback:
                        // Just split by whitespace. If cursor is in whitespace, we are in a "gap".
                        // BUT, we want to return something usable.
                        // If we just return what we have, `current_word_index` might point to the previous word.

                        // Let's try to match the behavior of finding where the cursor is.
                        if cursor_pos > *end {
                            current_word_index = i + 1;
                        }
                    }
                }
            }
        }
    }

    // Special case: if we are forcing a "new word" because of whitespace, we might need to insert an empty string
    // into `words` to represent the cursor being on a new, empty word.
    // E.g. "ls " -> words=["ls"], cursor after space.
    // We want words=["ls", ""], index=1.

    if cursor_pos > 0 && input[..cursor_pos].chars().last().unwrap().is_whitespace() {
        // We are after some whitespace.
        // If we are not already pointing to a word that starts exactly here...
        // Actually split_whitespace eats the whitespace.
        // So "ls " gives ["ls"]. Last word ends before cursor.
        // So we should append an empty word.
        if !words.is_empty() && indices.last().unwrap().1 < cursor_pos {
            // Only push if we haven't already pushed one in the block above
            // Check if the last word is empty (which we just pushed)
            if !words.last().unwrap().is_empty() {
                words.push(String::new());
                current_word_index = words.len() - 1;
            }
        }
    }

    ParsedLine::new(
        words.clone(),
        words, // raw_words same as words for fallback
        cursor_pos,
        current_word_index,
    )
}

pub fn unquote_string(s: &str) -> String {
    brush_parser::unquote_str(s).to_string()
}

/// Find the last pipe (|) operator index in the words list
/// Returns None if no pipe is found
pub fn find_last_pipe_index(words: &[String]) -> Option<usize> {
    words.iter().rposition(|w| w == "|")
}

/// Get the command after the last pipe operator
/// Returns (command_name, args_after_pipe) if found
pub fn get_command_after_pipe(words: &[String]) -> Option<(String, Vec<String>)> {
    let pipe_idx = find_last_pipe_index(words)?;
    let cmd_idx = pipe_idx + 1;

    if cmd_idx >= words.len() {
        return None;
    }

    let command = words[cmd_idx].clone();
    let args = words[cmd_idx + 1..].to_vec();
    Some((command, args))
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

    #[test]
    fn test_find_last_pipe_index() {
        let words = vec![
            "cat".to_string(),
            "foo.txt".to_string(),
            "|".to_string(),
            "grep".to_string(),
        ];
        assert_eq!(find_last_pipe_index(&words), Some(2));

        let words_no_pipe = vec!["ls".to_string(), "-la".to_string()];
        assert_eq!(find_last_pipe_index(&words_no_pipe), None);
    }

    #[test]
    fn test_get_command_after_pipe() {
        let words = vec![
            "cat".to_string(),
            "foo.txt".to_string(),
            "|".to_string(),
            "grep".to_string(),
            "bar".to_string(),
        ];
        let result = get_command_after_pipe(&words);
        assert_eq!(result, Some(("grep".to_string(), vec!["bar".to_string()])));

        let words_no_pipe = vec!["ls".to_string(), "-la".to_string()];
        assert_eq!(get_command_after_pipe(&words_no_pipe), None);

        let words_empty_after_pipe =
            vec!["cat".to_string(), "foo.txt".to_string(), "|".to_string()];
        assert_eq!(get_command_after_pipe(&words_empty_after_pipe), None);
    }

    #[test]
    fn test_fallback_parse() {
        let input = "ls $(cat ";
        // brush-parser would fail on this due to unclosed parenthesis/substitution
        // We expect the fallback to handle it.
        let parsed = parse_shell_line(input, 9).unwrap();
        assert_eq!(parsed.words, vec!["ls", "$(cat", ""]);
        // "ls", "$(cat", "" because of the trailing space.
        // Wait, "ls $(cat " -> split whitespace -> "ls", "$(cat".
        // cursor is at 9 (len is 9).
        // Input is "ls $(cat "
        // Indices:
        // "ls": 0..2
        // "$(cat": 3..8
        // Cursor at 9. 9 > 8.
        // Fallback logic for trailing whitespace: if cursor > last word end, push empty.

        assert_eq!(parsed.words, vec!["ls", "$(cat", ""]);
        assert_eq!(parsed.current_word_index, 2);
    }
}
