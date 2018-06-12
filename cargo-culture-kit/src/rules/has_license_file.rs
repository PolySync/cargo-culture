use super::super::file::search_manifest_and_workspace_dir_for_nonempty_file_name_match;
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
        search_manifest_and_workspace_dir_for_nonempty_file_name_match(
            &HAS_LICENSE_FILE,
            cargo_manifest_file_path,
            metadata,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use tempfile::tempdir;

    // TODO - Test for workspace style project edge cases

    #[test]
    fn happy_path() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("LICENSE");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a LICENSE file.")
            .expect("Could not write to target file");
        let rule = HasLicenseFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn additional_content_beyond_prefix_allowed() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("LICENSE-whatever.txt");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a LICENSE file.")
            .expect("Could not write to target file");
        let rule = HasLicenseFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn empty_license_file_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        {
            let file_path = dir.path().join("LICENSE");
            let mut f = File::create(file_path).expect("Could not make target file");
            f.write_all(b"").expect("Could not write emptiness to file");
            f.flush().expect("Could not flush file");
            f.sync_all().expect("Could not sync file");
        }
        let rule = HasLicenseFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn no_license_file_at_all_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let rule = HasLicenseFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }
}
