use super::super::file::search_manifest_and_workspace_dir_for_file_name_match;
use super::{Rule, RuleOutcome};
use cargo_metadata::Metadata;
use regex::Regex;
use std::io::Write;
use std::path::Path;

#[derive(Default, Debug)]
pub struct HasContinuousIntegrationFile;

lazy_static! {
    static ref HAS_CONTINUOUS_INTEGRATION_FILE: Regex =
        Regex::new(r"^(?i)(appveyor|\.appveyor|\.drone|\.gitlab-ci|\.travis)\.ya?ml")
            .expect("Failed to create HasContinuousIntegrationFile regex.");
}

impl Rule for HasContinuousIntegrationFile {
    fn description(&self) -> &'static str {
        "Should have a file suggesting the use of a continuous integration system."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        _verbose: bool,
        metadata: &Option<Metadata>,
        _: &mut Write,
    ) -> RuleOutcome {
        search_manifest_and_workspace_dir_for_file_name_match(
            &HAS_CONTINUOUS_INTEGRATION_FILE,
            cargo_manifest_file_path,
            metadata,
        )
    }
}
