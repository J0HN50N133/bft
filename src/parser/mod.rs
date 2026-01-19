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

    let mut current_word = String::new();
    let mut current_start = 0;
    let mut in_word = false;
    let mut quote_char = None;
    let mut escaped = false;

    for (i, c) in input.char_indices() {
        if escaped {
            current_word.push(c);
            escaped = false;
            continue;
        }

        if c == '\\' {
            escaped = true;
            current_word.push(c);
            if !in_word {
                in_word = true;
                current_start = i;
            }
            continue;
        }

        if let Some(q) = quote_char {
            if c == q {
                quote_char = None;
            }
            current_word.push(c);
        } else if c == '\'' || c == '"' {
            quote_char = Some(c);
            current_word.push(c);
            if !in_word {
                in_word = true;
                current_start = i;
            }
        } else if c.is_whitespace() {
            if in_word {
                words.push(current_word.clone());
                indices.push((current_start, i));
                current_word.clear();
                in_word = false;
            }
        } else {
            current_word.push(c);
            if !in_word {
                in_word = true;
                current_start = i;
            }
        }
    }

    if in_word {
        words.push(current_word);
        indices.push((current_start, input.len()));
    }

    let mut current_word_index = 0;
    if words.is_empty() {
        words.push(String::new());
        current_word_index = 0;
    } else {
        let mut found = false;
        for (i, (start, end)) in indices.iter().enumerate() {
            if cursor_pos >= *start && cursor_pos <= *end {
                current_word_index = i;
                found = true;
                break;
            }
        }

        if !found {
            if cursor_pos > indices.last().unwrap().1 {
                words.push(String::new());
                current_word_index = words.len() - 1;
            } else if cursor_pos < indices.first().unwrap().0 {
                current_word_index = 0;
            } else {
                // In between words, insert empty word
                for (i, (_, end)) in indices.iter().enumerate() {
                    if i + 1 < indices.len() {
                        let next_start = indices[i + 1].0;
                        if cursor_pos > *end && cursor_pos < next_start {
                            words.insert(i + 1, String::new());
                            current_word_index = i + 1;
                            break;
                        }
                    }
                }
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
    fn test_fallback_unclosed_quote() {
        let input = "ls 'file na";
        // brush-parser fails. Fallback used.
        // Current fallback: split_whitespace -> ["ls", "'file", "na"]
        // Desired: ["ls", "'file na"] (treated as one word)

        let parsed = parse_shell_line(input, 11).unwrap();
        assert_eq!(parsed.words, vec!["ls", "'file na"]);
        assert_eq!(parsed.current_word_index, 1);
    }

    #[test]
    fn test_fallback_parse() {
        let input = "ls $(cat ";
        let parsed = parse_shell_line(input, 9).unwrap();
        assert_eq!(parsed.words, vec!["ls", "$(cat", ""]);
        assert_eq!(parsed.current_word_index, 2);
    }
}
