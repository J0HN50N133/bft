use crate::{
    completion::CompletionContext,
    selector::{Selector, SelectorConfig, SelectorError},
};
use dialoguer::theme::ColorfulTheme;
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
            "DialoguerSelector::select_one called with {} candidates",
            candidates.len()
        );

        if candidates.is_empty() {
            debug!("No candidates, returning None");
            return Ok(None);
        }

        // If only one candidate, return it immediately
        if candidates.len() == 1 {
            debug!("Single candidate, returning: {}", candidates[0]);
            return Ok(Some(candidates[0].clone()));
        }

        // Convert candidates to &str slices for dialoguer
        let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();

        // Try to create a fuzzy selection
        let prompt = config
            .ctx
            .line
            .strip_suffix(current_word)
            .unwrap_or(&config.ctx.line);
        match dialoguer::FuzzySelect::with_theme(&ColorfulTheme::default())
            .report(false)
            .with_initial_text(current_word)
            .with_prompt(prompt)
            .default(0)
            .items(&candidate_refs)
            .interact_opt()
        {
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
                warn!("Dialoguer selection failed: {}", e);
                Err(SelectorError::ExecutionError(format!(
                    "Dialoguer selection failed: {}",
                    e
                )))
            }
        }
    }
}
