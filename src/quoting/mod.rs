use std::path::Path;
use shellexpand;
use glob::Pattern;
use shlex;

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
    shlex::try_quote(s).unwrap_or_else(|_| s.to_string().into()).to_string()
}

pub fn mark_directories(candidates: Vec<String>) -> Vec<String> {
    candidates.into_iter()
        .map(|path| {
            let expanded = shellexpand::tilde(&path);
            let unescaped = unescape_filename(&expanded);
            
            if Path::new(&unescaped).is_dir() {
                if !path.ends_with('/') {
                    format!("{}/", path)
                } else {
                    path
                }
            } else {
                path
            }
        })
        .collect()
}

fn unescape_filename(s: &str) -> String {
    brush_parser::unquote_str(s).to_string()
}

pub fn find_common_prefix(
    candidates: &[String], 
    input_len: usize,
    auto_common_prefix_part: bool
) -> (Vec<String>, bool, String) {
    if candidates.is_empty() {
        return (vec![], false, String::new());
    }
    
    let prefix = find_longest_common_prefix(candidates);
    let prefix_len = prefix.len();
    
    if prefix_len > input_len {
        let all_match = candidates.iter().all(|c| c.len() == prefix_len);
        
        if all_match || auto_common_prefix_part {
            let nospace = candidates.len() > 1;
            return (vec![prefix.clone()], nospace, prefix);
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
    candidates: &[String],
    current_word: &str
) -> Result<Vec<String>, glob::PatternError> {
    let Some(pattern_str) = filter else {
        return Ok(candidates.to_vec());
    };
    
    let pattern_str = pattern_str.replace('&', current_word);
    
    let invert = pattern_str.starts_with('!');
    let glob_pattern = if invert { &pattern_str[1..] } else { &pattern_str };
    
    let pattern = Pattern::new(glob_pattern)?;
    
    let result: Vec<String> = candidates.iter()
        .filter(|c| {
            let matches = pattern.matches(c);
            if invert { !matches } else { matches }
        })
        .cloned()
        .collect();
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_filename() {
        assert_eq!(quote_filename("foo bar", true), "'foo bar'");
        assert_eq!(quote_filename("~user/foo bar", true), "~user/'foo bar'");
        assert_eq!(quote_filename("simple", true), "simple");
    }

    #[test]
    fn test_common_prefix() {
        let candidates = vec!["file1", "file2"];
        let (res, _nospace, prefix) = find_common_prefix(&candidates.iter().map(|s| s.to_string()).collect::<Vec<_>>(), 0, false);
        // auto_common_prefix_part=false, so we don't complete partial prefix "file"
        // We expect original candidates and no prefix returned
        assert_eq!(prefix, "");
        assert_eq!(res.len(), 2);
        
        let (res, nospace, prefix) = find_common_prefix(&candidates.iter().map(|s| s.to_string()).collect::<Vec<_>>(), 0, true);
        // With auto_common_prefix_part=true, we complete to "file"
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], "file");
        assert_eq!(prefix, "file");
        assert!(nospace);
    }

    #[test]
    fn test_filter() {
        let candidates = vec!["foo", "bar", "baz"];
        let filtered = apply_filter(&Some("!b*".to_string()), &candidates.iter().map(|s| s.to_string()).collect::<Vec<_>>(), "").unwrap();
        assert_eq!(filtered, vec!["foo"]);
        
        let filtered = apply_filter(&Some("*z".to_string()), &candidates.iter().map(|s| s.to_string()).collect::<Vec<_>>(), "").unwrap();
        assert_eq!(filtered, vec!["baz"]);
    }
}
