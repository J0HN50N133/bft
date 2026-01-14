use std::fmt;

use crate::selector::{Selector, SelectorConfig, SelectorError};
use dialoguer::{console::Term, theme::Theme};
use log::{debug, warn};

#[derive(Default)]
pub struct DialoguerSelector;

impl DialoguerSelector {
    pub fn new() -> Self {
        Default::default()
    }
}

struct CustomSimpleTheme;
impl Theme for CustomSimpleTheme {
    fn format_fuzzy_select_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        search_term: &str,
        bytes_pos: usize,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(f, "{prompt}")?;
        }

        let (st_head, st_tail) = search_term.split_at(bytes_pos);
        write!(f, "{st_head}|{st_tail}")
    }
}

impl Selector for DialoguerSelector {
    fn select_one(
        &self,
        candidates: &[String],
        current_word: &str,
        config: &SelectorConfig,
    ) -> Result<Option<String>, SelectorError> {
        debug!(
            "DialoguerSelector::select_one called with {} candidates",
            candidates.len()
        );

        if candidates.is_empty() {
            debug!("No candidates, returning None");
            return Ok(None);
        }

        if candidates.len() == 1 {
            debug!("Single candidate, returning: {}", candidates[0]);
            return Ok(Some(candidates[0].clone()));
        }

        let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();

        let prompt = config
            .ctx
            .line
            .strip_suffix(current_word)
            .unwrap_or(&config.ctx.line);

        ctrlc::set_handler(|| {})?;

        let theme = &CustomSimpleTheme;

        let result = dialoguer::FuzzySelect::with_theme(theme)
            .report(false)
            .with_initial_text(current_word)
            .with_prompt(prompt)
            .default(0)
            .items(&candidate_refs)
            .interact_opt();

        if result.is_err() {
            let _ = Term::stderr().show_cursor();
        }

        match result {
            Ok(Some(index)) => {
                let selected: &String = &candidates[index];
                debug!("Selected candidate: {}", selected);
                Ok(Some(selected.clone()))
            }
            Ok(None) => {
                debug!("User cancelled selection");
                Ok(None)
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("interrupted") || error_msg.contains("Interrupted") {
                    debug!("Selection interrupted by user (Ctrl-C)");
                    Ok(None)
                } else {
                    warn!("Dialoguer selection failed: {}", e);
                    Err(SelectorError::ExecutionError(format!(
                        "Dialoguer selection failed: {}",
                        e
                    )))
                }
            }
        }
    }
}
