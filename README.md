# JSON Diff tool

A tool for comparing and identifying differences between JSON files or objects.

## Features
1. Compare json and display diffs in terminal buffers. It supports vim keybindings for navigation.
2. **Readable Format by Default**: Uses descriptive text like `[MODIFIED]`, `[ADDED]` instead of cryptic symbols for better usability.
3. **Line Number Tracking**: Shows source and target file line numbers for easy content location in original files.
4. **Flexible Display Options**: Switch between readable text and compact symbols with `--symbols` flag or `r` key in interactive mode.
5. It can specify json path in regex format to ignore certain fields.
6. It can compare certain array fields without order by specify json path in regex format.

## Modules
1. json compare module: compares json and returns diff result in to a file
2. diff display module: displays diff result in an interactive terminal UI with vim-like keybindings

## Documentation
- [Diff Format Specification](docs/diff_format.md): Details about the text-based diff format
- [Rules Format Specification](docs/rules_format.md): How to configure comparison rules

## Usage
1. Install the tool
```bash
cargo install --path .
```

2. Run the tool in text mode (outputs to console or file)
```bash
# Basic comparison (readable format by default)
json-diff <file1> <file2>

# Use compact symbols instead of readable text
json-diff --symbols <file1> <file2>

# With a profile for customizing comparison
json-diff --profile rules.toml <file1> <file2>

# Output to a file
json-diff --output diff.txt <file1> <file2>
```

3. Run the tool in interactive mode
```bash
json-diff --interactive <file1> <file2>
```

4. Interactive Mode Controls (Keyboard Only)
   - `j` or Down Arrow: Move to next diff
   - `k` or Up Arrow: Move to previous diff
   - `r`: Toggle between readable and symbols format
   - `h` or `?`: Toggle help screen with symbol explanations
   - `q` or Esc: Quit

   Note: The interactive mode is designed for keyboard-only operation and does not support mouse interactions.

## Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run interactive TUI tests
./tests/test_ratatui.sh
```

### Test Examples
```bash
# Basic comparison with test files
cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json

# Interactive mode with test files
cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json --interactive

# With profile configuration
cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json --profile tests/fixtures/profile.toml --interactive
```

### Example Output with Line Numbers

**Readable Format (Default):**
```
[MODIFIED] $.user.email (L5:L5): "john.doe@example.com" -> "john.smith@example.com"
[MODIFIED] $.metadata.version (L37:L45): "1.0" -> "1.1"
[ADDED] $.user.profile.preferences.timezone (L9:L13): "PST"
[ARRAY_REORDERED] $.users (L2:L2): [REORDERED]
```

**Symbols Format (with --symbols flag):**
```
~ $.user.email (L5:L5): "john.doe@example.com" -> "john.smith@example.com"
~ $.metadata.version (L37:L45): "1.0" -> "1.1"
+ $.user.profile.preferences.timezone (L9:L13): "PST"
* $.users (L2:L2): [REORDERED]
```

The format `(L5:L5)` shows line 5 in both source and target files, while `(L37:L45)` shows line 37 in the source file and line 45 in the target file.

See [tests/README.md](tests/README.md) for detailed information about test files and fixtures.
