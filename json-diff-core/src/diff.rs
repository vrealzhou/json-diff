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

impl DiffType {
    /// Get the symbol representation of the diff type
    pub fn symbol(&self) -> &'static str {
        match self {
            DiffType::Added => "+",
            DiffType::Removed => "-",
            DiffType::Modified => "~",
            DiffType::ArrayItemChanged => "!",
            DiffType::ArrayReordered => "*",
            DiffType::Ignored => "?",
        }
    }

    /// Get the readable text representation of the diff type
    pub fn readable_text(&self) -> &'static str {
        match self {
            DiffType::Added => "ADDED",
            DiffType::Removed => "REMOVED",
            DiffType::Modified => "MODIFIED",
            DiffType::ArrayItemChanged => "ARRAY_ITEM_CHANGED",
            DiffType::ArrayReordered => "ARRAY_REORDERED",
            DiffType::Ignored => "IGNORED",
        }
    }

    /// Get a description of what the diff type means
    pub fn description(&self) -> &'static str {
        match self {
            DiffType::Added => "Property exists in target but not in source",
            DiffType::Removed => "Property exists in source but not in target",
            DiffType::Modified => "Property exists in both but with different values",
            DiffType::ArrayItemChanged => "Array item has changed",
            DiffType::ArrayReordered => "Array elements are reordered",
            DiffType::Ignored => "Property was ignored based on configuration",
        }
    }
}

impl fmt::Display for DiffType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
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
    /// Line number in the left/source file (if applicable)
    pub left_line: Option<usize>,
    /// Line number in the right/target file (if applicable)
    pub right_line: Option<usize>,
}

impl DiffEntry {
    /// Format the diff entry with readable text instead of symbols
    pub fn format_readable(&self) -> String {
        let mut result = String::new();

        // Add readable diff type
        result.push_str(&format!("[{}] {}", self.diff_type.readable_text(), self.path));

        // Add line number information if available
        match (self.left_line, self.right_line) {
            (Some(left), Some(right)) => result.push_str(&format!(" (L{}:L{})", left, right)),
            (Some(left), None) => result.push_str(&format!(" (L{})", left)),
            (None, Some(right)) => result.push_str(&format!(" (L{})", right)),
            (None, None) => {},
        }

        result.push_str(": ");

        // Add value information
        match self.diff_type {
            DiffType::Added => {
                result.push_str(&serde_json::to_string(&self.new_value).unwrap_or_default());
            }
            DiffType::Removed => {
                result.push_str(&serde_json::to_string(&self.old_value).unwrap_or_default());
            }
            DiffType::Modified | DiffType::ArrayItemChanged => {
                result.push_str(&format!("{} -> {}",
                    serde_json::to_string(&self.old_value).unwrap_or_default(),
                    serde_json::to_string(&self.new_value).unwrap_or_default()
                ));
            }
            DiffType::ArrayReordered => {
                result.push_str("[REORDERED]");
            }
            DiffType::Ignored => {
                result.push_str("[IGNORED]");
            }
        }

        result
    }
}

impl fmt::Display for DiffEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the diff type and path
        write!(f, "{} {}", self.diff_type, self.path)?;

        // Add line number information if available
        match (self.left_line, self.right_line) {
            (Some(left), Some(right)) => write!(f, " (L{}:L{})", left, right)?,
            (Some(left), None) => write!(f, " (L{})", left)?,
            (None, Some(right)) => write!(f, " (L{})", right)?,
            (None, None) => {},
        }

        write!(f, ": ")?;

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