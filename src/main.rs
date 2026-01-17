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

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--init-script" {
        print!("{}", include_str!("../scripts/bft.bash"));
        return Ok(());
    }

    let readline_line = if args.len() >= 2 {
        args[1].clone()
    } else {
        env::var("READLINE_LINE").unwrap_or_default()
    };

    let readline_point: usize = if args.len() >= 3 {
        args[2].parse().unwrap_or(0)
    } else {
        env::var("READLINE_POINT")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap_or(0)
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
        "Command: '{}', current_word: '{}', current_word_idx: {}",
        ctx.command, ctx.current_word, ctx.current_word_idx
    );

    let mut candidates = Vec::new();
    let mut spec = completion::CompletionSpec::default();
    let mut used_carapace = false;

    // Environment variable completion
    if ctx.current_word.starts_with('$') {
        info!("Environment variable completion for '{}'", ctx.current_word);
        let var_prefix = ctx.current_word[1..].to_string();
        candidates = completion::get_env_variables(&var_prefix);
        info!("Generated {} env variable candidates", candidates.len());
    }
    // Try Carapace first
    else if let Ok(Some(items)) =
        completion::carapace::CarapaceProvider::fetch_suggestions(&ctx.command, &ctx.words)
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
        spec = completion::resolve_compspec(&ctx.command)?;
        debug!("Completion spec: {:?}", spec);

        if ctx.current_word_idx == 0
            && spec.function.is_none()
            && spec.wordlist.is_none()
            && spec.command.is_none()
            && spec.glob_pattern.is_none()
        {
            info!(
                "Using command completion for command name '{}'",
                ctx.current_word
            );
            candidates = bash::execute_compgen(&[
                "-c".to_string(),
                "--".to_string(),
                ctx.current_word.clone(),
            ])?;
        } else {
            candidates = completion::execute_completion(&spec, &ctx)?;
        }

        info!("Generated {} completion candidates", candidates.len());

        candidates = quoting::apply_filter(&spec.filter, &candidates, &ctx.current_word)?;

        if spec.options.filenames
            || spec.options.default
            || spec.options.bashdefault && spec.options.dirnames
        {
            candidates = quoting::mark_directories(candidates);
        }
    }

    let (candidates, nospace, _prefix) = quoting::find_common_prefix(
        &candidates,
        ctx.current_word.len(),
        config.auto_common_prefix_part,
    );

    debug!("After filtering: {} candidates", candidates.len());

    let selected = if candidates.len() > 1 {
        let selector_config = SelectorConfig {
            ctx: ctx.clone(),
            prompt: config.prompt.clone(),
            height: config.fzf_tmux_height.unwrap_or_else(|| "40%".to_string()),
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

        if spec.options.filenames || spec.options.default || spec.options.bashdefault {
            completion = quoting::quote_filename(&completion, true);
        }

        insert_completion(
            &readline_line,
            readline_point,
            &completion,
            nospace,
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
    let prefix_char_len = current_word.chars().count();
    let point_char = line.chars().take(point).count();

    let start_char = point_char.saturating_sub(prefix_char_len);

    let before: String = line.chars().take(start_char).collect();
    let after: String = line.chars().skip(point_char).collect();

    let new_line = format!("{}{}{}", before, completion, after);
    let new_point = start_char + completion.chars().count();

    if !nospace && !completion.ends_with('/') {
        let new_point_byte: usize = new_line.chars().take(new_point).map(|c| c.len_utf8()).sum();

        let mut new_line_bytes: Vec<u8> = new_line.bytes().collect();
        new_line_bytes.insert(new_point_byte, b' ');

        let new_line_with_space = String::from_utf8(new_line_bytes).unwrap();
        let final_point = new_point_byte + 1;

        println!("READLINE_LINE='{}'", new_line_with_space);
        println!("READLINE_POINT={}", final_point);
    } else {
        let new_point_byte: usize = new_line.chars().take(new_point).map(|c| c.len_utf8()).sum();
        println!("READLINE_LINE='{}'", new_line);
        println!("READLINE_POINT={}", new_point_byte);
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
