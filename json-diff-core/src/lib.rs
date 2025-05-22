//! JSON comparison library that generates diffs in a text-based format

mod diff;
mod compare;
mod path;
mod error;

pub use diff::{DiffEntry, DiffType, DiffResult};
pub use compare::{compare_json, compare_files, CompareOptions};
pub use error::JsonDiffError;
pub use path::JsonPath;