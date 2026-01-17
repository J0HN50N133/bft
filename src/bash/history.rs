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
    if let Ok(histfile) = env::var("HISTFILE") {
        if !histfile.is_empty() {
            return Some(PathBuf::from(histfile));
        }
    }

    // Default to ~/.bash_history
    if let Ok(home) = env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".bash_history");
        return Some(path);
    }

    None
}

pub fn read_history(limit: Option<usize>) -> Vec<HistoryEntry> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    if let Some(histfile) = get_history_file() {
        if histfile.exists() {
            if let Ok(file) = File::open(&histfile) {
                let reader = BufReader::new(file);
                for line in reader.lines().flatten() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // Skip duplicates and entries starting with space (ignored by bash)
                        if !trimmed.starts_with(' ') && seen.insert(trimmed.to_string()) {
                            entries.push(HistoryEntry {
                                command: trimmed.to_string(),
                                timestamp: None,
                            });
                            if let Some(limit) = limit {
                                if entries.len() >= limit {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
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

    commands
        .into_iter()
        .filter(|cmd| cmd.to_lowercase().starts_with(&prefix_lower))
        .take(limit.unwrap_or(usize::MAX))
        .collect()
}

/// Get full command lines from history that match the prefix
pub fn get_matching_history_commands(prefix: &str, limit: Option<usize>) -> Vec<String> {
    let history = read_history(limit);
    let prefix_lower = prefix.to_lowercase();

    history
        .into_iter()
        .filter(|entry| entry.command.to_lowercase().starts_with(&prefix_lower))
        .map(|entry| entry.command)
        .take(limit.unwrap_or(usize::MAX))
        .collect()
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
        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains(&"ls".to_string()));

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
}
