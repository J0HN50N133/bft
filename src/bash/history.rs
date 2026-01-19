use log::debug;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: Option<String>,
}

pub fn get_history_file() -> Option<PathBuf> {
    // Check HISTFILE environment variable first
    if let Ok(histfile) = env::var("HISTFILE")
        && !histfile.is_empty()
    {
        debug!("[history] Using HISTFILE env: {}", histfile);
        return Some(PathBuf::from(histfile));
    }

    // Default to ~/.bash_history
    if let Ok(home) = env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".bash_history");
        debug!("[history] Using default bash_history: {}", path.display());
        return Some(path);
    }

    debug!("[history] No HISTFILE found");
    None
}

pub fn read_history(limit: Option<usize>) -> Vec<HistoryEntry> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    if let Some(histfile) = get_history_file() {
        debug!("[history] Checking history file: {}", histfile.display());

        if !histfile.exists() {
            debug!("[history] History file does not exist");
            return entries;
        }

        if let Ok(file) = File::open(&histfile) {
            let reader = BufReader::new(file);
            #[allow(clippy::lines_filter_map_ok)]
            let total_lines: usize = reader.lines().map_while(Result::ok).count();
            debug!("[history] Total lines in history file: {}", total_lines);

            // Re-open file for reading
            if let Ok(file) = File::open(&histfile) {
                let reader = BufReader::new(file);
                #[allow(clippy::lines_filter_map_ok)]
                for line in reader.lines().map_while(Result::ok) {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // Skip duplicates and entries starting with space (ignored by bash)
                        if !trimmed.starts_with(' ') && seen.insert(trimmed.to_string()) {
                            entries.push(HistoryEntry {
                                command: trimmed.to_string(),
                                timestamp: None,
                            });
                            if let Some(limit) = limit
                                && entries.len() >= limit
                            {
                                break;
                            }
                        }
                    }
                }
            }

            debug!(
                "[history] Read {} unique entries (limit: {:?})",
                entries.len(),
                limit
            );
        }
    } else {
        debug!("[history] No history file available");
    }

    entries
}

/// Get unique command names from history (first word of each command)
pub fn get_history_commands(limit: Option<usize>) -> Vec<String> {
    let history = read_history(limit);
    let mut commands: Vec<String> = history
        .into_iter()
        .filter_map(|entry| {
            let first_word = entry.command.split_whitespace().next()?;
            if first_word.is_empty() {
                None
            } else {
                Some(first_word.to_string())
            }
        })
        .collect();

    commands.sort();
    commands.dedup();
    commands
}

/// Filter history commands by prefix
pub fn filter_history_commands(prefix: &str, limit: Option<usize>) -> Vec<String> {
    let commands = get_history_commands(None);
    let prefix_lower = prefix.to_lowercase();
    let total_commands = commands.len();

    let filtered: Vec<String> = commands
        .into_iter()
        .filter(|cmd| cmd.to_lowercase().starts_with(&prefix_lower))
        .take(limit.unwrap_or(usize::MAX))
        .collect();

    debug!(
        "[history] filter_history_commands(prefix='{}'): {} unique commands, {} matched",
        prefix,
        total_commands,
        filtered.len()
    );

    filtered
}

/// Get full command lines from history that match the prefix (starts with)
pub fn get_matching_history_commands(prefix: &str, limit: Option<usize>) -> Vec<String> {
    let history = read_history(limit);
    let prefix_lower = prefix.to_lowercase();

    let filtered: Vec<String> = history
        .into_iter()
        .filter(|entry| entry.command.to_lowercase().starts_with(&prefix_lower))
        .map(|entry| entry.command)
        .take(limit.unwrap_or(usize::MAX))
        .collect();

    debug!(
        "[history] get_matching_history_commands(prefix='{}'): {} matched",
        prefix,
        filtered.len()
    );

    filtered
}

/// Get full command lines from history that contain the substring, take the last [limit] entries.
/// If limit is none, all history entries will be returned
pub fn get_history_commands_by_prefix(substr: &str, limit: Option<usize>) -> Vec<String> {
    if substr.is_empty() {
        return Vec::new();
    }

    let history = read_history(None);
    let history_len = history.len();

    let filtered: Vec<String> = history
        .into_iter()
        .filter(|entry| entry.command.to_lowercase().starts_with(substr))
        .map(|entry| entry.command)
        .rev()
        .take(limit.unwrap_or(history_len))
        .collect();

    debug!(
        "[history] get_history_commands_by_substring(substr='{}'): {} matched from {} total",
        substr,
        filtered.len(),
        history_len
    );

    filtered
}

/// Get matching history entries and extract the second word (subcommand)
/// For example, with "git checkout feature" and prefix "git", returns ["checkout"]
pub fn get_history_subcommands(
    prefix: &str,
    current_word: &str,
    limit: Option<usize>,
) -> Vec<String> {
    if current_word.is_empty() {
        debug!(
            "[history] get_history_subcommands(prefix='{}', word=''): empty word, returning",
            prefix
        );
        return Vec::new();
    }

    let history = read_history(limit);
    let cmd_prefix_lower = prefix.to_lowercase();
    let word_lower = current_word.to_lowercase();

    let mut seen = std::collections::HashSet::new();
    let mut results: Vec<String> = Vec::new();

    for entry in history {
        let cmd_lower = entry.command.to_lowercase();
        if cmd_lower.starts_with(&cmd_prefix_lower) {
            // Extract second word (subcommand)
            if let Some(second_word) = entry.command.split_whitespace().nth(1)
                && second_word.to_lowercase().starts_with(&word_lower)
                && seen.insert(second_word.to_string())
            {
                results.push(second_word.to_string());
            }
        }
    }

    debug!(
        "[history] get_history_subcommands(prefix='{}', word='{}'): {} matched: {:?}",
        prefix,
        current_word,
        results.len(),
        results
    );

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_history_commands() {
        // Create a temp history file
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "ls -la").unwrap();
        writeln!(temp, "git status").unwrap();
        writeln!(temp, "cat file.txt").unwrap();
        writeln!(temp, "ls -lh").unwrap(); // Duplicate command, should be deduped

        // Set the temp file as HISTFILE
        unsafe { env::set_var("HISTFILE", temp.path()) };

        let commands = get_history_commands(None);
        assert!(commands.contains(&"ls".to_string()));
        assert!(commands.contains(&"git".to_string()));
        assert!(commands.contains(&"cat".to_string()));
        assert_eq!(commands.len(), 3);

        // Clean up
        unsafe { env::remove_var("HISTFILE") };
    }

    #[test]
    fn test_filter_history_commands() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "git status").unwrap();
        writeln!(temp, "git log").unwrap();
        writeln!(temp, "git checkout").unwrap();
        writeln!(temp, "ls -la").unwrap();

        unsafe { env::set_var("HISTFILE", temp.path()) };

        // filter_history_commands returns unique command names (first word)
        let filtered = filter_history_commands("git", None);
        // Should contain "git" since it's a unique command name
        assert!(filtered.contains(&"git".to_string()));

        let filtered = filter_history_commands("ls", None);
        // Should contain "ls" since it's a unique command name
        assert!(filtered.contains(&"ls".to_string()));
        // At least 1 (from our test history)
        assert!(!filtered.is_empty());

        unsafe { env::remove_var("HISTFILE") };
    }

    #[test]
    fn test_get_matching_history_commands() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "echo hello").unwrap();
        writeln!(temp, "echo world").unwrap();
        writeln!(temp, "ls -la").unwrap();

        unsafe { env::set_var("HISTFILE", temp.path()) };

        let matches = get_matching_history_commands("echo", None);
        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&"echo hello".to_string()));
        assert!(matches.contains(&"echo world".to_string()));

        unsafe { env::remove_var("HISTFILE") };
    }

    #[test]
    fn test_get_history_subcommands() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "git checkout main").unwrap();
        writeln!(temp, "git checkout feature").unwrap();
        writeln!(temp, "git status").unwrap();
        writeln!(temp, "git log").unwrap();
        writeln!(temp, "ls -la").unwrap();

        unsafe { env::set_var("HISTFILE", temp.path()) };

        // Get subcommands for "git che" - should return ["checkout"]
        let subcommands = get_history_subcommands("git", "che", None);
        assert!(subcommands.contains(&"checkout".to_string()));

        // Get subcommands for "git sta" - should return ["status"]
        let subcommands = get_history_subcommands("git", "sta", None);
        assert!(subcommands.contains(&"status".to_string()));

        // Empty prefix should return nothing
        let subcommands = get_history_subcommands("git", "", None);
        assert!(subcommands.is_empty());

        unsafe { env::remove_var("HISTFILE") };
    }
}
