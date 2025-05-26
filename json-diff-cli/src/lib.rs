//! CLI interface for the JSON diff tool

use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use clap::Parser;
use serde::Deserialize;
use json_diff_core::{compare_files, CompareOptions, JsonDiffError, JsonPath, DiffResult};
use json_diff_display;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// First JSON file to compare
    pub file1: PathBuf,

    /// Second JSON file to compare
    pub file2: PathBuf,

    /// Profile with comparison rules
    #[arg(short, long)]
    pub profile: Option<PathBuf>,

    /// Output file for diff result (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Display diff result in interactive terminal UI
    #[arg(short, long)]
    pub interactive: bool,

    /// Use symbols instead of readable text for diff types
    #[arg(short = 'S', long)]
    pub symbols: bool,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub ignore: Vec<String>,

    #[serde(default)]
    pub unordered: Vec<String>,

    #[serde(default)]
    pub show_nested_differences: bool,

    #[serde(default)]
    pub identify_array_item_changes: Option<bool>,
}

pub fn run(args: Args) -> Result<()> {
    // Load profile if specified
    let options = if let Some(profile_path) = args.profile {
        load_profile(&profile_path)?
    } else {
        CompareOptions::default()
    };

    // Compare files
    let result = compare_files(&args.file1, &args.file2, &options)
        .context("Failed to compare JSON files")?;

    if args.interactive {
        // Use the interactive display module (readable format is default, symbols if requested)
        json_diff_display::run_display_with_options(result, !args.symbols)
            .context("Failed to run interactive display")?;
    } else {
        // Output the result as text (readable format is default, symbols if requested)
        let diff_text = if args.symbols {
            result.to_string()
        } else {
            format_diff_readable(&result)
        };

        if let Some(output_path) = args.output {
            fs::write(&output_path, diff_text)
                .context("Failed to write diff result to file")?;
        } else {
            println!("{}", diff_text);
        }
    }

    Ok(())
}

fn format_diff_readable(result: &DiffResult) -> String {
    let mut output = String::new();

    output.push_str("DIFF-JSON v1\n");

    if let Some(left) = &result.left_file {
        output.push_str(&format!("LEFT: {}\n", left.display()));
    }

    if let Some(right) = &result.right_file {
        output.push_str(&format!("RIGHT: {}\n", right.display()));
    }

    output.push_str(&format!("TIMESTAMP: {}\n\n", result.timestamp.to_rfc3339()));

    for entry in &result.entries {
        output.push_str(&format!("{}\n", entry.format_readable()));
    }

    output
}

fn load_profile(path: &PathBuf) -> Result<CompareOptions, JsonDiffError> {
    let content = fs::read_to_string(path)?;
    let profile: Profile = toml::from_str(&content)
        .map_err(|e| JsonDiffError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse profile: {}", e)
        )))?;

    let mut options = CompareOptions::default();

    // Parse ignore paths
    for path_str in &profile.ignore {
        let path = JsonPath::new(path_str)?;
        options.ignore_paths.push(path);
    }

    // Parse unordered array paths
    for path_str in &profile.unordered {
        let path = JsonPath::new(path_str)?;
        options.unordered_arrays.push(path);
    }

    // Set show_nested_differences option
    options.show_nested_differences = profile.show_nested_differences;

    // Set identify_array_item_changes option if provided
    if let Some(identify_array_item_changes) = profile.identify_array_item_changes {
        options.identify_array_item_changes = identify_array_item_changes;
    }

    Ok(options)
}
