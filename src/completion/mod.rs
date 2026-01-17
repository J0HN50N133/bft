use crate::bash;
use crate::parser::{self, ParsedLine};
use thiserror::Error;

pub mod carapace;

#[derive(Error, Debug)]
pub enum CompletionError {
    #[error("No completer found for command: {0}")]
    NoCompleter(String),
    #[error("Bash completion error: {0}")]
    BashError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Bash module error: {0}")]
    BashModuleError(#[from] bash::BashError),
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
    /// If true, completion is for a command after pipe
    pub is_after_pipe: bool,
    /// The command before the pipe (for context)
    pub previous_command: Option<String>,
    /// Arguments for the command after the pipe
    pub pipe_command_args: Vec<String>,
}

impl CompletionContext {
    pub fn from_parsed(parsed: &ParsedLine, line: String, point: usize) -> Self {
        let command = parsed.words.first().cloned().unwrap_or_default();
        let current_word = parsed
            .words
            .get(parsed.current_word_index)
            .cloned()
            .unwrap_or_default();
        let previous_word = if parsed.current_word_index > 0 {
            parsed.words.get(parsed.current_word_index - 1).cloned()
        } else {
            None
        };

        // Check if we're completing after a pipe
        let pipe_idx = parser::find_last_pipe_index(&parsed.words);
        let (is_after_pipe, previous_command, pipe_command_args) = if let Some(pipe_idx) = pipe_idx {
            let cmd_idx = pipe_idx + 1;
            if parsed.current_word_index > pipe_idx {
                // We're after the pipe
                // previous_command is the word immediately before the pipe (could be the previous command or its last arg)
                let prev_cmd = parsed.words.get(pipe_idx.saturating_sub(1)).cloned();
                // pipe_command_args should exclude the command after the pipe
                let args = if cmd_idx + 1 < parsed.words.len() {
                    parsed.words[cmd_idx + 1..].to_vec()
                } else {
                    vec![]
                };
                (true, prev_cmd, args)
            } else {
                (false, None, vec![])
            }
        } else {
            (false, None, vec![])
        };

        // Determine the effective command for completion
        // If we're after a pipe, use the command after the pipe
        let effective_command = if is_after_pipe {
            if let Some(cmd) = parsed.words.get(pipe_idx.unwrap() + 1) {
                cmd.clone()
            } else {
                command
            }
        } else {
            command
        };

        Self {
            words: parsed.words.clone(),
            current_word_idx: parsed.current_word_index,
            line,
            point,
            command: effective_command,
            current_word,
            previous_word,
            is_after_pipe,
            previous_command,
            pipe_command_args,
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

pub fn resolve_compspec(command: &str) -> Result<CompletionSpec, CompletionError> {
    if command.is_empty() {
        return Ok(CompletionSpec::default());
    }

    if let Some(spec) = bash::query_complete(command)? {
        Ok(spec)
    } else {
        let mut spec = CompletionSpec::default();
        spec.options.default = true;
        Ok(spec)
    }
}

pub fn execute_completion(
    spec: &CompletionSpec,
    ctx: &CompletionContext,
) -> Result<Vec<String>, CompletionError> {
    let mut candidates = Vec::new();
    let word = &ctx.current_word;

    let run_compgen = |flags: Vec<String>| -> Result<Vec<String>, CompletionError> {
        let mut args = flags;
        args.push("--".to_string());
        args.push(word.clone());
        Ok(bash::execute_compgen(&args)?)
    };

    if let Some(function) = &spec.function {
        candidates.extend(bash::execute_completion_function(
            function,
            &ctx.command,
            word,
            ctx.previous_word.as_deref(),
            &ctx.words,
        )?);
    }

    if let Some(wordlist) = &spec.wordlist {
        candidates.extend(run_compgen(vec!["-W".to_string(), wordlist.clone()])?);
    }

    if let Some(cmd) = &spec.command {
        candidates.extend(run_compgen(vec!["-C".to_string(), cmd.clone()])?);
    }

    if let Some(glob) = &spec.glob_pattern {
        candidates.extend(run_compgen(vec!["-G".to_string(), glob.clone()])?);
    }

    if spec.options.filenames || spec.options.default {
        candidates.extend(run_compgen(vec!["-f".to_string()])?);
    }
    if spec.options.dirnames {
        candidates.extend(run_compgen(vec!["-d".to_string()])?);
    }

    Ok(candidates)
}

pub fn get_env_variables(prefix: &str) -> Vec<String> {
    let prefix_lower = prefix.to_lowercase();
    std::env::vars()
        .filter(|(k, _)| k.to_lowercase().starts_with(&prefix_lower))
        .map(|(k, _)| format!("${}", k))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParsedLine;

    fn create_parsed(words: Vec<String>, current_word_index: usize) -> ParsedLine {
        ParsedLine::new(
            words.clone(),
            words,
            0,
            current_word_index,
        )
    }

    #[test]
    fn test_completion_context_no_pipe() {
        let parsed = create_parsed(vec!["ls".to_string(), "-la".to_string()], 1);
        let ctx = CompletionContext::from_parsed(&parsed, "ls -la".to_string(), 3);
        
        assert!(!ctx.is_after_pipe);
        assert_eq!(ctx.command, "ls");
        assert_eq!(ctx.previous_command, None);
        assert!(ctx.pipe_command_args.is_empty());
    }

    #[test]
    fn test_completion_context_after_pipe() {
        let parsed = create_parsed(
            vec!["cat".to_string(), "foo.txt".to_string(), "|".to_string(), "grep".to_string(), "bar".to_string()],
            4
        );
        let ctx = CompletionContext::from_parsed(&parsed, "cat foo.txt | grep bar".to_string(), 20);
        
        assert!(ctx.is_after_pipe);
        assert_eq!(ctx.command, "grep");
        assert_eq!(ctx.previous_command, Some("foo.txt".to_string()));
        assert_eq!(ctx.pipe_command_args, vec!["bar".to_string()]);
    }

    #[test]
    fn test_completion_context_at_pipe_command() {
        let parsed = create_parsed(
            vec!["cat".to_string(), "foo.txt".to_string(), "|".to_string(), "gre".to_string()],
            3
        );
        let ctx = CompletionContext::from_parsed(&parsed, "cat foo.txt | gre".to_string(), 19);
        
        assert!(ctx.is_after_pipe);
        assert_eq!(ctx.command, "gre");
        assert_eq!(ctx.previous_command, Some("foo.txt".to_string()));
    }

    #[test]
    fn test_completion_context_before_pipe() {
        let parsed = create_parsed(
            vec!["cat".to_string(), "foo.txt".to_string(), "|".to_string(), "grep".to_string()],
            1
        );
        let ctx = CompletionContext::from_parsed(&parsed, "cat foo.txt | grep".to_string(), 8);
        
        assert!(!ctx.is_after_pipe);
        assert_eq!(ctx.command, "cat");
    }

    #[test]
    fn test_completion_context_multiple_pipes() {
        let parsed = create_parsed(
            vec!["cat".to_string(), "a.txt".to_string(), "|".to_string(), "grep".to_string(), "x".to_string(), "|".to_string(), "wc".to_string(), "-l".to_string()],
            7
        );
        let ctx = CompletionContext::from_parsed(&parsed, "cat a.txt | grep x | wc -l".to_string(), 25);
        
        assert!(ctx.is_after_pipe);
        assert_eq!(ctx.command, "wc");
        assert_eq!(ctx.previous_command, Some("x".to_string()));
        assert_eq!(ctx.pipe_command_args, vec!["-l".to_string()]);
    }
}
