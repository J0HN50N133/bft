pub mod history;

use crate::completion::{CompletionOptions, CompletionSpec};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BashError {
    #[error("Bash execution failed: {0}")]
    ExecutionError(String),
    #[error("Failed to parse completion spec: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(String),
}

pub fn query_complete(command: &str) -> Result<Option<CompletionSpec>, BashError> {
    let quoted_cmd = shlex::try_quote(command).map_err(|e| BashError::Other(e.to_string()))?;
    let output = Command::new("bash")
        .args(["-c", &format!("complete -p -- {}", quoted_cmd)])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| BashError::Other(format!("Failed to decode stdout as UTF-8: {}", e)))?;
    parse_compspec_output(&stdout)
}

pub fn execute_compgen(args: &[String]) -> Result<Vec<String>, BashError> {
    let quoted_args: Vec<String> = args
        .iter()
        .map(|a| {
            shlex::try_quote(a)
                .unwrap_or_else(|_| std::borrow::Cow::Owned(a.to_string()))
                .to_string()
        })
        .collect();

    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("compgen {}", quoted_args.join(" ")))
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| BashError::Other(format!("Failed to decode stdout as UTF-8: {}", e)))?;
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}

pub fn execute_completion_function(
    function: &str,
    _command: &str,
    _word: &str,
    _previous_word: Option<&str>,
    words: &[String],
    line: &str,
    point: usize,
) -> Result<Vec<String>, BashError> {
    let words_str = words
        .iter()
        .map(|w| shlex::try_quote(w).unwrap_or_else(|_| std::borrow::Cow::Owned(w.to_string())))
        .collect::<Vec<_>>()
        .join(" ");

    let script = format!(
        r#"
COMP_WORDS=({})
export COMP_CWORD={}
export COMP_LINE='{}'
export COMP_POINT={}
export COMP_KEY=""
export COMP_TYPE="9"

COMPREPLY=()
"{}" 2>/dev/null

for reply in "${{COMPREPLY[@]}}"; do
    echo "$reply"
done
"#,
        words_str,
        words.len().saturating_sub(1),
        line.replace("'", "'\\''"), // Escape single quotes for the bash string
        point,
        function
    );

    let output = Command::new("bash").arg("-c").arg(&script).output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| BashError::Other(format!("Failed to decode stdout as UTF-8: {}", e)))?;
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}

fn parse_compspec_output(output: &str) -> Result<Option<CompletionSpec>, BashError> {
    let args = shlex::split(output)
        .ok_or_else(|| BashError::ParseError("Failed to split output".to_string()))?;

    if args.first().map(|s| s.as_str()) != Some("complete") {
        return Ok(None);
    }

    let mut spec = CompletionSpec::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-F" => {
                i += 1;
                if i < args.len() {
                    spec.function = Some(args[i].clone());
                }
            }
            "-C" => {
                i += 1;
                if i < args.len() {
                    spec.command = Some(args[i].clone());
                }
            }
            "-W" => {
                i += 1;
                if i < args.len() {
                    spec.wordlist = Some(args[i].clone());
                }
            }
            "-P" => {
                i += 1;
                if i < args.len() {
                    spec.prefix = args[i].clone();
                }
            }
            "-S" => {
                i += 1;
                if i < args.len() {
                    spec.suffix = args[i].clone();
                }
            }
            "-X" => {
                i += 1;
                if i < args.len() {
                    spec.filter = Some(args[i].clone());
                }
            }
            "-o" => {
                i += 1;
                if i < args.len() {
                    parse_option(&args[i], &mut spec.options);
                }
            }
            _ => {}
        }
        i += 1;
    }

    Ok(Some(spec))
}

fn parse_option(opt: &str, options: &mut CompletionOptions) {
    match opt {
        "bashdefault" => options.bashdefault = true,
        "default" => options.default = true,
        "dirnames" => options.dirnames = true,
        "filenames" => options.filenames = true,
        "noquote" => options.noquote = true,
        "nospace" => options.nospace = true,
        "plusdirs" => options.plusdirs = true,
        "nosort" => options.nosort = true,
        _ => {}
    }
}
