use thiserror::Error;

#[derive(Error, Debug)]
pub enum BashError {
    #[error("Bash execution failed: {0}")]
    ExecutionError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(String),
}
