use crate::completion::CompletionEntry;
use glob::Pattern;
use shellexpand;
use shlex;
use std::path::Path;

pub fn quote_filename(path: &str, is_filename: bool) -> String {
    if !is_filename {
        return path.to_string();
    }

    if path.starts_with('~') {
        if let Some(idx) = path.find('/') {
            let (tilde_part, rest) = path.split_at(idx + 1);
            format!("{}{}", tilde_part, shell_quote(rest))
        } else {
            path.to_string()
        }
    } else {
        shell_quote(path)
    }
}

fn shell_quote(s: &str) -> String {
    shlex::try_quote(s)
        .unwrap_or_else(|_| s.to_string().into())
        .to_string()
}

pub fn mark_directories(candidates: Vec<CompletionEntry>) -> Vec<CompletionEntry> {
    candidates
        .into_iter()
        .map(|mut entry| {
            let expanded = shellexpand::tilde(&entry.value);
            let unescaped = unescape_filename(&expanded);

            if Path::new(&unescaped).is_dir() && !entry.value.ends_with('/') {
                entry.value = format!("{}/", entry.value);
            }
            entry
        })
        .collect()
}

fn unescape_filename(s: &str) -> String {
    brush_parser::unquote_str(s).to_string()
}

pub fn find_common_prefix(
    candidates: &[CompletionEntry],
    input_len: usize,
    auto_common_prefix_part: bool,
) -> (Vec<CompletionEntry>, bool, String) {
    if candidates.is_empty() {
        return (vec![], false, String::new());
    }

    let values: Vec<String> = candidates.iter().map(|c| c.value.clone()).collect();
    let prefix = find_longest_common_prefix(&values);
    let prefix_len = prefix.len();

    if prefix_len > input_len {
        let all_match = candidates.iter().all(|c| c.value.len() == prefix_len);

        if all_match || auto_common_prefix_part {
            let nospace = candidates.len() > 1;
            // Create a synthetic entry for the prefix.
            // Using the kind of the first candidate is a heuristic.
            // If candidates have mixed providers, this might be misleading,
            // but for a common prefix it often doesn't matter as much.
            let kind = candidates[0].kind;
            return (
                vec![CompletionEntry::new(prefix.clone(), kind)],
                nospace,
                prefix,
            );
        }
    }

    (candidates.to_vec(), false, String::new())
}

fn find_longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }

    let mut prefix = strings[0].clone();

    for s in &strings[1..] {
        while !s.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                return String::new();
            }
        }
    }

    prefix
}

pub fn apply_filter(
    filter: &Option<String>,
    candidates: &[CompletionEntry],
    current_word: &str,
) -> Result<Vec<CompletionEntry>, glob::PatternError> {
    let Some(pattern_str) = filter else {
        return Ok(candidates.to_vec());
    };

    let pattern_str = pattern_str.replace('&', current_word);

    let invert = pattern_str.starts_with('!');
    let glob_pattern = if invert {
        &pattern_str[1..]
    } else {
        &pattern_str
    };

    let pattern = Pattern::new(glob_pattern)?;

    let result: Vec<CompletionEntry> = candidates
        .iter()
        .filter(|c| {
            let matches = pattern.matches(&c.value);
            if invert { !matches } else { matches }
        })
        .cloned()
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::completion::ProviderKind;

    use super::*;

    #[test]
    fn test_quote_filename() {
        assert_eq!(quote_filename("foo bar", true), "'foo bar'");
        assert_eq!(quote_filename("~user/foo bar", true), "~user/'foo bar'");
        assert_eq!(quote_filename("simple", true), "simple");
    }

    #[test]
    fn test_common_prefix() {
        let candidates = [
            CompletionEntry::new("file1".to_string(), ProviderKind::Bash),
            CompletionEntry::new("file2".to_string(), ProviderKind::Bash),
        ];
        let (res, _nospace, prefix) = find_common_prefix(&candidates, 0, false);
        // auto_common_prefix_part=false, so we don't complete partial prefix "file"
        // We expect original candidates and no prefix returned
        assert_eq!(prefix, "");
        assert_eq!(res.len(), 2);

        let (res, nospace, prefix) = find_common_prefix(&candidates, 0, true);
        // With auto_common_prefix_part=true, we complete to "file"
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].value, "file");
        assert_eq!(prefix, "file");
        assert!(nospace);
    }

    #[test]
    fn test_filter() {
        let candidates = [
            CompletionEntry::new("foo".to_string(), ProviderKind::Bash),
            CompletionEntry::new("bar".to_string(), ProviderKind::Bash),
            CompletionEntry::new("baz".to_string(), ProviderKind::Bash),
        ];
        let filtered = apply_filter(&Some("!b*".to_string()), &candidates, "").unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].value, "foo");

        let filtered = apply_filter(&Some("*z".to_string()), &candidates, "").unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].value, "baz");
    }
}
