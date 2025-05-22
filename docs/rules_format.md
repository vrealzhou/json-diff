# JSON Diff Rules Format

The rules file (`rules.toml`) allows you to customize how JSON files are compared by specifying paths to ignore or arrays to compare without considering order.

## Format Structure

The rules file uses TOML format with the following structure:

```toml
# Paths to ignore during comparison
ignore = [
    "$.metadata.timestamp",
    "$.*.id",
    "$.users[*].lastLogin"
]

# Arrays to compare without considering order
unordered = [
    "$.items",
    "$.users[*].permissions",
    "$.config.allowedDomains"
]
```

## Path Specification

Paths use a simplified JSONPath-like syntax with regex support:

- `$` represents the root of the document
- `.property` accesses an object property
- `.*` matches any property name
- `[n]` accesses an array element by index
- `[*]` matches any array index

## Examples

### Ignoring Timestamps and Generated IDs

```toml
ignore = [
    "$.metadata.timestamp",
    "$.created_at",
    "$.updated_at",
    "$..id"  # Ignore all properties named "id" at any level
]
```

### Comparing Arrays Without Order

```toml
unordered = [
    "$.users",                # Compare users array without considering order
    "$.products[*].tags",     # Compare tags arrays within each product without order
    "$.settings.permissions"  # Compare permissions array without order
]
```

### Unordered Array Examples

Consider these two JSON documents:

**Document 1:**
```json
{
  "users": [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"},
    {"id": 3, "name": "Charlie"}
  ],
  "tags": ["important", "urgent", "review"]
}
```

**Document 2:**
```json
{
  "users": [
    {"id": 3, "name": "Charlie"},
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"}
  ],
  "tags": ["urgent", "review", "important"]
}
```

With this rule:
```toml
unordered = [
    "$.users",
    "$.tags"
]
```

The comparison would show no differences, as both arrays contain the same elements but in different orders.

### Complex Example

```toml
ignore = [
    "$.metadata.*",           # Ignore all metadata properties
    "$.audit.*_time",         # Ignore all time-related audit fields
    "$..generated_id"         # Ignore all generated_id fields at any level
]

unordered = [
    "$.users",                # Users can be in any order
    "$.config.features",      # Features can be in any order
    "$..tags"                 # All tag arrays can be in any order
]
```

## Usage

Pass the rules file to the JSON diff tool using the `--profile` option:

```bash
json-diff --profile rules.toml file1.json file2.json
```
