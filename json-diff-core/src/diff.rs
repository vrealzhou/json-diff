use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::fmt;

/// Types of differences that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
    ArrayItemChanged,
    ArrayReordered,
    Ignored,
}

impl fmt::Display for DiffType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiffType::Added => write!(f, "+"),
            DiffType::Removed => write!(f, "-"),
            DiffType::Modified => write!(f, "~"),
            DiffType::ArrayItemChanged => write!(f, "!"),
            DiffType::ArrayReordered => write!(f, "*"),
            DiffType::Ignored => write!(f, "?"),
        }
    }
}

/// A single difference entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffEntry {
    /// Type of difference
    pub diff_type: DiffType,
    /// JSON path to the difference
    pub path: String,
    /// Original value (if applicable)
    pub old_value: Option<serde_json::Value>,
    /// New value (if applicable)
    pub new_value: Option<serde_json::Value>,
}

impl fmt::Display for DiffEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: ", self.diff_type, self.path)?;
        
        match self.diff_type {
            DiffType::Added => {
                write!(f, "{}", serde_json::to_string(&self.new_value).unwrap_or_default())
            }
            DiffType::Removed => {
                write!(f, "{}", serde_json::to_string(&self.old_value).unwrap_or_default())
            }
            DiffType::Modified | DiffType::ArrayItemChanged => {
                write!(f, "{} -> {}", 
                    serde_json::to_string(&self.old_value).unwrap_or_default(),
                    serde_json::to_string(&self.new_value).unwrap_or_default()
                )
            }
            DiffType::ArrayReordered => {
                write!(f, "[REORDERED]")
            }
            DiffType::Ignored => {
                write!(f, "[IGNORED]")
            }
        }
    }
}

/// Complete diff result between two JSON documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// Path to the left (original) file
    pub left_file: Option<PathBuf>,
    /// Path to the right (modified) file
    pub right_file: Option<PathBuf>,
    /// When the diff was generated
    pub timestamp: DateTime<Utc>,
    /// List of differences
    pub entries: Vec<DiffEntry>,
}

impl fmt::Display for DiffResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DIFF-JSON v1")?;
        
        if let Some(left) = &self.left_file {
            writeln!(f, "LEFT: {}", left.display())?;
        }
        
        if let Some(right) = &self.right_file {
            writeln!(f, "RIGHT: {}", right.display())?;
        }
        
        writeln!(f, "TIMESTAMP: {}", self.timestamp.to_rfc3339())?;
        writeln!(f)?;
        
        for entry in &self.entries {
            writeln!(f, "{}", entry)?;
        }
        
        Ok(())
    }
}