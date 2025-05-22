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
+ $.new.property: "added value"
- $.removed.property: "removed value"
~ $.modified.property: "old value" -> "new value"
! $.array[2]: {"old": "value"} -> {"new": "value"}
* $.unordered.array: [REORDERED]
? $.ignored.property: [IGNORED]
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

## Advanced Features

### Unordered Arrays with Nested Differences

When arrays are marked as unordered, the tool can now show both the reordering and the nested differences within the array elements. This is useful for comparing arrays of complex objects where the order doesn't matter but you still want to see specific changes within the objects.

Example:
```
* $.users: [REORDERED]
~ $.users[0].settings.theme: "light" -> "dark"
~ $.users[1].name: "Bob" -> "Robert"
```

### Specific Array Item Differences

The tool can identify specific different items in arrays rather than marking the whole array as different. This makes it easier to locate exactly which elements have changed.

Example:
```
~ $.items[1].value: "original" -> "modified"
+ $.items[3].value: "newly added"
- $.items[4].value: "removed"
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

+ $.address.zipcode: "10001"
- $.phone.home: "555-1234"
~ $.name: "John Doe" -> "John Smith"
! $.hobbies[1]: "swimming" -> "hiking"
* $.friends: [REORDERED]
~ $.friends[0].name: "Alice" -> "Alicia"
? $.lastLogin: [IGNORED]
```
```