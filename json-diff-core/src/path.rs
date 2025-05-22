use regex::Regex;
use crate::error::JsonDiffError;

/// Represents a JSON path for matching elements
#[derive(Debug, Clone)]
pub struct JsonPath {
    path: String,
    regex: Option<Regex>,
}

impl JsonPath {
    /// Create a new JSON path
    pub fn new(path: &str) -> Result<Self, JsonDiffError> {
        Ok(Self {
            path: path.to_string(),
            regex: None,
        })
    }
    
    /// Create a JSON path with regex pattern matching
    pub fn with_regex(path: &str, pattern: &str) -> Result<Self, JsonDiffError> {
        let regex = Regex::new(pattern).map_err(JsonDiffError::from)?;
        
        Ok(Self {
            path: path.to_string(),
            regex: Some(regex),
        })
    }
    
    /// Check if this path matches the given path string
    pub fn matches(&self, path: &str) -> bool {
        if let Some(regex) = &self.regex {
            regex.is_match(path)
        } else {
            self.path == path
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_path_match() {
        let path = JsonPath::new("$.user.name").unwrap();
        
        assert!(path.matches("$.user.name"));
        assert!(!path.matches("$.user.age"));
        assert!(!path.matches("$.user"));
    }

    #[test]
    fn test_path_with_regex() {
        let path = JsonPath::with_regex("$", "^\\$\\.user\\.[a-z]+$").unwrap();
        
        assert!(path.matches("$.user.name"));
        assert!(path.matches("$.user.age"));
        assert!(!path.matches("$.user"));
        assert!(!path.matches("$.user.name.first"));
    }

    #[test]
    fn test_array_index_path() {
        let path = JsonPath::new("$.users[0].name").unwrap();
        
        assert!(path.matches("$.users[0].name"));
        assert!(!path.matches("$.users[1].name"));
    }

    #[test]
    fn test_wildcard_path() {
        let path = JsonPath::with_regex("$", "^\\$\\.users\\[\\d+\\]\\.name$").unwrap();
        
        assert!(path.matches("$.users[0].name"));
        assert!(path.matches("$.users[1].name"));
        assert!(path.matches("$.users[42].name"));
        assert!(!path.matches("$.users.name"));
    }
}
