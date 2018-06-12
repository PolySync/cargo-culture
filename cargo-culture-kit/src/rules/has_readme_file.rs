use super::super::file::is_file_present;
use super::{Rule, RuleOutcome};
use cargo_metadata::Metadata;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Default)]
pub struct HasReadmeFile;

impl Rule for HasReadmeFile {
    fn description(&self) -> &'static str {
        "Should have a README.md file in the project directory."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        _verbose: bool,
        _metadata: &Option<Metadata>,
        _print_output: &mut Write,
    ) -> RuleOutcome {
        let mut path = cargo_manifest_file_path.to_path_buf();
        path.pop();
        is_file_present(&path.join("README.md")).into()
    }
}
