use super::super::file::search_manifest_and_workspace_dir_for_nonempty_file_name_match;
use super::{Rule, RuleContext, RuleOutcome};
use regex::Regex;

/// Rule that asserts a good Rust project:
/// "Should have a file suggesting the use of a continuous integration system."
///
/// # Justification
///
/// Continuous integration can reduce the odds of project functionality
/// regression and several options are available to make this process
/// accessible for Rust projects.
///
/// See also: https://github.com/japaric/trust
#[derive(Default, Debug)]
pub struct HasContinuousIntegrationFile;

lazy_static! {
    static ref HAS_CONTINUOUS_INTEGRATION_FILE: Regex =
        Regex::new(r"^(?i)(appveyor|\.appveyor|\.drone|\.gitlab-ci|\.travis)\.ya?ml$")
            .expect("Failed to create HasContinuousIntegrationFile regex.");
}

impl Rule for HasContinuousIntegrationFile {
    fn description(&self) -> &'static str {
        "Should have a file suggesting the use of a continuous integration system."
    }

    fn evaluate(&self, context: RuleContext) -> RuleOutcome {
        search_manifest_and_workspace_dir_for_nonempty_file_name_match(
            &HAS_CONTINUOUS_INTEGRATION_FILE,
            context.cargo_manifest_file_path,
            context.metadata,
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

    fn manual_allowed_set() -> Vec<&'static str> {
        vec![
            "appveyor.yml",
            ".appveyor.yml",
            ".drone.yml",
            ".gitlab-ci.yml",
            ".travis.yml",
            "appveyor.yaml",
            ".appveyor.yaml",
            ".drone.yaml",
            ".gitlab-ci.yaml",
            ".travis.yaml",
        ]
    }

    #[test]
    fn has_continuous_integration_file_happy_paths() {
        for a in manual_allowed_set().iter() {
            let dir = tempdir().expect("Failed to make a temp dir");
            let file_path = dir.path().join(a);
            let mut file = File::create(file_path).expect("Could not make target file");
            file.write_all(b"Hello, I am a CI file.")
                .expect("Could not write to target file");
            let rule = HasContinuousIntegrationFile::default();
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }
    }

    prop_compose! {

        fn arb_ci_file_name()(file_name in r"(?i)(appveyor|\.appveyor|\.drone|\.gitlab-ci|\.travis)\.ya?ml") -> String {
            file_name
        }
    }

    proptest! {
        #[test]
        fn has_continuous_integration_file_generated(ref file_name in arb_ci_file_name()) {
            let dir = tempdir().expect("Failed to make a temp dir");
            let file_path = dir.path().join(file_name);
            let mut file = File::create(file_path).expect("Could not make target file");
            file.write_all(b"Hello, I am a CI file.")
                .expect("Could not write to target file");
            let rule = HasContinuousIntegrationFile::default();
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }

        #[test]
        fn has_continuous_integration_additional_suffices_disallowed(
                ref base_name in arb_ci_file_name(),
                ref suffix in r"(\.extra|\.toml|\.yml)") {
            let dir = tempdir().expect("Failed to make a temp dir");
            let file_path = dir.path().join(format!("{}{}", base_name, suffix));
            let mut file = File::create(file_path).expect("Could not make target file");
            file.write_all(b"Hello, I am a CI file.")
                .expect("Could not write to target file");
            let rule = HasContinuousIntegrationFile::default();
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
            assert_eq!(RuleOutcome::Failure, verbose.outcome);
            assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
        }

        #[test]
        fn has_continuous_integration_additional_prefices_disallowed(
                ref base_name in arb_ci_file_name(),
                ref prefix in r"(a-Z|legacy)") {
            let dir = tempdir().expect("Failed to make a temp dir");
            let file_path = dir.path().join(format!("{}{}", prefix, base_name));
            let mut file = File::create(file_path).expect("Could not make target file");
            file.write_all(b"Hello, I am a CI file.")
                .expect("Could not write to target file");
            let rule = HasContinuousIntegrationFile::default();
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
            assert_eq!(RuleOutcome::Failure, verbose.outcome);
            assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
        }
    }

    #[test]
    fn has_continuous_integration_empty_ci_file_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        {
            let file_path = dir.path().join(
                manual_allowed_set()
                    .first()
                    .expect("Could not get first allowed option"),
            );
            let mut f = File::create(file_path).expect("Could not make target file");
            f.write_all(b"").expect("Could not write emptiness to file");
            f.flush().expect("Could not flush file");
            f.sync_all().expect("Could not sync file");
        }
        let rule = HasContinuousIntegrationFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn has_continuous_integration_no_ci_file_at_all_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let rule = HasContinuousIntegrationFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }
}
