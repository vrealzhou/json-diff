use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
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

    // Build line number mappings
    let left_line_map = build_line_number_map(&left_content, &left_json);
    let right_line_map = build_line_number_map(&right_content, &right_json);

    let mut result = compare_json_with_lines(&left_json, &right_json, options, &left_line_map, &right_line_map)?;
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
    let empty_map = HashMap::new();
    compare_json_with_lines(left, right, options, &empty_map, &empty_map)
}

/// Compare two JSON values with line number information
pub fn compare_json_with_lines(
    left: &Value,
    right: &Value,
    options: &CompareOptions,
    left_line_map: &HashMap<String, usize>,
    right_line_map: &HashMap<String, usize>,
) -> Result<DiffResult, JsonDiffError> {
    let mut entries = Vec::new();

    compare_values_with_lines(left, right, "$", &mut entries, options, left_line_map, right_line_map)?;

    // Sort entries by line number (based on left file's line numbers)
    sort_entries_by_line_number(&mut entries);

    let result = DiffResult {
        left_file: None,
        right_file: None,
        timestamp: Utc::now(),
        entries,
    };

    Ok(result)
}

/// Sort diff entries by line number (based on left file's line numbers)
/// Entries without line numbers are placed at the end
fn sort_entries_by_line_number(entries: &mut Vec<DiffEntry>) {
    entries.sort_by_key(|entry| {
        // Use left_line as primary sort key, fall back to right_line if left_line is None
        // Entries without any line numbers go to the end (using usize::MAX)
        match (entry.left_line, entry.right_line) {
            (Some(left_line), _) => left_line,
            (None, Some(right_line)) => right_line,
            (None, None) => usize::MAX,
        }
    });
}

/// Build a mapping from JSON paths to line numbers
fn build_line_number_map(content: &str, json: &Value) -> HashMap<String, usize> {
    let mut line_map = HashMap::new();

    // Build a mapping by parsing the JSON structure and tracking line numbers
    build_path_line_mapping(json, "$", content, &mut line_map);

    line_map
}

/// Recursively build path to line number mapping
fn build_path_line_mapping(
    value: &Value,
    current_path: &str,
    content: &str,
    line_map: &mut HashMap<String, usize>,
) {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                let field_path = if current_path == "$" {
                    format!("$.{}", key)
                } else {
                    format!("{}.{}", current_path, key)
                };

                // Find the line number for this field
                if let Some(line_num) = find_field_line_number(content, key) {
                    line_map.insert(field_path.clone(), line_num);
                }

                // Recursively process nested values
                build_path_line_mapping(val, &field_path, content, line_map);
            }
        }
        Value::Array(arr) => {
            for (index, val) in arr.iter().enumerate() {
                let array_path = format!("{}[{}]", current_path, index);

                // For arrays, we'll use the line number of the parent field
                // A more sophisticated approach would track the exact array element line
                if let Some(parent_line) = line_map.get(current_path) {
                    line_map.insert(array_path.clone(), *parent_line);
                }

                // Recursively process array elements
                build_path_line_mapping(val, &array_path, content, line_map);
            }
        }
        _ => {
            // For primitive values, the line number is already set by the parent object
        }
    }
}

/// Find the line number where a specific field is defined
fn find_field_line_number(content: &str, field_name: &str) -> Option<usize> {
    let mut found_lines = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // 1-based line numbers

        // Look for the field name as a quoted string followed by a colon
        let pattern = format!("\"{}\"", field_name);
        if let Some(start) = line.find(&pattern) {
            // Check if this is followed by a colon (with possible whitespace)
            let after_quote = start + pattern.len();
            if line[after_quote..].trim_start().starts_with(':') {
                found_lines.push(line_num);
            }
        }
    }

    // Return the first occurrence for now
    // A more sophisticated approach would track context to find the right occurrence
    found_lines.first().copied()
}

/// Find line number for a given path, with fallback strategies
fn find_line_for_path(path: &str, line_map: &HashMap<String, usize>) -> Option<usize> {
    // Try exact match first
    if let Some(line) = line_map.get(path) {
        return Some(*line);
    }

    // Try to find a parent path that exists
    // For example, if looking for "$.user.profile.age" and it doesn't exist,
    // try "$.user.profile", then "$.user"
    let mut current_path = path.to_string();
    while let Some(last_dot) = current_path.rfind('.') {
        current_path = current_path[..last_dot].to_string();
        if let Some(line) = line_map.get(&current_path) {
            return Some(*line);
        }
    }

    // Try to find by field name only (last component of the path)
    if let Some(field_name) = path.split('.').last() {
        // Remove array indices if present
        let clean_field = field_name.split('[').next().unwrap_or(field_name);

        // Look for any path ending with this field name
        for (map_path, line) in line_map {
            if map_path.ends_with(&format!(".{}", clean_field)) || map_path == &format!("$.{}", clean_field) {
                return Some(*line);
            }
        }
    }

    None
}



fn compare_values_with_lines(
    left: &Value,
    right: &Value,
    path: &str,
    entries: &mut Vec<DiffEntry>,
    options: &CompareOptions,
    left_line_map: &HashMap<String, usize>,
    right_line_map: &HashMap<String, usize>,
) -> Result<(), JsonDiffError> {
    // Check if this path should be ignored
    if options.ignore_paths.iter().any(|p| p.matches(path)) {
        entries.push(DiffEntry {
            diff_type: DiffType::Ignored,
            path: path.to_string(),
            old_value: None,
            new_value: None,
            left_line: find_line_for_path(path, left_line_map),
            right_line: find_line_for_path(path, right_line_map),
        });
        return Ok(());
    }

    match (left, right) {
        (Value::Object(left_obj), Value::Object(right_obj)) => {
            compare_objects_with_lines(left_obj, right_obj, path, entries, options, left_line_map, right_line_map)?;
        }
        (Value::Array(left_arr), Value::Array(right_arr)) => {
            // Check if this array should be compared without order
            let unordered = options.unordered_arrays.iter().any(|p| p.matches(path));
            compare_arrays_with_lines(left_arr, right_arr, path, entries, options, unordered, left_line_map, right_line_map)?;
        }
        _ if left == right => {
            // Values are equal, no diff needed
        }
        _ => {
            // Values are different
            let left_line = find_line_for_path(path, left_line_map);
            let right_line = find_line_for_path(path, right_line_map);

            entries.push(DiffEntry {
                diff_type: DiffType::Modified,
                path: path.to_string(),
                old_value: Some(left.clone()),
                new_value: Some(right.clone()),
                left_line,
                right_line,
            });
        }
    }

    Ok(())
}



fn compare_objects_with_lines(
    left: &Map<String, Value>,
    right: &Map<String, Value>,
    path: &str,
    entries: &mut Vec<DiffEntry>,
    options: &CompareOptions,
    left_line_map: &HashMap<String, usize>,
    right_line_map: &HashMap<String, usize>,
) -> Result<(), JsonDiffError> {
    // Find keys that exist in left but not in right
    for key in left.keys() {
        if !right.contains_key(key) {
            let key_path = format!("{}.{}", path, key);

            // Check if this path should be ignored
            if options.ignore_paths.iter().any(|p| p.matches(&key_path)) {
                entries.push(DiffEntry {
                    diff_type: DiffType::Ignored,
                    path: key_path.clone(),
                    old_value: None,
                    new_value: None,
                    left_line: find_line_for_path(&key_path, left_line_map),
                    right_line: find_line_for_path(&key_path, right_line_map),
                });
                continue;
            }

            entries.push(DiffEntry {
                diff_type: DiffType::Removed,
                path: key_path.clone(),
                old_value: Some(left[key].clone()),
                new_value: None,
                left_line: find_line_for_path(&key_path, left_line_map),
                right_line: find_line_for_path(&key_path, right_line_map),
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
                path: key_path.clone(),
                old_value: None,
                new_value: None,
                left_line: find_line_for_path(&key_path, left_line_map),
                right_line: find_line_for_path(&key_path, right_line_map),
            });
            continue;
        }

        if !left.contains_key(key) {
            entries.push(DiffEntry {
                diff_type: DiffType::Added,
                path: key_path.clone(),
                old_value: None,
                new_value: Some(right[key].clone()),
                left_line: find_line_for_path(&key_path, left_line_map),
                right_line: find_line_for_path(&key_path, right_line_map),
            });
        } else {
            compare_values_with_lines(&left[key], &right[key], &key_path, entries, options, left_line_map, right_line_map)?;
        }
    }

    Ok(())
}



fn compare_arrays_with_lines(
    left: &[Value],
    right: &[Value],
    path: &str,
    entries: &mut Vec<DiffEntry>,
    options: &CompareOptions,
    unordered: bool,
    left_line_map: &HashMap<String, usize>,
    right_line_map: &HashMap<String, usize>,
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
                left_line: find_line_for_path(path, left_line_map),
                right_line: find_line_for_path(path, right_line_map),
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
                            compare_values_with_lines(left_item, right_item, &item_path, entries, options, left_line_map, right_line_map)?;
                        }
                    } else {
                        // Item in left not found in right
                        let item_path = format!("{}[{}]", path, i);
                        entries.push(DiffEntry {
                            diff_type: DiffType::Removed,
                            path: item_path.clone(),
                            old_value: Some(left_item.clone()),
                            new_value: None,
                            left_line: find_line_for_path(&item_path, left_line_map),
                            right_line: find_line_for_path(&item_path, right_line_map),
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
                            path: item_path.clone(),
                            old_value: None,
                            new_value: Some(right_item.clone()),
                            left_line: find_line_for_path(&item_path, left_line_map),
                            right_line: find_line_for_path(&item_path, right_line_map),
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
                compare_values_with_lines(&left[i], &right[i], &item_path, entries, options, left_line_map, right_line_map)?;
            }

            // Handle extra elements in left
            for i in min_len..left.len() {
                let item_path = format!("{}[{}]", path, i);
                entries.push(DiffEntry {
                    diff_type: DiffType::Removed,
                    path: item_path.clone(),
                    old_value: Some(left[i].clone()),
                    new_value: None,
                    left_line: find_line_for_path(&item_path, left_line_map),
                    right_line: find_line_for_path(&item_path, right_line_map),
                });
            }

            // Handle extra elements in right
            for i in min_len..right.len() {
                let item_path = format!("{}[{}]", path, i);
                entries.push(DiffEntry {
                    diff_type: DiffType::Added,
                    path: item_path.clone(),
                    old_value: None,
                    new_value: Some(right[i].clone()),
                    left_line: find_line_for_path(&item_path, left_line_map),
                    right_line: find_line_for_path(&item_path, right_line_map),
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
                    left_line: find_line_for_path(path, left_line_map),
                    right_line: find_line_for_path(path, right_line_map),
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
