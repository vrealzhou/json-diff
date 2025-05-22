//! CLI interface for the JSON diff tool

use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use clap::Parser;
use serde::Deserialize;
use json_diff_core::{compare_files, CompareOptions, JsonDiffError, JsonPath};

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

    // Output the result
    let diff_text = result.to_string();

    if let Some(output_path) = args.output {
        fs::write(&output_path, diff_text)
            .context("Failed to write diff result to file")?;
    } else {
        println!("{}", diff_text);
    }

    Ok(())
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
