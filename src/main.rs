use json_diff_cli::{Args, run};
use clap::Parser;

fn main() {
    let args = Args::parse();

    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

// Usage examples:
// json-diff file1.json file2.json
// json-diff file1.json file2.json -o diff.txt
// json-diff file1.json file2.json -p profile.toml
// json-diff file1.json file2.json -i  # Interactive mode
