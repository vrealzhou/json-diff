use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonDiffError {
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
    
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid JSON path: {0}")]
    InvalidPath(String),
    
    #[error("Invalid regex pattern: {0}")]
    RegexError(#[from] regex::Error),
}