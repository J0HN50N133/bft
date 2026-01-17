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

impl From<anyhow::Error> for CompletionError {
    fn from(e: anyhow::Error) -> Self {
        CompletionError::Other(e.to_string())
    }
}

impl From<glob::PatternError> for CompletionError {
    fn from(e: glob::PatternError) -> Self {
        CompletionError::Other(e.to_string())
    }
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

        let pipe_idx = parser::find_last_pipe_index(&parsed.words);
        let (is_after_pipe, previous_command, pipe_command_args) = if let Some(pipe_idx) = pipe_idx
        {
            let cmd_idx = pipe_idx + 1;
            if parsed.current_word_index > pipe_idx {
                let prev_cmd = parsed.words.get(pipe_idx.saturating_sub(1)).cloned();
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

    /// Returns true if we're completing a command name after a pipe
    pub fn is_completing_pipe_command(&self) -> bool {
        self.is_after_pipe
            && self.current_word_idx > 0
            && parser::find_last_pipe_index(&self.words)
                .is_some_and(|pipe_idx| self.current_word_idx == pipe_idx + 1)
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

/// Trait for completion providers
pub trait CompletionProvider: Send {
    fn name(&self) -> &str;
    fn try_complete(&self, ctx: &CompletionContext)
    -> Result<Option<Vec<String>>, CompletionError>;
}

/// Result of a completion attempt
#[derive(Debug, Clone)]
pub struct CompletionResult {
    pub candidates: Vec<String>,
    pub used_provider: String,
    pub spec: CompletionSpec,
}

impl CompletionResult {
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }
}

/// Carapace-based completion provider
pub struct CarapaceProvider;

impl Default for CarapaceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CarapaceProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CompletionProvider for CarapaceProvider {
    fn name(&self) -> &'static str {
        "carapace"
    }

    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<String>>, CompletionError> {
        let args = if ctx.is_after_pipe {
            std::iter::once(ctx.command.clone())
                .chain(ctx.pipe_command_args.clone())
                .collect()
        } else {
            ctx.words.clone()
        };

        let items = carapace::CarapaceProvider::fetch_suggestions(&ctx.command, &args)?;

        Ok(items.map(|items| items.into_iter().map(|i| i.value).collect()))
    }
}

/// Bash-based completion provider
pub struct BashProvider;

impl Default for BashProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl BashProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CompletionProvider for BashProvider {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<String>>, CompletionError> {
        let spec = resolve_compspec(&ctx.command)?;

        if ctx.is_completing_pipe_command() || is_command_name_completion(&spec, ctx) {
            let candidates = bash::execute_compgen(&[
                "-c".to_string(),
                "--".to_string(),
                ctx.current_word.clone(),
            ])?;
            Ok(Some(candidates))
        } else {
            let candidates = execute_completion(&spec, ctx)?;
            Ok(Some(candidates))
        }
    }
}

fn is_command_name_completion(spec: &CompletionSpec, ctx: &CompletionContext) -> bool {
    ctx.current_word_idx == 0
        && spec.function.is_none()
        && spec.wordlist.is_none()
        && spec.command.is_none()
        && spec.glob_pattern.is_none()
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

/// Environment variable completion provider
pub struct EnvVarProvider;

impl Default for EnvVarProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvVarProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CompletionProvider for EnvVarProvider {
    fn name(&self) -> &'static str {
        "envvar"
    }

    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<String>>, CompletionError> {
        if ctx.current_word.starts_with('$') {
            let var_prefix = ctx.current_word[1..].to_string();
            Ok(Some(get_env_variables(&var_prefix)))
        } else {
            Ok(None)
        }
    }
}

pub fn get_env_variables(prefix: &str) -> Vec<String> {
    let prefix_lower = prefix.to_lowercase();
    std::env::vars()
        .filter(|(k, _)| k.to_lowercase().starts_with(&prefix_lower))
        .map(|(k, _)| format!("${}", k))
        .collect()
}

/// History-based completion provider
pub struct HistoryProvider;

impl Default for HistoryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CompletionProvider for HistoryProvider {
    fn name(&self) -> &'static str {
        "history"
    }

    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<String>>, CompletionError> {
        // Use the full line as prefix to match history
        let prefix = ctx.line.trim();
        if prefix.is_empty() {
            return Ok(None);
        }

        let matches = crate::bash::history::get_history_commands_by_substring(prefix, Some(20));

        if !matches.is_empty() {
            return Ok(Some(matches));
        }

        Ok(None)
    }
}

/// Orchestrates completion providers in order of priority
pub struct CompletionEngine {
    providers: Vec<Box<dyn CompletionProvider>>,
}

impl CompletionEngine {
    pub fn new() -> Self {
        Self {
            providers: vec![
                Box::new(EnvVarProvider::new()) as Box<dyn CompletionProvider>,
                Box::new(CarapaceProvider::new()) as Box<dyn CompletionProvider>,
                Box::new(HistoryProvider::new()) as Box<dyn CompletionProvider>,
                Box::new(BashProvider::new()) as Box<dyn CompletionProvider>,
            ],
        }
    }

    /// Generate completion candidates using all providers
    /// Returns the first non-empty result
    pub fn complete(&self, ctx: &CompletionContext) -> Result<CompletionResult, CompletionError> {
        for provider in &self.providers {
            if let Some(candidates) = provider.try_complete(ctx)?
                && !candidates.is_empty()
            {
                let spec = resolve_compspec(&ctx.command)?;
                return Ok(CompletionResult {
                    candidates,
                    used_provider: provider.name().to_string(),
                    spec,
                });
            }
        }
        Ok(CompletionResult {
            candidates: vec![],
            used_provider: "none".to_string(),
            spec: CompletionSpec::default(),
        })
    }

    /// Generate completion candidates using a pipeline
    /// Results are merged from all providers with deduplication
    pub fn complete_pipeline(
        &self,
        ctx: &CompletionContext,
        pipeline: &PipelineProvider,
    ) -> Result<CompletionResult, CompletionError> {
        let candidates = pipeline.try_complete(ctx)?;

        if let Some(merged) = candidates
            && !merged.is_empty()
        {
            let spec = resolve_compspec(&ctx.command)?;
            return Ok(CompletionResult {
                candidates: merged,
                used_provider: pipeline.name().to_string(),
                spec,
            });
        }

        // Fall back to first non-empty provider
        self.complete(ctx)
    }
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Combines multiple providers into a pipeline
/// Results are merged with deduplication, earlier providers have higher priority
pub struct PipelineProvider {
    name: String,
    providers: Vec<Box<dyn CompletionProvider>>,
}

impl PipelineProvider {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            providers: Vec::new(),
        }
    }

    /// Add a provider to the pipeline
    pub fn with<P: CompletionProvider + 'static>(mut self, provider: P) -> Self {
        self.providers.push(Box::new(provider));
        self
    }

    /// Add a boxed provider to the pipeline
    pub fn with_boxed(mut self, provider: Box<dyn CompletionProvider>) -> Self {
        self.providers.push(provider);
        self
    }
}

impl CompletionProvider for PipelineProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<String>>, CompletionError> {
        let mut merged: Vec<String> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

        for provider in &self.providers {
            if let Some(candidates) = provider.try_complete(ctx)? {
                log::debug!(
                    "[pipeline] {} returned {} candidates: {:?}",
                    provider.name(),
                    candidates.len(),
                    candidates
                );
                for c in candidates {
                    if seen.insert(c.clone()) {
                        merged.push(c);
                    }
                }
            }
        }

        log::debug!(
            "[pipeline] merged result ({} total): {:?}",
            merged.len(),
            merged
        );

        if merged.is_empty() {
            Ok(None)
        } else {
            Ok(Some(merged))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParsedLine;

    fn create_parsed(words: Vec<String>, current_word_index: usize) -> ParsedLine {
        ParsedLine::new(words.clone(), words, 0, current_word_index)
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
            vec![
                "cat".to_string(),
                "foo.txt".to_string(),
                "|".to_string(),
                "grep".to_string(),
                "bar".to_string(),
            ],
            4,
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
            vec![
                "cat".to_string(),
                "foo.txt".to_string(),
                "|".to_string(),
                "gre".to_string(),
            ],
            3,
        );
        let ctx = CompletionContext::from_parsed(&parsed, "cat foo.txt | gre".to_string(), 19);

        assert!(ctx.is_after_pipe);
        assert_eq!(ctx.command, "gre");
        assert_eq!(ctx.previous_command, Some("foo.txt".to_string()));
    }

    #[test]
    fn test_completion_context_before_pipe() {
        let parsed = create_parsed(
            vec![
                "cat".to_string(),
                "foo.txt".to_string(),
                "|".to_string(),
                "grep".to_string(),
            ],
            1,
        );
        let ctx = CompletionContext::from_parsed(&parsed, "cat foo.txt | grep".to_string(), 8);

        assert!(!ctx.is_after_pipe);
        assert_eq!(ctx.command, "cat");
    }

    #[test]
    fn test_completion_context_multiple_pipes() {
        let parsed = create_parsed(
            vec![
                "cat".to_string(),
                "a.txt".to_string(),
                "|".to_string(),
                "grep".to_string(),
                "x".to_string(),
                "|".to_string(),
                "wc".to_string(),
                "-l".to_string(),
            ],
            7,
        );
        let ctx =
            CompletionContext::from_parsed(&parsed, "cat a.txt | grep x | wc -l".to_string(), 25);

        assert!(ctx.is_after_pipe);
        assert_eq!(ctx.command, "wc");
        assert_eq!(ctx.previous_command, Some("x".to_string()));
        assert_eq!(ctx.pipe_command_args, vec!["-l".to_string()]);
    }
}
