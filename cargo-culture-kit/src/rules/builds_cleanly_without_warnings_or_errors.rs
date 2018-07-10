use super::{Rule, RuleContext, RuleOutcome};
use cargo_metadata::Metadata;
use regex::Regex;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

/// Rule that asserts a good Rust project:
/// "Should `cargo clean` and `cargo build` without any warnings or errors."
///
/// # Justification
///
/// A Rust project striving for excellence and accessibility should be able to
/// be built "out of the box" with common tooling and do so without any
/// superfluous warnings or build errors. The `clean` step is necessary to get
/// complete error/warning messages repeatably.
///
/// While not every warning is appropriate to be absent from every project,
/// developers have the ability to thoughtfully silence warnings that are not
/// relevant to the present use case.
///
/// # Caveats
///
/// Though this rule makes an effort to avoid needless work by targeting
/// the `cargo clean` invocations to the project's own packages,
/// unless dependencies have been previously built, `evaluate` is likely
/// to take a while.
#[derive(Debug, Default)]
pub struct BuildsCleanlyWithoutWarningsOrErrors;

impl Rule for BuildsCleanlyWithoutWarningsOrErrors {
    fn description(&self) -> &'static str {
        "Should `cargo clean` and `cargo build` without any warnings or errors."
    }

    fn evaluate(&self, context: RuleContext) -> RuleOutcome {
        let cargo = get_cargo_command();
        let RuleContext {
            cargo_manifest_file_path,
            verbose,
            metadata,
            print_output,
        } = context;
        let packages_cleaned = clean_packages(
            &cargo,
            cargo_manifest_file_path,
            verbose,
            metadata,
            print_output,
        );
        if !packages_cleaned {
            return RuleOutcome::Failure;
        }
        let mut build_cmd = Command::new(&cargo);
        build_cmd.arg("build");
        build_cmd
            .arg("--manifest-path")
            .arg(cargo_manifest_file_path);
        build_cmd.arg("--message-format=json");
        let command_str = format!("{:?}", build_cmd);
        let build_output = match build_cmd.output() {
            Ok(o) => o,
            Err(_e) => {
                return RuleOutcome::Undetermined;
            }
        };
        if !build_output.status.success() {
            if verbose {
                let _ = writeln!(print_output, "Build command `{}` failed", command_str);
                if let Ok(s) = String::from_utf8(build_output.stdout) {
                    let _ = writeln!(print_output, "`{}` StdOut:\n{}\n\n", command_str, s);
                }
                if let Ok(s) = String::from_utf8(build_output.stderr) {
                    let _ = writeln!(print_output, "`{}` StdErr:\n{}\n\n", command_str, s);
                }
            }
            return RuleOutcome::Failure;
        }
        let stdout = match from_utf8(&build_output.stdout) {
            Ok(stdout) => stdout,
            Err(e) => {
                if verbose {
                    let _ = writeln!(
                        print_output,
                        "Reading stdout for command `{}` failed : {}",
                        command_str, e
                    );
                }
                return RuleOutcome::Undetermined;
            }
        };

        if WARNING_JSON.is_match(stdout) {
            if verbose {
                let _ = writeln!(
                    print_output,
                    "Found warnings in the cargo build command output:\n{}\n\n",
                    stdout
                );
            }
            return RuleOutcome::Failure;
        }
        RuleOutcome::Success
    }
}

lazy_static! {
    static ref WARNING_JSON: Regex = Regex::new(".*\"level\":\"warning\".*")
        .expect("Failed to create BuildsCleanlyWithoutWarningsOrErrors regex.");
}

fn clean_packages(
    cargo_command: &str,
    cargo_manifest_file_path: &Path,
    verbose: bool,
    metadata: &Option<Metadata>,
    print_output: &mut Write,
) -> bool {
    match *metadata {
        None => {
            if verbose {
                let _ = writeln!(
                    print_output,
                    "No metadata to discover which packages to clean."
                );
            }
            false
        }
        Some(ref m) if m.packages.is_empty() => {
            if verbose {
                let _ = writeln!(print_output, "No packages to clean.");
            }
            false
        }
        Some(ref m) => {
            let mut all_cleaned = true;
            for p in &m.packages {
                let cleaned = clean_package(
                    cargo_command,
                    &p.name,
                    cargo_manifest_file_path,
                    verbose,
                    print_output,
                );
                if !cleaned && verbose {
                    let _ = writeln!(print_output, "Could not clean package {} .", &p.name);
                }
                all_cleaned = all_cleaned && cleaned;
            }
            all_cleaned
        }
    }
}

fn clean_package(
    cargo_command: &str,
    package_name: &str,
    cargo_manifest_file_path: &Path,
    verbose: bool,
    print_output: &mut Write,
) -> bool {
    let mut clean_cmd = Command::new(&cargo_command);
    clean_cmd.arg("clean");
    clean_cmd
        .arg("--manifest-path")
        .arg(cargo_manifest_file_path);
    clean_cmd.arg("--package").arg(package_name);
    let clean_output = match clean_cmd.output() {
        Ok(o) => o,
        Err(e) => {
            if verbose {
                let _ = writeln!(print_output, "{}", e);
            }
            return false;
        }
    };
    clean_output.status.success()
}

fn get_cargo_command() -> String {
    ::std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rules::test_support::*;
    use std::fs::{create_dir_all, File};
    use tempfile::tempdir;

    #[test]
    fn builds_cleanly_happy_path_flat_project() {
        let dir = tempdir().expect("Failed to make a temp dir");
        write_package_cargo_toml(dir.path(), None);
        write_clean_src_main_file(dir.path());
        let rule = BuildsCleanlyWithoutWarningsOrErrors::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn builds_cleanly_fails_for_erroneous_main() {
        let dir = tempdir().expect("Failed to make a temp dir");
        write_package_cargo_toml(dir.path(), None);
        write_erroneous_src_main_file(dir.path());
        let rule = BuildsCleanlyWithoutWarningsOrErrors::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn builds_cleanly_fails_for_warningful_main() {
        let dir = tempdir().expect("Failed to make a temp dir");
        write_package_cargo_toml(dir.path(), None);
        write_warningful_src_main_file(dir.path());
        let rule = BuildsCleanlyWithoutWarningsOrErrors::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn builds_cleanly_happy_path_workspace_project() {
        let base_dir = tempdir().expect("Failed to make a temp dir");
        create_workspace_cargo_toml(base_dir.path().join("Cargo.toml"));
        let subproject_dir = base_dir.path().join("kid");
        create_dir_all(&subproject_dir).expect("Could not create subproject dir");
        write_package_cargo_toml(&subproject_dir, None);
        write_clean_src_main_file(&subproject_dir);
        let rule = BuildsCleanlyWithoutWarningsOrErrors::default();

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
    fn builds_cleanly_fails_for_workspace_project_with_warningful_main_in_subproject() {
        let base_dir = tempdir().expect("Failed to make a temp dir");
        create_workspace_cargo_toml(base_dir.path().join("Cargo.toml"));
        let subproject_dir = base_dir.path().join("kid");
        create_dir_all(&subproject_dir).expect("Could not create subproject dir");
        write_package_cargo_toml(&subproject_dir, None);
        write_warningful_src_main_file(&subproject_dir);
        let rule = BuildsCleanlyWithoutWarningsOrErrors::default();

        {
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(base_dir.path(), &rule);
            assert_eq!(RuleOutcome::Failure, verbose.outcome);
            assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
        }
        {
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(&subproject_dir, &rule);
            assert_eq!(RuleOutcome::Failure, verbose.outcome);
            assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
        }
    }

    #[test]
    fn builds_cleanly_fails_for_workspace_project_with_erroneous_main_in_subproject() {
        let base_dir = tempdir().expect("Failed to make a temp dir");
        create_workspace_cargo_toml(base_dir.path().join("Cargo.toml"));
        let subproject_dir = base_dir.path().join("kid");
        create_dir_all(&subproject_dir).expect("Could not create subproject dir");
        write_package_cargo_toml(&subproject_dir, None);
        write_erroneous_src_main_file(&subproject_dir);
        let rule = BuildsCleanlyWithoutWarningsOrErrors::default();

        {
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(base_dir.path(), &rule);
            assert_eq!(RuleOutcome::Failure, verbose.outcome);
            assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
        }
        {
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(&subproject_dir, &rule);
            assert_eq!(RuleOutcome::Failure, verbose.outcome);
            assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
        }
    }

    fn write_warningful_src_main_file(project_dir: &Path) {
        let src_dir = project_dir.join("src");
        create_dir_all(&src_dir).expect("Could not create src dir");
        let file_path = src_dir.join("main.rs");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(
            br##"//! Sample rust file for testing cargo-culture
fn hello() { println!("Hello"); }

fn main() { println!("Note we didn't use that function, which should cause a warning"); }
        "##,
        ).expect("Could not write to target file");
    }

    fn write_erroneous_src_main_file(project_dir: &Path) {
        let src_dir = project_dir.join("src");
        create_dir_all(&src_dir).expect("Could not create src dir");
        let file_path = src_dir.join("main.rs");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(
            br##"//! Sample rust file for testing cargo-culture
fn main() { totally_not_a_function(); }
        "##,
        ).expect("Could not write to target file");
    }

    #[test]
    fn builds_cleanly_empty_dir_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let rule = BuildsCleanlyWithoutWarningsOrErrors::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn builds_cleanly_non_toml_manifest_fails() {
        let dir = tempdir().expect("Failed to make a temp dir");
        {
            let cargo_path = dir.path().join("Cargo.toml");
            let mut cargo_file = File::create(cargo_path).expect("Could not make target file");
            cargo_file
                .write_all(br##"{"wat": true}"##)
                .expect("Could not write to Cargo.toml file");
        }
        write_clean_src_main_file(dir.path());
        let rule = BuildsCleanlyWithoutWarningsOrErrors::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }
}
