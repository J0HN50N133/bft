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

use crate::completion::CompletionContext;
use crate::config::Config;
use crate::selector::{Selector, SelectorConfig};

const ARG_INIT_SCRIPT: &str = "--init-script";
const ENV_READLINE_LINE: &str = "READLINE_LINE";
const ENV_READLINE_POINT: &str = "READLINE_POINT";
const DEFAULT_POINT_VALUE: &str = "0";
const DEFAULT_USIZE: usize = 0;
const COMPGEN_ARG_COMMAND: &str = "-c";
const COMPGEN_ARG_SEPARATOR: &str = "--";
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

    env_logger::init();

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

    let mut candidates = Vec::new();
    let mut completion_spec = completion::CompletionSpec::default();
    let mut used_carapace = false;

    // Determine the arguments to pass to carapace
    // If we're after a pipe, only pass the command after the pipe and its args
    // Otherwise, pass all words
    let carapace_args = if ctx.is_after_pipe {
        std::iter::once(ctx.command.clone())
            .chain(ctx.pipe_command_args.clone())
            .collect()
    } else {
        ctx.words.clone()
    };

    debug!("carapace_args: {:?}", carapace_args);

    // Environment variable completion
    if ctx.current_word.starts_with('$') {
        info!("Environment variable completion for '{}'", ctx.current_word);
        let var_prefix = ctx.current_word[1..].to_string();
        candidates = completion::get_env_variables(&var_prefix);
        info!("Generated {} env variable candidates", candidates.len());
    }
    // Try Carapace first
    else if let Ok(Some(items)) =
        completion::carapace::CarapaceProvider::fetch_suggestions(&ctx.command, &carapace_args)
    {
        if !items.is_empty() {
            info!(
                "Using Carapace provider for '{}' ({} items)",
                ctx.command,
                items.len()
            );
            candidates = items.into_iter().map(|i| i.value).collect();
            used_carapace = true;
        } else {
            debug!(
                "Carapace returned 0 items for '{}', falling back to Bash",
                ctx.command
            );
        }
    } else {
        debug!(
            "Carapace provider failed or not available for '{}'",
            ctx.command
        );
    }

    // Fallback to Bash
    if !used_carapace && !ctx.current_word.starts_with('$') {
        info!("Using Bash completion for command '{}'", ctx.command);
        completion_spec = completion::resolve_compspec(&ctx.command)?;
        debug!("Completion spec: {:?}", completion_spec);

        // Check if we're completing a command name after a pipe
        let is_completing_pipe_command = ctx.is_after_pipe 
            && ctx.current_word_idx > 0
            && parser::find_last_pipe_index(&ctx.words).map_or(false, |pipe_idx| {
                ctx.current_word_idx == pipe_idx + 1
            });

        if is_completing_pipe_command
            || (ctx.current_word_idx == 0
                && completion_spec.function.is_none()
                && completion_spec.wordlist.is_none()
                && completion_spec.command.is_none()
                && completion_spec.glob_pattern.is_none())
        {
            info!(
                "Using command completion for command name '{}'",
                ctx.current_word
            );
            candidates = bash::execute_compgen(&[
                COMPGEN_ARG_COMMAND.to_string(),
                COMPGEN_ARG_SEPARATOR.to_string(),
                ctx.current_word.clone(),
            ])?;
        } else {
            candidates = completion::execute_completion(&completion_spec, &ctx)?;
        }

        info!("Generated {} completion candidates", candidates.len());

        candidates = quoting::apply_filter(&completion_spec.filter, &candidates, &ctx.current_word)?;

        if completion_spec.options.filenames
            || completion_spec.options.default
            || completion_spec.options.bashdefault && completion_spec.options.dirnames
        {
            candidates = quoting::mark_directories(candidates);
        }
    }

    let (candidates, no_space_after_completion, _prefix) = quoting::find_common_prefix(
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
        };

        info!("Opening selector with {} candidates", candidates.len());

        let selector = crate::selector::dialoguer::DialoguerSelector::new();
        selector.select_one(&candidates, &ctx.current_word, &selector_config)?
    } else {
        debug!("Single candidate, skipping selector");
        candidates.first().cloned()
    };

    if let Some(mut completion) = selected {
        debug!("Selected completion: '{}'", completion);

        if completion_spec.options.filenames
            || completion_spec.options.default
            || completion_spec.options.bashdefault
        {
            completion = quoting::quote_filename(&completion, true);
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

fn insert_completion(
    line: &str,
    point: usize,
    completion: &str,
    nospace: bool,
    current_word: &str,
) -> Result<()> {
    let current_word_char_count = current_word.chars().count();
    let cursor_position_chars = line.chars().take(point).count();

    let replacement_start_char_index = cursor_position_chars.saturating_sub(current_word_char_count);

    let before: String = line.chars().take(replacement_start_char_index).collect();
    let after: String = line.chars().skip(cursor_position_chars).collect();

    let new_line = format!("{}{}{}", before, completion, after);
    let new_point = replacement_start_char_index + completion.chars().count();

    if !nospace && !completion.ends_with('/') {
        let new_point_byte: usize = new_line.chars().take(new_point).map(|c| c.len_utf8()).sum();

        let mut new_line_bytes: Vec<u8> = new_line.bytes().collect();
        new_line_bytes.insert(new_point_byte, b' ');

        let new_line_with_space = String::from_utf8(new_line_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to convert line to UTF-8: {}", e))?;
        let final_point = new_point_byte + 1;

        println!("{}", OUTPUT_READLINE_LINE_FORMAT.replace("{}", &new_line_with_space));
        println!("{}", OUTPUT_READLINE_POINT_FORMAT.replace("{}", &final_point.to_string()));
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
    fn test_insert_completion_trailing_utf8() {
        let line = "ls 中文";
        let point = line.chars().take(4).collect::<String>().len();
        let completion = "file.txt";
        let current_word = "中";

        let result = insert_completion(line, point, completion, false, current_word);
        assert!(result.is_ok());
    }
}
