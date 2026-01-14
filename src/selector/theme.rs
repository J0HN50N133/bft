use std::fmt;

use brush_parser::prompt;
use dialoguer::{
    console::style,
    theme::{ColorfulTheme, Theme},
};
use fuzzy_matcher::skim::SkimMatcherV2;

pub struct CustomSimpleTheme;
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

pub struct CustomColorfulTheme(ColorfulTheme);

impl CustomColorfulTheme {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl Theme for CustomColorfulTheme {
    fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.0.format_prompt(f, prompt)
    }

    fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
        self.0.format_error(f, err)
    }

    fn format_confirm_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<bool>,
    ) -> fmt::Result {
        self.0.format_confirm_prompt(f, prompt, default)
    }

    fn format_confirm_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: Option<bool>,
    ) -> fmt::Result {
        self.0.format_confirm_prompt_selection(f, prompt, selection)
    }

    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        self.0.format_input_prompt(f, prompt, default)
    }

    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.0.format_input_prompt_selection(f, prompt, sel)
    }

    fn format_password_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.0.format_password_prompt(f, prompt)
    }

    fn format_password_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
    ) -> fmt::Result {
        self.0.format_password_prompt_selection(f, prompt)
    }

    fn format_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.0.format_select_prompt(f, prompt)
    }

    fn format_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.0.format_select_prompt_selection(f, prompt, sel)
    }

    fn format_multi_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.0.format_multi_select_prompt(f, prompt)
    }

    fn format_sort_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.0.format_sort_prompt(f, prompt)
    }

    fn format_multi_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        self.0
            .format_multi_select_prompt_selection(f, prompt, selections)
    }

    fn format_sort_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        self.0.format_sort_prompt_selection(f, prompt, selections)
    }

    fn format_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        self.0.format_select_prompt_item(f, text, active)
    }

    fn format_multi_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        checked: bool,
        active: bool,
    ) -> fmt::Result {
        self.0
            .format_multi_select_prompt_item(f, text, checked, active)
    }

    fn format_sort_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        picked: bool,
        active: bool,
    ) -> fmt::Result {
        self.0.format_sort_prompt_item(f, text, picked, active)
    }

    fn format_fuzzy_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
        highlight_matches: bool,
        matcher: &SkimMatcherV2,
        search_term: &str,
    ) -> fmt::Result {
        self.0.format_fuzzy_select_prompt_item(
            f,
            text,
            active,
            highlight_matches,
            matcher,
            search_term,
        )
    }

    fn format_fuzzy_select_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        search_term: &str,
        bytes_pos: usize,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {}",
                self.0.prompt_prefix,
                self.0.prompt_style.apply_to(prompt)
            )?;
        }

        let (st_head, remaining) = search_term.split_at(bytes_pos);
        let mut chars = remaining.chars();
        let chr = chars.next().unwrap_or(' ');
        let st_cursor = self.0.fuzzy_cursor_style.apply_to(chr);
        let st_tail = chars.as_str();

        write!(f, "{st_head}{st_cursor}{st_tail}",)
    }
}
