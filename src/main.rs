pub mod parser;
pub mod completion;
pub mod fzf;
pub mod quoting;
pub mod bash;
pub mod config;

use std::env;
use std::io::{self, Write};
use anyhow::{Context, Result};
use crossterm::cursor::{SavePosition, RestorePosition};
use crossterm::terminal::{Clear, ClearType};
use crossterm::execute;

use crate::config::Config;
use crate::completion::CompletionContext;
use crate::fzf::FzfConfig;

fn main() -> Result<()> {
    let config = Config::from_env();
    let readline_line = env::var("READLINE_LINE").unwrap_or_default();
    let readline_point: usize = env::var("READLINE_POINT")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .unwrap_or(0);

    if config.no_empty_cmd_completion && readline_line.trim().is_empty() {
        return Ok(());
    }

    show_loading();

    let parsed = parser::parse_shell_line(&readline_line, readline_point)?;
    let ctx = CompletionContext::from_parsed(&parsed, readline_line.clone(), readline_point);

    let spec = completion::resolve_compspec(&ctx.command)?;
    let mut candidates = completion::execute_completion(&spec, &ctx)?;

    candidates = quoting::apply_filter(&spec.filter, &candidates, &ctx.current_word)?;

    if spec.options.filenames || spec.options.default || spec.options.bashdefault {
        if spec.options.filenames || spec.options.dirnames || spec.options.default {
             candidates = quoting::mark_directories(candidates);
        }
    }

    let (candidates, nospace, _prefix) = quoting::find_common_prefix(
        &candidates, 
        ctx.current_word.len(),
        config.auto_common_prefix_part
    );

    let selected = if candidates.len() > 1 {
        let fzf_config = FzfConfig {
            height: config.fzf_tmux_height.unwrap_or_else(|| "40%".to_string()),
            prompt: config.prompt.clone(),
            completion_sep: config.completion_sep.clone(),
            options: shlex::split(&config.fzf_completion_opts).unwrap_or_default(),
            ..Default::default()
        };
        
        clear_loading();
        
        fzf::select_with_fzf(&candidates, &ctx.current_word, &fzf_config)?
    } else {
        clear_loading();
        candidates.first().cloned()
    };

    if let Some(mut completion) = selected {
        if spec.options.filenames || spec.options.default || spec.options.bashdefault {
             completion = quoting::quote_filename(&completion, true);
        }
        
        insert_completion(&readline_line, readline_point, &completion, nospace, &ctx.current_word)?;
    }

    Ok(())
}

fn show_loading() {
    let mut stderr = io::stderr();
    let _ = execute!(stderr, SavePosition);
    let _ = write!(stderr, "Loading matches ...");
    let _ = stderr.flush();
}

fn clear_loading() {
    let mut stderr = io::stderr();
    let _ = execute!(stderr, RestorePosition, Clear(ClearType::CurrentLine));
    let _ = stderr.flush();
}

fn insert_completion(
    line: &str,
    point: usize,
    completion: &str,
    nospace: bool,
    current_word: &str
) -> Result<()> {
    let prefix_len = current_word.len();
    
    let start_index = point.saturating_sub(prefix_len);
    
    let before = &line[..start_index];
    let after = &line[point..];
    
    let mut new_line = format!("{}{}{}", before, completion, after);
    let mut new_point = start_index + completion.len();
    
    if !nospace {
        if !completion.ends_with('/') {
            new_line.insert(new_point, ' ');
            new_point += 1;
        }
    }
    
    println!("READLINE_LINE='{}'", new_line);
    println!("READLINE_POINT={}", new_point);
    
    Ok(())
}
