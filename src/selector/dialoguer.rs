use crate::selector::{Selector, SelectorConfig, SelectorError, theme};
use dialoguer::console::Term;
use fuzzy_matcher::FuzzyMatcher;
use log::{debug, warn};

#[derive(Default)]
pub struct DialoguerSelector;

impl DialoguerSelector {
    pub fn new() -> Self {
        Default::default()
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
            "DialoguerSelector::select_one called with {} candidates (fuzzy={})",
            candidates.len(),
            config.fuzzy
        );

        if candidates.is_empty() {
            debug!("No candidates, returning None");
            return Ok(None);
        }

        if candidates.len() == 1 {
            debug!("Single candidate, returning: {}", candidates[0]);
            return Ok(Some(candidates[0].clone()));
        }

        let prompt = config
            .ctx
            .line
            .strip_suffix(current_word)
            .unwrap_or(&config.ctx.line);

        ctrlc::set_handler(|| {})?;

        let theme = &theme::CustomColorfulTheme::new();

        // Apply fuzzy filtering while preserving input order (history first, then carapace)
        let filtered_candidates: Vec<String> = if config.fuzzy && !current_word.is_empty() {
            let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
            let mut scored: Vec<(i64, usize, String)> = candidates
                .iter()
                .enumerate()
                .filter_map(|(idx, cand)| {
                    matcher
                        .fuzzy_match(cand, current_word)
                        .map(|score| (score, idx, cand.clone()))
                })
                .collect();

            // Sort by score (descending), but preserve original order for same scores
            scored.sort_by_key(|(score, idx, _)| (-score, *idx));

            scored.into_iter().map(|(_, _, cand)| cand).collect()
        } else {
            candidates.to_vec()
        };

        if filtered_candidates.is_empty() {
            debug!("No candidates after fuzzy filtering");
            return Ok(None);
        }

        debug!(
            "Filtered from {} to {} candidates",
            candidates.len(),
            filtered_candidates.len()
        );

        let select_result = dialoguer::Select::with_theme(theme)
            .report(false)
            .with_prompt(prompt)
            .default(0)
            .items(&filtered_candidates)
            .interact_opt();

        if select_result.is_err() {
            let _ = Term::stderr().show_cursor();
        }

        match select_result {
            Ok(Some(index)) => {
                let selected: &String = &filtered_candidates[index];
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
