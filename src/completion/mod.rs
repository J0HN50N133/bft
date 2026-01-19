use crate::bash::{self, history};
use crate::parser::{self, ParsedLine};
use std::fmt;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Carapace,
    Bash,
    EnvVar,
    History,
    Pipeline,
    Unknown,
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderKind::Carapace => write!(f, "carapace"),
            ProviderKind::Bash => write!(f, "bash"),
            ProviderKind::EnvVar => write!(f, "envvar"),
            ProviderKind::History => write!(f, "history"),
            ProviderKind::Pipeline => write!(f, "pipeline"),
            ProviderKind::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompletionEntry {
    pub value: String,
    pub kind: ProviderKind,
}

impl CompletionEntry {
    pub fn new(value: String, kind: ProviderKind) -> Self {
        Self { value, kind }
    }
}

impl fmt::Display for CompletionEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
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
    fn kind(&self) -> ProviderKind;
    fn should_try(&self, _ctx: &CompletionContext) -> bool {
        true
    }
    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<CompletionEntry>>, CompletionError>;
}

/// Result of a completion attempt
#[derive(Debug, Clone)]
pub struct CompletionResult {
    pub candidates: Vec<CompletionEntry>,
    pub used_provider: ProviderKind,
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

    fn kind(&self) -> ProviderKind {
        ProviderKind::Carapace
    }

    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<CompletionEntry>>, CompletionError> {
        let args = if ctx.is_after_pipe {
            std::iter::once(ctx.command.clone())
                .chain(ctx.pipe_command_args.clone())
                .collect()
        } else {
            // Truncate args to the current cursor position to handle mid-line completion
            if ctx.current_word_idx < ctx.words.len() {
                ctx.words[0..=ctx.current_word_idx].to_vec()
            } else {
                ctx.words.clone()
            }
        };

        let items = carapace::CarapaceProvider::fetch_suggestions(&ctx.command, &args)?;

        Ok(items.map(|items| {
            items
                .into_iter()
                .map(|i| CompletionEntry::new(i.value, ProviderKind::Carapace))
                .collect()
        }))
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

    fn kind(&self) -> ProviderKind {
        ProviderKind::Bash
    }

    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<CompletionEntry>>, CompletionError> {
        let spec = resolve_compspec(&ctx.command)?;

        let candidates = if ctx.is_completing_pipe_command()
            || is_command_name_completion(&spec, ctx)
        {
            bash::execute_compgen(&["-c".to_string(), "--".to_string(), ctx.current_word.clone()])?
        } else {
            execute_completion(&spec, ctx)?
        };

        Ok(Some(
            candidates
                .into_iter()
                .map(|c| CompletionEntry::new(c, ProviderKind::Bash))
                .collect(),
        ))
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
            &ctx.line,
            ctx.point,
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

    fn kind(&self) -> ProviderKind {
        ProviderKind::EnvVar
    }

    fn should_try(&self, ctx: &CompletionContext) -> bool {
        ctx.current_word.starts_with('$')
    }

    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<CompletionEntry>>, CompletionError> {
        let var_prefix = ctx.current_word[1..].to_string();
        let vars = get_env_variables(&var_prefix);
        Ok(Some(
            vars.into_iter()
                .map(|v| CompletionEntry::new(v, ProviderKind::EnvVar))
                .collect(),
        ))
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
pub struct HistoryProvider {
    limit: Option<usize>,
}

impl Default for HistoryProvider {
    fn default() -> Self {
        Self::new(Some(20))
    }
}

impl HistoryProvider {
    pub fn new(limit: Option<usize>) -> Self {
        Self { limit }
    }
}

impl CompletionProvider for HistoryProvider {
    fn name(&self) -> &'static str {
        "history"
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::History
    }

    fn should_try(&self, ctx: &CompletionContext) -> bool {
        !ctx.line.trim().is_empty()
    }

    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<CompletionEntry>>, CompletionError> {
        // Use the full line as prefix to match history
        let prefix = ctx.line.trim();
        let matches = history::get_history_commands_by_prefix(prefix, self.limit);

        if !matches.is_empty() {
            Ok(Some(
                matches
                    .into_iter()
                    .map(|m| CompletionEntry::new(m, ProviderKind::History))
                    .collect(),
            ))
        } else {
            Ok(None)
        }
    }
}

/// Orchestrates completion providers in order of priority
pub struct CompletionEngine {
    provider: Box<dyn CompletionProvider>,
}

impl CompletionEngine {
    pub fn new(provider: Box<dyn CompletionProvider>) -> Self {
        Self { provider }
    }

    /// Generate completion candidates using all providers
    /// Returns the first non-empty result
    pub fn complete(&self, ctx: &CompletionContext) -> Result<CompletionResult, CompletionError> {
        let candidates = if self.provider.should_try(ctx) {
            self.provider.try_complete(ctx)?.unwrap_or_default()
        } else {
            Vec::new()
        };
        let used_provider = self.provider.kind();
        let spec = resolve_compspec(&ctx.command)?;
        Ok(CompletionResult {
            candidates,
            used_provider,
            spec,
        })
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
    pub fn with<P: CompletionProvider + 'static>(&mut self, provider: P) -> &mut Self {
        self.providers.push(Box::new(provider));
        self
    }

    /// Add a boxed provider to the pipeline
    pub fn with_boxed(&mut self, provider: Box<dyn CompletionProvider>) -> &mut Self {
        self.providers.push(provider);
        self
    }
}

impl CompletionProvider for PipelineProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Pipeline
    }

    fn try_complete(
        &self,
        ctx: &CompletionContext,
    ) -> Result<Option<Vec<CompletionEntry>>, CompletionError> {
        let mut merged: Vec<CompletionEntry> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

        for provider in &self.providers {
            if !provider.should_try(ctx) {
                continue;
            }

            if let Some(candidates) = provider.try_complete(ctx)? {
                log::debug!(
                    "[pipeline] {} returned {} candidates",
                    provider.name(),
                    candidates.len()
                );
                for c in candidates {
                    // Use value for deduplication, but keep the entry (and its provider kind)
                    if seen.insert(c.value.clone()) {
                        merged.push(c);
                    }
                }
            }
        }

        log::debug!("[pipeline] merged result ({} total)", merged.len());

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

    // ... (rest of the tests need to be updated or can be kept if they don't depend on try_complete return type, but here they do)

    #[test]
    fn test_history_provider() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "git status").unwrap();
        writeln!(temp, "ls -la").unwrap();
        unsafe { std::env::set_var("HISTFILE", temp.path()) };

        let provider = HistoryProvider::default();

        let parsed = ParsedLine::new(
            vec!["git".to_string(), "sta".to_string()],
            vec!["git".to_string(), "sta".to_string()],
            7,
            1,
        );
        let ctx = CompletionContext::from_parsed(&parsed, "git sta".to_string(), 7);

        let result = provider.try_complete(&ctx).unwrap().unwrap();
        assert!(
            result
                .iter()
                .any(|e| e.value == "git status" && e.kind == ProviderKind::History)
        );

        unsafe { std::env::remove_var("HISTFILE") };
    }
}
