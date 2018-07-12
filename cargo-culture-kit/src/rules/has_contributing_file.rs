use super::super::file::{
    find_nonempty_child_file, search_manifest_and_workspace_dir_for_nonempty_file_name_match,
};
use super::{Rule, RuleContext, RuleOutcome};
use regex::Regex;
use std::path::PathBuf;

/// Rule that asserts a good Rust project:
/// "Should have a CONTRIBUTING file in the project directory."
///
/// # Justification
///
/// A CONTRIBUTING file is a starting point for would-be collaborators
/// popularized in the open-source world. Even for closed-source projects, a
/// CONTRIBUTING file can be a gateway to developer-focused guidance, and thus
/// useful for on-boarding in a more targeted manner than the general README.
#[derive(Debug, Default)]
pub struct HasContributingFile;

lazy_static! {
    static ref HAS_CONTRIBUTING_FILE: Regex =
        Regex::new(r"^(?i)CONTRIBUTING").expect("Failed to create HasContributingFile regex.");
}

impl Rule for HasContributingFile {
    fn description(&self) -> &str {
        "Should have a CONTRIBUTING file in the project directory."
    }

    fn evaluate(&self, context: RuleContext) -> RuleOutcome {
        let initial_outcome = search_manifest_and_workspace_dir_for_nonempty_file_name_match(
            &HAS_CONTRIBUTING_FILE,
            context.cargo_manifest_file_path,
            context.metadata,
        );
        if initial_outcome == RuleOutcome::Success {
            return RuleOutcome::Success;
        }
        let github_dir = {
            let mut p = context.cargo_manifest_file_path.to_path_buf();
            p.pop();
            p.join(".github")
        };
        if find_nonempty_child_file(&HAS_CONTRIBUTING_FILE, &github_dir) == RuleOutcome::Success {
            return RuleOutcome::Success;
        }
        if let Some(ref metadata) = context.metadata {
            let workspace_github_dir = PathBuf::from(&metadata.workspace_root).join(".github");
            match find_nonempty_child_file(&HAS_CONTRIBUTING_FILE, &workspace_github_dir) {
                RuleOutcome::Success => RuleOutcome::Success,
                RuleOutcome::Failure | RuleOutcome::Undetermined => initial_outcome,
            }
        } else {
            initial_outcome
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn has_contributing_file_minimal_happy_path() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("CONTRIBUTING");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a CONTRIBUTING file.")
            .expect("Could not write to target file");
        let rule = HasContributingFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn has_contributing_file_in_dot_github_dir() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let github_dir_path = dir.path().join(".github");
        create_dir_all(&github_dir_path).expect("Could not make .github dir");
        let file_path = github_dir_path.join("CONTRIBUTING");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a CONTRIBUTING file.")
            .expect("Could not write to target file");
        let rule = HasContributingFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn has_contributing_file_fails_with_empty_dot_github_dir() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let github_dir_path = dir.path().join(".github");
        create_dir_all(github_dir_path).expect("Could not make .github dir");
        let rule = HasContributingFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn has_contributing_file_workspace_project_happy_path() {
        let workspace_dir = tempdir().expect("Failed to make a temp dir");
        create_workspace_cargo_toml(workspace_dir.path().join("Cargo.toml"));
        let kid_dir = workspace_dir.path().join("kid");
        create_dir_all(&kid_dir).expect("Could not make a kid project dir");
        write_package_cargo_toml(&kid_dir, None);
        write_clean_src_main_file(&kid_dir);

        let mut contributing_file = File::create(workspace_dir.path().join("CONTRIBUTING"))
            .expect("Could not make target file");
        contributing_file
            .write_all(b"Hello, I am a CONTRIBUTING file.")
            .expect("Could not write to target file");
        let rule = HasContributingFile::default();

        {
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(workspace_dir.path(), &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }
        {
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(&kid_dir, &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }
    }
    #[test]
    fn has_contributing_file_workspace_dot_github_happy_path() {
        let workspace_dir = tempdir().expect("Failed to make a temp dir");
        create_workspace_cargo_toml(workspace_dir.path().join("Cargo.toml"));
        let kid_dir = workspace_dir.path().join("kid");
        create_dir_all(&kid_dir).expect("Could not make a kid project dir");
        write_package_cargo_toml(&kid_dir, None);
        write_clean_src_main_file(&kid_dir);

        let github_dir_path = workspace_dir.path().join(".github");
        create_dir_all(&github_dir_path).expect("Could not make .github dir");
        let mut contributing_file =
            File::create(github_dir_path.join("CONTRIBUTING")).expect("Could not make target file");
        contributing_file
            .write_all(b"Hello, I am a CONTRIBUTING file.")
            .expect("Could not write to target file");
        let rule = HasContributingFile::default();

        {
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(workspace_dir.path(), &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }
        {
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(&kid_dir, &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }
    }

    #[test]
    fn additional_content_beyond_prefix_allowed() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("CONTRIBUTING-whatever.txt");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a CONTRIBUTING file.")
            .expect("Could not write to target file");
        let rule = HasContributingFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn empty_contributing_file_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        {
            let file_path = dir.path().join("CONTRIBUTING");
            let mut f = File::create(file_path).expect("Could not make target file");
            f.write_all(b"").expect("Could not write emptiness to file");
            f.flush().expect("Could not flush file");
            f.sync_all().expect("Could not sync file");
        }
        let rule = HasContributingFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn no_contributing_file_at_all_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let rule = HasContributingFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }
}
