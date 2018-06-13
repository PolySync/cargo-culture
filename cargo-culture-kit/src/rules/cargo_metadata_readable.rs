use super::{Rule, RuleOutcome};
use cargo_metadata::Metadata;
use std::io::Write;
use std::path::Path;

#[derive(Default, Debug)]
pub struct CargoMetadataReadable;

impl Rule for CargoMetadataReadable {
    fn description(&self) -> &'static str {
        "Should have a well-formed Cargo.toml file readable by `cargo metadata`"
    }

    fn evaluate(
        &self,
        _: &Path,
        _: bool,
        metadata: &Option<Metadata>,
        _: &mut Write,
    ) -> RuleOutcome {
        match *metadata {
            None => RuleOutcome::Failure,
            Some(_) => RuleOutcome::Success,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use std::fs::{create_dir_all, File};
    use tempfile::tempdir;

    #[test]
    fn cargo_metadata_readable_happy_path_flat_project() {
        let dir = tempdir().expect("Failed to make a temp dir");
        {
            let cargo_path = dir.path().join("Cargo.toml");
            let mut cargo_file = File::create(cargo_path).expect("Could not make target file");
            cargo_file
                .write_all(
                    br##"[package]
name = "a_minimal_package"
version = "0.1.0"
authors = []

[dependencies]

[dev-dependencies]
        "##,
                )
                .expect("Could not write to Cargo.toml file");
        }
        write_src_lib_file(dir.path());
        let rule = CargoMetadataReadable::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn cargo_metadata_readable_happy_path_workspace_project() {
        let base_dir = tempdir().expect("Failed to make a temp dir");
        {
            let workspace_cargo_path = base_dir.path().join("Cargo.toml");
            let mut workspace_cargo_file =
                File::create(workspace_cargo_path).expect("Could not make workspace Cargo file");
            workspace_cargo_file
                .write_all(
                    br##"
[workspace]

members = [
  "kid"
]
        "##,
                )
                .expect("Could not write to workspace Cargo.toml file");
        }
        let subproject_dir = base_dir.path().join("kid");
        create_dir_all(&subproject_dir).expect("Could not create subproject dir");
        {
            let cargo_path = subproject_dir.join("Cargo.toml");
            let mut cargo_file = File::create(cargo_path).expect("Could not make target file");
            cargo_file
                .write_all(
                    br##"[package]
name = "kid"
version = "0.1.0"
authors = []

[dependencies]

[dev-dependencies]
        "##,
                )
                .expect("Could not write to Cargo.toml file");
            write_src_lib_file(&subproject_dir);
        }
        let rule = CargoMetadataReadable::default();

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

    fn write_src_lib_file(project_dir: &Path) {
        let src_dir = project_dir.join("src");
        create_dir_all(&src_dir).expect("Could not create src dir");
        let file_path = src_dir.join("lib.rs");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(
            br##"//! Sample rust file for testing cargo-culture
fn hello() { println!("Hello"); }
        "##,
        ).expect("Could not write to target file");
    }

    #[test]
    fn empty_dir_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let rule = CargoMetadataReadable::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn non_toml_manifest_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        {
            let cargo_path = dir.path().join("Cargo.toml");
            let mut cargo_file = File::create(cargo_path).expect("Could not make target file");
            cargo_file
                .write_all(br##"{"wat": true}"##)
                .expect("Could not write to Cargo.toml file");
        }
        write_src_lib_file(dir.path());
        let rule = CargoMetadataReadable::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }
}
