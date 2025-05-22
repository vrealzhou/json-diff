use std::path::Path;
use std::fs::File;
use std::io::Read;
use chrono::Utc;
use serde_json::{Value, Map};

use crate::diff::{DiffEntry, DiffType, DiffResult};
use crate::path::JsonPath;
use crate::error::JsonDiffError;

/// Options for JSON comparison
#[derive(Debug, Clone)]
pub struct CompareOptions {
    /// Paths to ignore during comparison
    pub ignore_paths: Vec<JsonPath>,
    /// Paths to arrays that should be compared without considering order
    pub unordered_arrays: Vec<JsonPath>,
    /// Whether to show nested differences in unordered arrays
    pub show_nested_differences: bool,
    /// Whether to identify specific different items in arrays rather than marking whole arrays as different
    pub identify_array_item_changes: bool,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            ignore_paths: Vec::new(),
            unordered_arrays: Vec::new(),
            show_nested_differences: false,
            identify_array_item_changes: true,
        }
    }
}

/// Compare two JSON files and generate a diff result
pub fn compare_files<P: AsRef<Path>>(
    left_path: P,
    right_path: P,
    options: &CompareOptions
) -> Result<DiffResult, JsonDiffError> {
    let mut left_file = File::open(left_path.as_ref())?;
    let mut right_file = File::open(right_path.as_ref())?;

    let mut left_content = String::new();
    let mut right_content = String::new();

    left_file.read_to_string(&mut left_content)?;
    right_file.read_to_string(&mut right_content)?;

    let left_json: Value = serde_json::from_str(&left_content)?;
    let right_json: Value = serde_json::from_str(&right_content)?;

    let mut result = compare_json(&left_json, &right_json, options)?;
    result.left_file = Some(left_path.as_ref().to_path_buf());
    result.right_file = Some(right_path.as_ref().to_path_buf());

    Ok(result)
}

/// Compare two JSON values and generate a diff result
pub fn compare_json(
    left: &Value,
    right: &Value,
    options: &CompareOptions
) -> Result<DiffResult, JsonDiffError> {
    let mut entries = Vec::new();

    compare_values(left, right, "$", &mut entries, options)?;

    let result = DiffResult {
        left_file: None,
        right_file: None,
        timestamp: Utc::now(),
        entries,
    };

    Ok(result)
}

fn compare_values(
    left: &Value,
    right: &Value,
    path: &str,
    entries: &mut Vec<DiffEntry>,
    options: &CompareOptions,
) -> Result<(), JsonDiffError> {
    // Check if this path should be ignored
    if options.ignore_paths.iter().any(|p| p.matches(path)) {
        entries.push(DiffEntry {
            diff_type: DiffType::Ignored,
            path: path.to_string(),
            old_value: None,
            new_value: None,
        });
        return Ok(());
    }

    match (left, right) {
        (Value::Object(left_obj), Value::Object(right_obj)) => {
            compare_objects(left_obj, right_obj, path, entries, options)?;
        }
        (Value::Array(left_arr), Value::Array(right_arr)) => {
            // Check if this array should be compared without order
            let unordered = options.unordered_arrays.iter().any(|p| p.matches(path));
            compare_arrays(left_arr, right_arr, path, entries, options, unordered)?;
        }
        _ if left == right => {
            // Values are equal, no diff needed
        }
        _ => {
            // Values are different
            entries.push(DiffEntry {
                diff_type: DiffType::Modified,
                path: path.to_string(),
                old_value: Some(left.clone()),
                new_value: Some(right.clone()),
            });
        }
    }

    Ok(())
}

fn compare_objects(
    left: &Map<String, Value>,
    right: &Map<String, Value>,
    path: &str,
    entries: &mut Vec<DiffEntry>,
    options: &CompareOptions,
) -> Result<(), JsonDiffError> {
    // Find keys that exist in left but not in right
    for key in left.keys() {
        if !right.contains_key(key) {
            let key_path = format!("{}.{}", path, key);

            // Check if this path should be ignored
            if options.ignore_paths.iter().any(|p| p.matches(&key_path)) {
                entries.push(DiffEntry {
                    diff_type: DiffType::Ignored,
                    path: key_path,
                    old_value: None,
                    new_value: None,
                });
                continue;
            }

            entries.push(DiffEntry {
                diff_type: DiffType::Removed,
                path: key_path,
                old_value: Some(left[key].clone()),
                new_value: None,
            });
        }
    }

    // Find keys that exist in right but not in left, or compare values for common keys
    for key in right.keys() {
        let key_path = format!("{}.{}", path, key);

        // Check if this path should be ignored
        if options.ignore_paths.iter().any(|p| p.matches(&key_path)) {
            entries.push(DiffEntry {
                diff_type: DiffType::Ignored,
                path: key_path,
                old_value: None,
                new_value: None,
            });
            continue;
        }

        if !left.contains_key(key) {
            entries.push(DiffEntry {
                diff_type: DiffType::Added,
                path: key_path,
                old_value: None,
                new_value: Some(right[key].clone()),
            });
        } else {
            compare_values(&left[key], &right[key], &key_path, entries, options)?;
        }
    }

    Ok(())
}

fn compare_arrays(
    left: &[Value],
    right: &[Value],
    path: &str,
    entries: &mut Vec<DiffEntry>,
    options: &CompareOptions,
    unordered: bool,
) -> Result<(), JsonDiffError> {
    if unordered {
        // For unordered comparison, we check if the arrays have the same elements
        // regardless of their position

        // First, mark the array as reordered if the arrays are different
        if left != right {
            entries.push(DiffEntry {
                diff_type: DiffType::ArrayReordered,
                path: path.to_string(),
                old_value: None,
                new_value: None,
            });

            // If show_nested_differences is enabled, we also want to show the specific differences
            if options.show_nested_differences {
                // We need to match items between the arrays to find corresponding elements
                // This is a simplified approach that works for arrays with unique elements
                // A more sophisticated approach would be needed for complex cases

                // First, try to match items by their "id" field if they are objects
                let mut matched_indices: Vec<Option<usize>> = vec![None; left.len()];

                // Try to match items by their content
                for (i, left_item) in left.iter().enumerate() {
                    for (j, right_item) in right.iter().enumerate() {
                        // Skip already matched items
                        if matched_indices.iter().any(|idx| idx.map_or(false, |idx_val| idx_val == j)) {
                            continue;
                        }

                        // Try to match by id field if objects
                        if let (Value::Object(left_obj), Value::Object(right_obj)) = (left_item, right_item) {
                            if let (Some(left_id), Some(right_id)) = (left_obj.get("id"), right_obj.get("id")) {
                                if left_id == right_id {
                                    matched_indices[i] = Some(j);
                                    break;
                                }
                            }
                        }

                        // If not matched by id, try exact equality
                        if left_item == right_item {
                            matched_indices[i] = Some(j);
                            break;
                        }
                    }
                }

                // Now compare matched items for nested differences
                for (i, left_item) in left.iter().enumerate() {
                    if let Some(j) = matched_indices[i] {
                        let right_item = &right[j];
                        if left_item != right_item {
                            // Items are matched but different, compare their contents
                            let item_path = format!("{}[{}]", path, i);
                            compare_values(left_item, right_item, &item_path, entries, options)?;
                        }
                    } else {
                        // Item in left not found in right
                        let item_path = format!("{}[{}]", path, i);
                        entries.push(DiffEntry {
                            diff_type: DiffType::Removed,
                            path: item_path,
                            old_value: Some(left_item.clone()),
                            new_value: None,
                        });
                    }
                }

                // Find items in right that weren't matched to anything in left
                let matched_right_indices: Vec<usize> = matched_indices.iter()
                    .filter_map(|opt| *opt)
                    .collect();

                for (j, right_item) in right.iter().enumerate() {
                    if !matched_right_indices.contains(&j) {
                        // Item in right not found in left
                        let item_path = format!("{}[{}]", path, j);
                        entries.push(DiffEntry {
                            diff_type: DiffType::Added,
                            path: item_path,
                            old_value: None,
                            new_value: Some(right_item.clone()),
                        });
                    }
                }
            }
        }
    } else {
        // For ordered comparison, we compare elements at the same indices
        if options.identify_array_item_changes {
            // Compare common elements
            let min_len = left.len().min(right.len());

            for i in 0..min_len {
                let item_path = format!("{}[{}]", path, i);
                compare_values(&left[i], &right[i], &item_path, entries, options)?;
            }

            // Handle extra elements in left
            for i in min_len..left.len() {
                let item_path = format!("{}[{}]", path, i);
                entries.push(DiffEntry {
                    diff_type: DiffType::Removed,
                    path: item_path,
                    old_value: Some(left[i].clone()),
                    new_value: None,
                });
            }

            // Handle extra elements in right
            for i in min_len..right.len() {
                let item_path = format!("{}[{}]", path, i);
                entries.push(DiffEntry {
                    diff_type: DiffType::Added,
                    path: item_path,
                    old_value: None,
                    new_value: Some(right[i].clone()),
                });
            }
        } else {
            // Mark the whole array as modified if there are any differences
            if left != right {
                entries.push(DiffEntry {
                    diff_type: DiffType::Modified,
                    path: path.to_string(),
                    old_value: Some(Value::Array(left.to_vec())),
                    new_value: Some(Value::Array(right.to_vec())),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_compare_equal_values() {
        let left = json!({"name": "John", "age": 30});
        let right = json!({"name": "John", "age": 30});
        let options = CompareOptions::default();

        let result = compare_json(&left, &right, &options).unwrap();
        assert_eq!(result.entries.len(), 0);
    }

    #[test]
    fn test_compare_different_values() {
        let left = json!({"name": "John", "age": 30});
        let right = json!({"name": "Jane", "age": 30});
        let options = CompareOptions::default();

        let result = compare_json(&left, &right, &options).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].diff_type, DiffType::Modified);
        assert_eq!(result.entries[0].path, "$.name");
    }

    #[test]
    fn test_compare_added_property() {
        let left = json!({"name": "John"});
        let right = json!({"name": "John", "age": 30});
        let options = CompareOptions::default();

        let result = compare_json(&left, &right, &options).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].diff_type, DiffType::Added);
        assert_eq!(result.entries[0].path, "$.age");
    }

    #[test]
    fn test_compare_removed_property() {
        let left = json!({"name": "John", "age": 30});
        let right = json!({"name": "John"});
        let options = CompareOptions::default();

        let result = compare_json(&left, &right, &options).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].diff_type, DiffType::Removed);
        assert_eq!(result.entries[0].path, "$.age");
    }

    #[test]
    fn test_ignore_path() {
        let left = json!({"name": "John", "timestamp": "2023-01-01"});
        let right = json!({"name": "John", "timestamp": "2023-01-02"});

        let mut options = CompareOptions::default();
        options.ignore_paths.push(JsonPath::new("$.timestamp").unwrap());

        let result = compare_json(&left, &right, &options).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].diff_type, DiffType::Ignored);
        assert_eq!(result.entries[0].path, "$.timestamp");
    }

    #[test]
    fn test_unordered_array() {
        let left = json!({"items": [1, 2, 3]});
        let right = json!({"items": [3, 1, 2]});

        // With ordered comparison
        let options = CompareOptions::default();
        let result = compare_json(&left, &right, &options).unwrap();
        assert!(result.entries.len() > 0); // Should have differences

        // With unordered comparison
        let mut unordered_options = CompareOptions::default();
        unordered_options.unordered_arrays.push(JsonPath::new("$.items").unwrap());

        let result = compare_json(&left, &right, &unordered_options).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].diff_type, DiffType::ArrayReordered);
        assert_eq!(result.entries[0].path, "$.items");
    }

    #[test]
    fn test_unordered_array_with_nested_differences() {
        let left = json!({
            "users": [
                {
                    "id": 1,
                    "name": "Alice",
                    "settings": {
                        "theme": "dark",
                        "notifications": true
                    }
                },
                {
                    "id": 2,
                    "name": "Bob",
                    "settings": {
                        "theme": "light",
                        "notifications": false
                    }
                }
            ]
        });

        let right = json!({
            "users": [
                {
                    "id": 2,
                    "name": "Bob",
                    "settings": {
                        "theme": "dark", // Changed from "light" to "dark"
                        "notifications": false
                    }
                },
                {
                    "id": 1,
                    "name": "Alice",
                    "settings": {
                        "theme": "dark",
                        "notifications": true
                    }
                }
            ]
        });

        // With unordered comparison but without nested differences
        let mut unordered_options = CompareOptions::default();
        unordered_options.unordered_arrays.push(JsonPath::new("$.users").unwrap());
        unordered_options.show_nested_differences = false;

        let result = compare_json(&left, &right, &unordered_options).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].diff_type, DiffType::ArrayReordered);
        assert_eq!(result.entries[0].path, "$.users");

        // With unordered comparison and with nested differences
        let mut unordered_options_with_nested = CompareOptions::default();
        unordered_options_with_nested.unordered_arrays.push(JsonPath::new("$.users").unwrap());
        unordered_options_with_nested.show_nested_differences = true;

        let result = compare_json(&left, &right, &unordered_options_with_nested).unwrap();

        // Should have the reordered entry and the nested difference
        assert!(result.entries.len() > 1);

        // Check for the reordered entry
        let reordered_entry = result.entries.iter().find(|e| e.diff_type == DiffType::ArrayReordered);
        assert!(reordered_entry.is_some());
        assert_eq!(reordered_entry.unwrap().path, "$.users");

        // Check for the nested difference (theme change)
        let nested_diff = result.entries.iter().any(|e|
            e.path.contains("theme") &&
            e.diff_type == DiffType::Modified &&
            e.old_value.as_ref().map(|v| v.as_str() == Some("light")).unwrap_or(false) &&
            e.new_value.as_ref().map(|v| v.as_str() == Some("dark")).unwrap_or(false)
        );

        assert!(nested_diff, "Should detect the nested theme change");
    }

    #[test]
    fn test_array_item_changes() {
        let left = json!({
            "items": [
                {"id": 1, "value": "unchanged"},
                {"id": 2, "value": "original"},
                {"id": 3, "value": "unchanged"},
                {"id": 4, "value": "to be removed"},
                {"id": 5, "value": "unchanged"}
            ]
        });

        let right = json!({
            "items": [
                {"id": 1, "value": "unchanged"},
                {"id": 2, "value": "modified"},
                {"id": 3, "value": "unchanged"},
                {"id": 6, "value": "newly added"},
                {"id": 5, "value": "unchanged"}
            ]
        });

        // With identify_array_item_changes = true (default)
        let options = CompareOptions::default();
        let result = compare_json(&left, &right, &options).unwrap();

        // Should identify specific changes
        assert!(result.entries.iter().any(|e| e.path == "$.items[1].value"));
        assert!(result.entries.iter().any(|e| e.path.starts_with("$.items[3]")));

        // Should not report unchanged items
        assert!(!result.entries.iter().any(|e| e.path == "$.items[0]"));
        assert!(!result.entries.iter().any(|e| e.path == "$.items[2]"));
        assert!(!result.entries.iter().any(|e| e.path == "$.items[4]"));

        // With identify_array_item_changes = false
        let mut whole_array_options = CompareOptions::default();
        whole_array_options.identify_array_item_changes = false;

        let result = compare_json(&left, &right, &whole_array_options).unwrap();

        // Should mark the whole array as modified
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].path, "$.items");
        assert_eq!(result.entries[0].diff_type, DiffType::Modified);
    }

    #[test]
    fn test_nested_objects() {
        let left = json!({
            "user": {
                "name": "John",
                "address": {
                    "city": "New York",
                    "zip": "10001"
                }
            }
        });

        let right = json!({
            "user": {
                "name": "John",
                "address": {
                    "city": "Boston",
                    "zip": "10001"
                }
            }
        });

        let options = CompareOptions::default();
        let result = compare_json(&left, &right, &options).unwrap();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].diff_type, DiffType::Modified);
        assert_eq!(result.entries[0].path, "$.user.address.city");
    }
}
