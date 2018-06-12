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

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use tempfile::tempdir;

    #[test]
    fn happy_path() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("README.md");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a README file.")
            .expect("Could not write to target file");
        let rule = HasReadmeFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn empty_readme_file_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        {
            let file_path = dir.path().join("README.md");
            let mut f = File::create(file_path).expect("Could not make target file");
            f.write_all(b"").expect("Could not write emptiness to file");
            f.flush().expect("Could not flush file");
            f.sync_all().expect("Could not sync file");
        }
        let rule = HasReadmeFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn no_readme_file_at_all_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let rule = HasReadmeFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn non_md_extension_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("README.txt");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a README file.")
            .expect("Could not write to target file");
        let rule = HasReadmeFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }
}
