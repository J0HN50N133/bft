use crate::bash;
use crate::parser::ParsedLine;
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
