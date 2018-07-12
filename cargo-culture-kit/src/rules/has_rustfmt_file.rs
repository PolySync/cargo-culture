use super::super::file::search_manifest_and_workspace_dir_for_nonempty_file_name_match;
use super::{Rule, RuleContext, RuleOutcome};
use regex::Regex;

/// Rule that asserts a good Rust project:
/// "Should have a rustfmt.toml file in the project directory."
///
/// # Justification
///
/// Code style linting is a valuable tool to enhance project-wide
/// consistency and readability for new developers.
/// `rustfmt` is the de-facto standard style linter in the Rust
/// community.
///
/// A rustfmt.toml file suggests that the project
/// maintainers are aware of the advantages of unified formatting,
/// and have taken the time to select the `rustfmt` rules that make
/// sense for this project, even if the choice is simply to use the
/// defaults.
#[derive(Debug, Default)]
pub struct HasRustfmtFile;

lazy_static! {
    static ref HAS_RUSTFMT_FILE: Regex =
        Regex::new(r"^\.?(legacy-)?rustfmt.toml$").expect("Failed to create HasRustfmtFile regex.");
}

impl Rule for HasRustfmtFile {
    fn description(&self) -> &'static str {
        "Should have a rustfmt.toml file in the project directory."
    }

    fn evaluate(&self, context: RuleContext) -> RuleOutcome {
        search_manifest_and_workspace_dir_for_nonempty_file_name_match(
            &HAS_RUSTFMT_FILE,
            context.cargo_manifest_file_path,
            context.metadata,
        )
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
    fn has_rustfmt_happy_path() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("rustfmt.toml");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a rustfmt file.")
            .expect("Could not write to target file");
        let rule = HasRustfmtFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn has_rustfmt_file_workspace_project_happy_path() {
        let base_dir = tempdir().expect("Failed to make a temp dir");
        create_workspace_cargo_toml(base_dir.path().join("Cargo.toml"));
        let subproject_dir = base_dir.path().join("kid");
        create_dir_all(&subproject_dir).expect("Could not create subproject dir");
        write_package_cargo_toml(&subproject_dir, None);
        write_clean_src_main_file(&subproject_dir);
        let mut rustfmt_file = File::create(base_dir.path().join(".rustfmt.toml"))
            .expect("Could not make target file");
        rustfmt_file
            .write_all(b"hard_tabs = true")
            .expect("Could not write to target file");
        let rule = HasRustfmtFile::default();
        {
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(base_dir.path(), &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }
        {
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(&subproject_dir, &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }
    }

    #[test]
    fn has_rustfmt_period_prefix_allowed() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join(".rustfmt.toml");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a rustfmt file.")
            .expect("Could not write to target file");
        let rule = HasRustfmtFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn has_rustfmt_legacy_prefix_allowed() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("legacy-rustfmt.toml");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a rustfmt file.")
            .expect("Could not write to target file");
        let rule = HasRustfmtFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn has_rustfmt_additional_suffices_disallowed() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("rustfmt.toml.whatever");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(b"Hello, I am a rustfmt file.")
            .expect("Could not write to target file");
        let rule = HasRustfmtFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn has_rustfmt_empty_rustfmt_file_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        {
            let file_path = dir.path().join("rustfmt.toml");
            let mut f = File::create(file_path).expect("Could not make target file");
            f.write_all(b"").expect("Could not write emptiness to file");
            f.flush().expect("Could not flush file");
            f.sync_all().expect("Could not sync file");
        }
        let rule = HasRustfmtFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn has_rustfmt_nonempty_blank_content_rustfmt_file_succeeds() {
        let dir = tempdir().expect("Failed to make a temp dir");
        {
            let file_path = dir.path().join("rustfmt.toml");
            let mut f = File::create(file_path).expect("Could not make target file");
            f.write_all(b"\n")
                .expect("Could not write a newline to file");
            f.flush().expect("Could not flush file");
            f.sync_all().expect("Could not sync file");
        }
        let rule = HasRustfmtFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn has_rustfmt_no_rustfmt_file_at_all_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let rule = HasRustfmtFile::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }
}
