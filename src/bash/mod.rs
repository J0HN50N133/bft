use thiserror::Error;
use std::process::Command;
use crate::completion::{CompletionSpec, CompletionOptions};

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
    let output = Command::new("bash")
        .args(["-c", &format!("complete -p -- {}", shlex::quote(command))])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_compspec_output(&stdout)
}

pub fn execute_compgen(args: &[String]) -> Result<Vec<String>, BashError> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("compgen {}", args.join(" ")))
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}

fn parse_compspec_output(output: &str) -> Result<Option<CompletionSpec>, BashError> {
    let args = shlex::split(output).ok_or_else(|| BashError::ParseError("Failed to split output".to_string()))?;
    
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
