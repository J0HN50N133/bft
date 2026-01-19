pub mod bash;
pub mod completion;
pub mod config;
pub mod parser;
pub mod quoting;
pub mod selector;

use anyhow::Result;
use log::{debug, info};
use std::env;
use std::rc::Rc;

use crate::completion::{
    BashProvider, CarapaceProvider, CompletionContext, CompletionEngine, CompletionEntry,
    CompletionResult, EnvVarProvider, HistoryProvider, PipelineProvider, ProviderKind,
};
use crate::config::Config;
use crate::selector::{Selector, SelectorConfig};

const ARG_INIT_SCRIPT: &str = "--init-script";
const ENV_READLINE_LINE: &str = "READLINE_LINE";
const ENV_READLINE_POINT: &str = "READLINE_POINT";
const DEFAULT_POINT_VALUE: &str = "0";
const DEFAULT_USIZE: usize = 0;
const OUTPUT_READLINE_LINE_FORMAT: &str = "READLINE_LINE='{}'";
const OUTPUT_READLINE_POINT_FORMAT: &str = "READLINE_POINT={}";
const DEFAULT_FZF_TMUX_HEIGHT: &str = "40%";

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == ARG_INIT_SCRIPT {
        print!("{}", include_str!("../scripts/bft.bash"));
        return Ok(());
    }

    let readline_line = if args.len() >= 2 {
        args[1].clone()
    } else {
        env::var(ENV_READLINE_LINE).unwrap_or_default()
    };

    let readline_point: usize = if args.len() >= 3 {
        args[2].parse().unwrap_or(DEFAULT_USIZE)
    } else {
        env::var(ENV_READLINE_POINT)
            .unwrap_or_else(|_| DEFAULT_POINT_VALUE.to_string())
            .parse()
            .unwrap_or(DEFAULT_USIZE)
    };

    env_logger::builder()
        .format_file(true)
        .format_line_number(true)
        .init();

    info!("Starting bft");

    let config = Config::from_env();

    debug!("Input: line='{}', point={}", readline_line, readline_point);

    if config.no_empty_cmd_completion && readline_line.trim().is_empty() {
        debug!("Empty command line, skipping completion");
        return Ok(());
    }

    let parsed = parser::parse_shell_line(&readline_line, readline_point)?;
    debug!("Parsed command: {:?}", parsed);

    let ctx = Rc::new(CompletionContext::from_parsed(
        &parsed,
        readline_line.clone(),
        readline_point,
    ));

    debug!(
        "Command: '{}', current_word: '{}', current_word_idx: {}, is_after_pipe: {}",
        ctx.command, ctx.current_word, ctx.current_word_idx, ctx.is_after_pipe
    );

    let pipeline = PipelineProvider::new("history+envvar+carapace+bash")
        .with(BashProvider::new())
        .with(HistoryProvider::new())
        .with(CarapaceProvider::new())
        .with(EnvVarProvider::new());
    let engine = CompletionEngine::new(Box::new(pipeline));
    let result = engine.complete(&ctx)?;

    info!(
        "Using {} provider, generated {} candidates",
        result.used_provider,
        result.candidates.len()
    );

    let candidates = apply_post_processing(&result, &ctx.current_word, &config)?;

    let (candidates, no_space_after_completion, _prefix) = crate::quoting::find_common_prefix(
        &candidates,
        ctx.current_word.len(),
        config.auto_common_prefix_part,
    );

    debug!("After filtering: {} candidates", candidates.len());

    let selected = if candidates.len() > 1 {
        let selector_config = SelectorConfig {
            ctx: ctx.clone(),
            prompt: config.prompt.clone(),
            height: config
                .fzf_tmux_height
                .clone()
                .unwrap_or_else(|| DEFAULT_FZF_TMUX_HEIGHT.to_string()),
            header: Some(readline_line.clone()),
            fuzzy: true,
        };

        info!("Opening selector with {} candidates", candidates.len());

        let selector = crate::selector::dialoguer::DialoguerSelector::new();
        selector.select_one(&candidates, &ctx.current_word, &selector_config)?
    } else {
        debug!("Single candidate, skipping selector");
        candidates.first().cloned()
    };

    if let Some(entry) = selected {
        debug!("Selected completion: '{}' ({})", entry.value, entry.kind);
        let mut completion = entry.value;

        let current_word_char_count = ctx.current_word.chars().count();
        let cursor_position_chars = readline_line.chars().take(readline_point).count();
        let replacement_start_char_index =
            cursor_position_chars.saturating_sub(current_word_char_count);
        let before: String = readline_line
            .chars()
            .take(replacement_start_char_index)
            .collect();

        let is_full_line = !before.is_empty() && completion.starts_with(&before);

        if !is_full_line
            && entry.kind != ProviderKind::History
            && (result.spec.options.filenames
                || result.spec.options.default
                || result.spec.options.bashdefault)
        {
            completion = crate::quoting::quote_filename(&completion, true);
        }

        insert_completion(
            &readline_line,
            readline_point,
            &completion,
            no_space_after_completion,
            &ctx.current_word,
        )?;
    } else {
        info!("No completion selected");
    }

    info!("Completion finished");
    Ok(())
}

fn apply_post_processing(
    result: &CompletionResult,
    current_word: &str,
    _config: &Config,
) -> Result<Vec<CompletionEntry>, crate::completion::CompletionError> {
    let mut candidates = result.candidates.clone();

    candidates = crate::quoting::apply_filter(&result.spec.filter, &candidates, current_word)?;

    if result.spec.options.filenames
        || result.spec.options.default
        || result.spec.options.bashdefault && result.spec.options.dirnames
    {
        candidates = crate::quoting::mark_directories(candidates);
    }

    Ok(candidates)
}

fn insert_completion(
    line: &str,
    point: usize,
    completion: &str,
    nospace: bool,
    current_word: &str,
) -> Result<()> {
    let current_word_char_count = current_word.chars().count();
    let cursor_position_chars = line.chars().take(point).count();

    let replacement_start_char_index =
        cursor_position_chars.saturating_sub(current_word_char_count);

    let before: String = line.chars().take(replacement_start_char_index).collect();
    let after: String = line.chars().skip(cursor_position_chars).collect();

    let new_line = if completion.starts_with(&before) && !before.is_empty() {
        format!("{}{}", completion, after)
    } else {
        format!("{}{}{}", before, completion, after)
    };

    let new_point = if completion.starts_with(&before) && !before.is_empty() {
        completion.chars().count()
    } else {
        replacement_start_char_index + completion.chars().count()
    };

    if !nospace && !completion.ends_with('/') {
        let new_point_byte: usize = new_line.chars().take(new_point).map(|c| c.len_utf8()).sum();

        let mut new_line_bytes: Vec<u8> = new_line.bytes().collect();
        new_line_bytes.insert(new_point_byte, b' ');

        let new_line_with_space = String::from_utf8(new_line_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to convert line to UTF-8: {}", e))?;
        let final_point = new_point_byte + 1;

        println!(
            "{}",
            OUTPUT_READLINE_LINE_FORMAT.replace("{}", &new_line_with_space)
        );
        println!(
            "{}",
            OUTPUT_READLINE_POINT_FORMAT.replace("{}", &final_point.to_string())
        );
    } else {
        let new_point_byte: usize = new_line.chars().take(new_point).map(|c| c.len_utf8()).sum();
        println!("{}", OUTPUT_READLINE_LINE_FORMAT.replace("{}", &new_line));
        println!(
            "{}",
            OUTPUT_READLINE_POINT_FORMAT.replace("{}", &new_point_byte.to_string())
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_completion_ascii() {
        let line = "ls file";
        let point = line.len();
        let completion = "file.txt";
        let current_word = "file";

        let result = insert_completion(line, point, completion, false, current_word);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_completion_chinese() {
        let line = "ls 中文";
        let point = line.len();
        let completion = "test.txt";
        let current_word = "中文";

        let result = insert_completion(line, point, completion, false, current_word);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_completion_mixed() {
        let line = "git checkout feat";
        let point = line.len();
        let completion = "feature-中文";
        let current_word = "feat";

        let result = insert_completion(line, point, completion, false, current_word);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_completion_nospace() {
        let line = "cd path";
        let point = line.len();
        let completion = "/";
        let current_word = "path";

        let result = insert_completion(line, point, completion, true, current_word);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_completion_empty_word() {
        let line = "ls ";
        let point = line.len();
        let completion = "file.txt";
        let current_word = "";

        let result = insert_completion(line, point, completion, false, current_word);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_completion_full_line() {
        let line = "git sta";
        let point = line.len();
        let completion = "git status"; // Full line completion
        let current_word = "sta";

        let result = insert_completion(line, point, completion, false, current_word);
        assert!(result.is_ok());
    }
}
