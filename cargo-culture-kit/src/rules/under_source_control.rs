use super::{Rule, RuleContext, RuleOutcome};
use std::path::Path;

/// Rule that asserts a good Rust project:
/// "Should be under source control"
///
/// # Justification
///
/// Source control, a.k.a. version control, is essential for the coordinated
/// development of software projects.
///
/// # Caveats
///
/// The current implementation does a surface level check for the
/// presence of hidden metadata subdirectories associated with popular
/// version control systems:
///
/// * git
/// * mercurial (hg)
/// * svn
/// * bazaar
/// * darcs
///
#[derive(Debug, Default)]
pub struct UnderSourceControl;

const VC_SUBDIRS: &[&str] = &[".git", ".hg", ".bzr", ".svn", "_darcs"];

impl Rule for UnderSourceControl {
    fn description(&self) -> &str {
        "Should be under source control."
    }

    fn evaluate(&self, context: RuleContext) -> RuleOutcome {
        if AncestorDirs::from_file(context.cargo_manifest_file_path)
            .any(|dir| VC_SUBDIRS.iter().any(|subdir| dir.join(subdir).is_dir()))
        {
            RuleOutcome::Success
        } else {
            RuleOutcome::Failure
        }
    }
}

struct AncestorDirs<'p> {
    next: Option<&'p Path>,
}

impl<'p> AncestorDirs<'p> {
    fn from_file(file_path: &'p Path) -> AncestorDirs<'p> {
        AncestorDirs {
            next: file_path.parent(),
        }
    }
}

impl<'p> Iterator for AncestorDirs<'p> {
    type Item = &'p Path;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        self.next = match next {
            Some(path) => path.parent(),
            None => None,
        };
        next
    }
}
#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use std::fs::create_dir_all;
    use tempfile::tempdir;

    #[test]
    fn not_under_source_control_project_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        write_package_cargo_toml(dir.path(), None);
        let rule = UnderSourceControl::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn not_under_source_control_project_with_dummy_metadata_subdir_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        create_dir_all(dir.path().join(".my_unrecognized_vcs"))
            .expect("Failed to make a dummy subdir");
        write_package_cargo_toml(dir.path(), None);
        let rule = UnderSourceControl::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn under_source_control_project_succeeds_for_all_hint_subdir() {
        for subdir_name in VC_SUBDIRS {
            let dir = tempdir().expect("Failed to make a temp dir");
            create_dir_all(dir.path().join(subdir_name)).expect("Failed to make a named subdir");
            write_package_cargo_toml(dir.path(), None);
            let rule = UnderSourceControl::default();
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }
    }

    #[test]
    fn under_source_control_at_workspace_level_project_succeeds_for_all_hint_subdir() {
        for subdir_name in VC_SUBDIRS {
            let workspace_dir = tempdir().expect("Failed to make a temp dir");
            create_workspace_cargo_toml(workspace_dir.path().join("Cargo.toml"));
            create_dir_all(workspace_dir.path().join(subdir_name))
                .expect("Failed to make a named vc subdir");
            let kid_dir = workspace_dir.path().join("kid");
            create_dir_all(&kid_dir).expect("Could not make a kid project dir");
            write_package_cargo_toml(&kid_dir, None);
            let rule = UnderSourceControl::default();
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(workspace_dir.path(), &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }
    }
}
