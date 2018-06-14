use super::{Rule, RuleOutcome};
use cargo_metadata::Metadata;
use regex::Regex;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

/// Rule that asserts a good Rust project:
/// "Project should have multiple tests which pass."
///
/// # Justification
///
/// Some degree of automated testing is necessary for nearly all code,
/// and Rust makes adding tests nearly painless. The exact number
/// of tests is highly situational, but there should be more than
/// one, as even brand-new `cargo` library projects are supplied with
/// a dummy test by default.
///
/// # Caveats
///
/// This rule will actually attempt to run a project's tests through
/// `cargo test`. If this `Rule` is executed before the project has
/// been built or tested at all, the process of acquiring dependencies
/// and building them may take a while.
#[derive(Default, Debug)]
pub struct PassesMultipleTests;

lazy_static! {
    static ref TEST_RESULT_NUM_PASSED: Regex =
        Regex::new(r"(?m)^test result: ok. (?P<num_passed>\d+) passed;")
            .expect("Failed to create regex for PassesMultipleTests.");
}

impl Rule for PassesMultipleTests {
    fn description(&self) -> &'static str {
        "Project should have multiple tests which pass."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        verbose: bool,
        _: &Option<Metadata>,
        print_output: &mut Write,
    ) -> RuleOutcome {
        let mut test_cmd = Command::new(&get_cargo_command());
        test_cmd
            .arg("test")
            .arg("--manifest-path")
            .arg(cargo_manifest_file_path)
            .arg("--message-format")
            .arg("json")
            .env("CARGO_CULTURE_TEST_RECURSION_BUSTER", "true");
        let test_output = match test_cmd.output() {
            Ok(o) => o,
            Err(_) => return RuleOutcome::Failure,
        };

        if let Ok(s) = from_utf8(&test_output.stdout) {
            for num_passed_capture in TEST_RESULT_NUM_PASSED.captures_iter(s) {
                match num_passed_capture
                    .name("num_passed")
                    .map(|num_passed_str| num_passed_str.as_str().parse::<usize>())
                {
                    Some(Ok(num_passed)) if num_passed > 1usize => return RuleOutcome::Success,
                    _ => (),
                }
            }
            RuleOutcome::Failure
        } else {
            if verbose {
                let _ = writeln!(
                    print_output,
                    "Failed to interpret `cargo test` output as utf8 for parsing."
                );
            }
            RuleOutcome::Undetermined
        }
    }
}

fn get_cargo_command() -> String {
    ::std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"))
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use std::fs::{create_dir_all, File};
    use tempfile::tempdir;

    #[test]
    fn passes_multiple_tests_happy_path_flat_project() {
        let dir = tempdir().expect("Failed to make a temp dir");
        write_package_cargo_toml(dir.path());
        write_lib_file_with_dummy_tests(dir.path(), 2);
        let rule = PassesMultipleTests::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn passes_multiple_tests_more_specifically_ten_in_a_flat_project_succeeds() {
        let dir = tempdir().expect("Failed to make a temp dir");
        write_package_cargo_toml(dir.path());
        write_lib_file_with_dummy_tests(dir.path(), 10);
        let rule = PassesMultipleTests::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    #[test]
    fn passes_multiple_tests_fails_when_zero_tests_present() {
        let dir = tempdir().expect("Failed to make a temp dir");
        write_package_cargo_toml(dir.path());
        write_lib_file_with_dummy_tests(dir.path(), 0);
        let rule = PassesMultipleTests::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    #[test]
    fn passes_multiple_tests_fails_when_only_one_test_present() {
        let dir = tempdir().expect("Failed to make a temp dir");
        write_package_cargo_toml(dir.path());
        write_lib_file_with_dummy_tests(dir.path(), 1);
        let rule = PassesMultipleTests::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Failure, verbose.outcome);
        assert_eq!(RuleOutcome::Failure, not_verbose.outcome);
    }

    fn write_package_cargo_toml(project_dir: &Path) {
        let cargo_path = project_dir.join("Cargo.toml");
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
    }

    fn write_lib_file_with_dummy_tests(project_dir: &Path, num_tests: usize) {
        let src_dir = project_dir.join("src");
        create_dir_all(&src_dir).expect("Could not create src dir");
        let file_path = src_dir.join("main.rs");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(
            br##"//! Sample rust file for testing cargo-culture
fn hello() { println!("Hello"); }

#[cfg(test)]
mod tests {
    use super::hello;
        "##,
        ).expect("Could not write to target file");
        for i in 0..num_tests {
            writeln!(
                file,
                "#[test] fn dummy_test_{}() {{ assert_eq!(hello(), ()); }}",
                i
            ).expect("Could not write dummy test");
        }
        file.write_all(b"\n}\n")
            .expect("Could not write end of tests to target file")
    }
}
