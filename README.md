# JSON Diff tool

A tool for comparing and identifying differences between JSON files or objects.

## Features
1. Compare json and display diffs in terminal buffers. It supports vim keybindings for navigation.
2. It can specify json path in regex format to ignore certain fields.
3. It can compare certain array fields without order by specify json path in regex format.

## Modules
1. json compare module: compares json and returns diff result in to a file
2. diff display module: displays diff result file in terminal buffers

## Documentation
- [Diff Format Specification](docs/diff_format.md): Details about the text-based diff format
- [Rules Format Specification](docs/rules_format.md): How to configure comparison rules

## Usage
1. Install the tool
```bash
cargo install --path .
```
2. Run the tool
```bash
json-diff --profile rules.toml <file1> <file2>
```
3. Use vim keybindings to navigate between diffs
4. Use `:q` to quit
