# JSON Diff Format Specification

The JSON diff format is a text-based representation of differences between two JSON documents. It's designed to be human-readable, git-friendly, and easily parsable.

## Format Structure

```
DIFF-JSON v1
# Header information
LEFT: path/to/left.json
RIGHT: path/to/right.json
TIMESTAMP: 2023-07-21T14:30:00Z

# Diff entries
+ $.new.property (L5:L8): "added value"
- $.removed.property (L12): "removed value"
~ $.modified.property (L3:L3): "old value" -> "new value"
! $.array[2] (L15:L15): {"old": "value"} -> {"new": "value"}
* $.unordered.array (L10:L10): [REORDERED]
? $.ignored.property (L7:L7): [IGNORED]
```

## Entry Types

- `+` Added: A property exists in the right file but not in the left
- `-` Removed: A property exists in the left file but not in the right
- `~` Modified: A property exists in both files but with different values
- `!` Array item changed: An item in an array has changed
- `*` Array reordered: An array was compared without considering order
- `?` Ignored: A property was ignored based on rules

## JSON Path

Each entry uses JSONPath notation to identify the location of the difference.

## Line Numbers

Each diff entry includes line number information in the format `(L<left>:<right>)` or `(L<line>)` to help users locate the content in the original JSON files:

- `(L3:L3)` - The field appears on line 3 in both source and target files
- `(L5:L8)` - The field appears on line 5 in the source file and line 8 in the target file
- `(L12)` - The field appears on line 12 in the source file only (for removed items)
- `(L8)` - The field appears on line 8 in the target file only (for added items)

### Line Number Examples

```
~ $.user.email (L5:L5): "john.doe@example.com" -> "john.smith@example.com"
+ $.user.profile.timezone (L9:L13): "PST"
- $.user.phone (L15): "555-1234"
* $.users (L2:L2): [REORDERED]
```

The line numbers help users quickly navigate to the specific content in their original JSON files, making it easier to understand and verify the changes.

## Advanced Features

### Unordered Arrays with Nested Differences

When arrays are marked as unordered, the tool can now show both the reordering and the nested differences within the array elements. This is useful for comparing arrays of complex objects where the order doesn't matter but you still want to see specific changes within the objects.

Example:
```
* $.users (L2:L2): [REORDERED]
~ $.users[0].settings.theme (L7:L7): "light" -> "dark"
~ $.users[1].name (L5:L5): "Bob" -> "Robert"
```

### Specific Array Item Differences

The tool can identify specific different items in arrays rather than marking the whole array as different. This makes it easier to locate exactly which elements have changed.

Example:
```
~ $.items[1].value (L8:L8): "original" -> "modified"
+ $.items[3].value (L12): "newly added"
- $.items[4].value (L15): "removed"
```

## Configuration Options

In the profile TOML file, you can configure the following options:

```toml
# Paths to ignore during comparison
ignore = ["$.timestamp", "$.metadata.lastUpdated"]

# Paths to arrays that should be compared without considering order
unordered = ["$.users", "$.items"]

# Whether to show nested differences in unordered arrays
show_nested_differences = true

# Whether to identify specific different items in arrays
identify_array_item_changes = true
```

## Examples

```
DIFF-JSON v1
LEFT: user1.json
RIGHT: user2.json
TIMESTAMP: 2023-07-21T14:30:00Z

+ $.address.zipcode (L8:L12): "10001"
- $.phone.home (L15): "555-1234"
~ $.name (L2:L2): "John Doe" -> "John Smith"
! $.hobbies[1] (L10:L10): "swimming" -> "hiking"
* $.friends (L18:L18): [REORDERED]
~ $.friends[0].name (L20:L20): "Alice" -> "Alicia"
? $.lastLogin (L25:L25): [IGNORED]
```