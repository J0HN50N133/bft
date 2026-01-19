use crate::selector::{Selector, SelectorConfig, SelectorError, theme};
use dialoguer::console::Term;
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

        if candidates.is_empty() {
            debug!("No candidates after fuzzy filtering");
            return Ok(None);
        }

        debug!(
            "Filtered from {} to {} candidates",
            candidates.len(),
            candidates.len()
        );

        let select_result = dialoguer::FuzzySelect::with_theme(theme)
            .report(false)
            .with_initial_text(current_word)
            .with_prompt(prompt)
            .default(0)
            .items(candidates)
            .interact_opt();

        if select_result.is_err() {
            let _ = Term::stderr().show_cursor();
        }

        match select_result {
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
