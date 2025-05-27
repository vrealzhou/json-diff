# JSON Diff tool

A tool for comparing and identifying differences between JSON files or objects.

## Features
1. **Split-Screen View**: Side-by-side JSON file comparison with syntax highlighting and diff navigation.
2. **Dual View Modes**: Toggle between List view (diff entries) and Split-Screen view (actual file content).
3. **JSON Syntax Highlighting**: Color-coded JSON syntax in split-screen mode for better readability.
4. **Smart Diff Navigation**: Jump between diff locations with `n`/`N` keys in split-screen mode.
5. **Line Number Sorted Output**: Diff results are automatically sorted by line number (based on the first file) for intuitive reading order.
6. **Readable Format by Default**: Uses descriptive text like `[MODIFIED]`, `[ADDED]` instead of cryptic symbols for better usability.
7. **Line Number Tracking**: Shows source and target file line numbers for easy content location in original files.
8. **Flexible Display Options**: Switch between readable text and compact symbols with `--symbols` flag or `r` key in interactive mode.
9. It can specify json path in regex format to ignore certain fields.
10. It can compare certain array fields without order by specify json path in regex format.

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

   **View Controls:**
   - `v`: Toggle between List and Split-Screen view modes

   **List View Navigation:**
   - `j` or Down Arrow: Move to next diff entry
   - `k` or Up Arrow: Move to previous diff entry

   **Split-Screen View Navigation:**
   - `j` or Down Arrow: Scroll down both files
   - `k` or Up Arrow: Scroll up both files
   - `n`: Jump to next diff location (in line number order)
   - `N`: Jump to previous diff location (in line number order)

   **Common Controls:**
   - `r`: Toggle between readable and symbols format
   - `h` or `?`: Toggle help screen with symbol explanations
   - `q` or Esc: Quit

   Note: The interactive mode is designed for keyboard-only operation and does not support mouse interactions.

## Split-Screen View Features

The split-screen view provides a powerful side-by-side comparison experience:

- **JSON Syntax Highlighting**: Keys, strings, numbers, and structural elements are color-coded
- **Line Numbers**: Each line is numbered for easy reference
- **Smart Diff Highlighting**:
  - **Current focused diff**: Highlighted with bright magenta background and bold text
  - **Other diff lines**: Highlighted with gray background
  - **Normal lines**: JSON syntax highlighting
- **Smart Navigation**: Use `n`/`N` to jump directly to diff locations in line number order
- **Synchronized Scrolling**: Both files scroll together with `j`/`k` keys
- **Real-time View Switching**: Toggle between list and split-screen views instantly with `v`
- **Current Diff Display**: Footer shows which diff is currently focused

### Split-Screen View Example
```
┌Left File──────────────────┐ ┌Right File─────────────────┐
│   1 {                     │ │   1 {                     │
│   2   "name": "John Doe", │ │   2   "name": "Jane Smith"│ ← FOCUSED (magenta bg)
│   3   "age": 30,          │ │   3   "age": 30,          │
│   4   "email": "john@..." │ │   4   "email": "jane@..." │ ← Other diff (gray bg)
│   5 }                     │ │   5 }                     │
└───────────────────────────┘ └───────────────────────────┘
Footer: Diff 1/2 | Current: [MODIFIED] $.name | j/k: scroll, n/N: diff
```

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

**Readable Format (Default - sorted by line number):**
```
[ARRAY_REORDERED] $.users (L2:L2): [REORDERED]
[MODIFIED] $.user.email (L5:L5): "john.doe@example.com" -> "john.smith@example.com"
[ADDED] $.user.profile.preferences.timezone (L9:L13): "PST"
[MODIFIED] $.metadata.version (L37:L45): "1.0" -> "1.1"
```

**Symbols Format (with --symbols flag - also sorted by line number):**
```
* $.users (L2:L2): [REORDERED]
~ $.user.email (L5:L5): "john.doe@example.com" -> "john.smith@example.com"
+ $.user.profile.preferences.timezone (L9:L13): "PST"
~ $.metadata.version (L37:L45): "1.0" -> "1.1"
```

The format `(L5:L5)` shows line 5 in both source and target files, while `(L37:L45)` shows line 37 in the source file and line 45 in the target file.

See [tests/README.md](tests/README.md) for detailed information about test files and fixtures.
