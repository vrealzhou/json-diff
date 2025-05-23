#!/bin/bash

echo "Testing JSON Diff Interactive Mode with Ratatui"
echo "==============================================="
echo ""
echo "This will start the interactive JSON diff viewer using ratatui."
echo "Features:"
echo "  - Professional TUI with borders and sections"
echo "  - Color-coded diff entries"
echo "  - Vim-like navigation (j/k keys)"
echo "  - Help screen (h or ?)"
echo "  - Automatic text wrapping"
echo "  - List selection highlighting"
echo "  - Keyboard-only operation (no mouse support)"
echo ""
echo "Use the following keys:"
echo "  j or Down Arrow: Move to next diff"
echo "  k or Up Arrow: Move to previous diff"
echo "  h or ?: Show help"
echo "  q or Esc: Quit"
echo ""
echo "Press Enter to start the basic comparison..."
read

# Run the interactive mode
echo "Running: cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json --interactive"
cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json --interactive

echo ""
echo "Press Enter to test with profile (ignore rules and unordered arrays)..."
read

echo "Running: cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json --profile tests/fixtures/profile.toml --interactive"
cargo run -- tests/fixtures/sample1.json tests/fixtures/sample2.json --profile tests/fixtures/profile.toml --interactive

echo ""
echo "Press Enter to test with long text (text wrapping)..."
read

echo "Running: cargo run -- tests/fixtures/long1.json tests/fixtures/long2.json --interactive"
cargo run -- tests/fixtures/long1.json tests/fixtures/long2.json --interactive
