use super::super::file::search_manifest_and_workspace_dir_for_file_name_match;
use super::{Rule, RuleOutcome};
use cargo_metadata::Metadata;
use regex::Regex;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Default)]
pub struct HasLicenseFile;

lazy_static! {
    static ref HAS_LICENSE_FILE: Regex =
        Regex::new(r"^(?i)LICENSE").expect("Failed to create HasLicenseFile regex.");
}

impl Rule for HasLicenseFile {
    fn description(&self) -> &'static str {
        "Should have a LICENSE file in the project directory."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        _verbose: bool,
        metadata: &Option<Metadata>,
        _print_output: &mut Write,
    ) -> RuleOutcome {
        search_manifest_and_workspace_dir_for_file_name_match(
            &HAS_LICENSE_FILE,
            cargo_manifest_file_path,
            metadata,
        )
    }
}
