use super::super::file::shallow_scan_project_dir_for_nonempty_file_name_match;
use super::{Rule, RuleContext, RuleOutcome};
use regex::Regex;

/// Rule that asserts a good Rust project:
/// "Should have a README.md file in the project directory."
///
/// # Justification
///
/// A README file is likely the first and last piece of documentation
/// people may read about a project.
#[derive(Debug, Default)]
pub struct HasReadmeFile;

lazy_static! {
    static ref HAS_README_FILE: Regex =
        Regex::new(r"^README\.?.*").expect("Failed to create HasReadmeFile regex.");
}

impl Rule for HasReadmeFile {
    fn description(&self) -> &'static str {
        "Should have a README.md file in the project directory."
    }

    fn evaluate(&self, context: RuleContext) -> RuleOutcome {
        shallow_scan_project_dir_for_nonempty_file_name_match(
            &HAS_README_FILE,
            context.cargo_manifest_file_path,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn has_readme_happy_path() {
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
    fn non_md_extension_acceptable() {
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
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }
}
