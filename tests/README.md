# Test Files

This directory contains test files and fixtures for the JSON diff tool.

## Structure

- `integration_tests.rs` - Integration tests for the CLI
- `test_ratatui.sh` - Interactive test script for the ratatui TUI
- `fixtures/` - Test data files

## Test Fixtures

### Basic Comparison Files
- `sample1.json` - Sample JSON file with user data
- `sample2.json` - Modified version of sample1.json with various changes

### Long Text Files (for testing text wrapping)
- `long1.json` - JSON file with very long field names and values
- `long2.json` - Modified version with changes to test text wrapping

### Configuration
- `profile.toml` - Test profile with ignore rules and unordered array settings

## Running Tests

### Unit and Integration Tests
```bash
cargo test
```

### Interactive TUI Tests
```bash
# Make the script executable
chmod +x tests/test_ratatui.sh

# Run the interactive test script
./tests/test_ratatui.sh
```

### Manual Testing Examples

#### Basic comparison
```bash
cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json
```

#### With profile
```bash
cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json --profile tests/fixtures/profile.toml
```

#### Interactive mode
```bash
cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json --interactive
```

#### Interactive mode with profile
```bash
cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json --profile tests/fixtures/profile.toml --interactive
```

#### Long text wrapping test
```bash
cargo run -- tests/fixtures/long1.json tests/fixtures/long2.json --interactive
```

## Test Data Description

### sample1.json vs sample2.json
These files demonstrate various types of changes:
- Modified fields (name, email, age, location, etc.)
- Added fields (timezone, sessionTimeout, etc.)
- Removed fields (when arrays are reordered)
- Array modifications (projects, roles)
- Nested object changes

### long1.json vs long2.json
These files test text wrapping with:
- Very long field names
- Very long string values
- Long array elements
- Nested structures with long content

### profile.toml
Demonstrates profile features:
- `ignore` - Fields to ignore during comparison
- `unordered` - Arrays to treat as unordered sets
- `show_nested_differences` - Show nested changes in unordered arrays
- `identify_array_item_changes` - Identify specific array item changes
