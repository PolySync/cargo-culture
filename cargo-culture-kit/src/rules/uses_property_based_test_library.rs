use super::{Rule, RuleContext, RuleOutcome};
use cargo_metadata::DependencyKind;
use regex::Regex;

/// Rule that asserts a good Rust project:
/// "Should be making an effort to use property based tests."
///
/// # Justification
///
/// Property based testing is an advanced testing technique
/// that combines pseudo-random test input generation with
/// checking desired behavioral characteristics.
///
/// Property based testing libraries provide tools to
/// generate many input values for a test, kind of like
/// a type-safe and typically shorter-running fuzz test.
///
/// Since property based tests encourage reasoning about
/// how input data affects the state-space of the program
/// and can help rapidly achieve more comprehensive correctness
/// assurances, this `Rule` recommends it here as important
/// for quality-attentive projects.
///
/// # See Also
///
/// * [proptest](https://github.com/AltSysrq/proptest)
/// * [quickcheck](https://github.com/BurntSushi/quickcheck)
/// * [suppositions](https://github.com/cstorey/suppositions)
///
///
/// # Caveats
///
/// This `Rule` presently only checks whether or not
/// a known property-based test library is present in the
/// project dependencies (or dev-dependencies). Actually
/// writing good tests is an exercise left to the reader.
#[derive(Debug, Default)]
pub struct UsesPropertyBasedTestLibrary;

lazy_static! {
    static ref USES_PROPERTY_BASED_TEST_LIBRARY: Regex =
        Regex::new(r"^(?i)(proptest|quickcheck|suppositions).*")
            .expect("Failed to create UsesPropertyBasedTestLibrary regex.");
}

impl Rule for UsesPropertyBasedTestLibrary {
    fn description(&self) -> &'static str {
        "Should be making an effort to use property based tests."
    }

    fn evaluate(&self, context: RuleContext) -> RuleOutcome {
        match *context.metadata {
            None => RuleOutcome::Undetermined,
            Some(ref m) => {
                if m.packages.is_empty() {
                    return RuleOutcome::Undetermined;
                }
                for package in &m.packages {
                    let has_pbt_dep = package
                        .dependencies
                        .iter()
                        .filter(|d| d.kind == DependencyKind::Development)
                        .any(|d| USES_PROPERTY_BASED_TEST_LIBRARY.is_match(&d.name));
                    if !has_pbt_dep {
                        return RuleOutcome::Failure;
                    }
                }
                RuleOutcome::Success
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn uses_property_based_test_library_happy_path_flat_project() {
        let dir = tempdir().expect("Failed to make a temp dir");
        write_package_cargo_toml(dir.path(), "proptest");
        write_clean_src_main_file(dir.path());
        let rule = UsesPropertyBasedTestLibrary::default();
        let VerbosityOutcomes {
            verbose,
            not_verbose,
        } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
        assert_eq!(RuleOutcome::Success, verbose.outcome);
        assert_eq!(RuleOutcome::Success, not_verbose.outcome);
    }

    prop_compose!{
        fn arb_pbt_dep()(name in r"(?i)(proptest|quickcheck|suppositions)") -> String {
            name
        }
    }

    proptest! {
        #[test]
        fn uses_property_based_test_library_generated(ref name in arb_pbt_dep()) {
            let dir = tempdir().expect("Failed to make a temp dir");
            write_package_cargo_toml(dir.path(), name);
            write_clean_src_main_file(dir.path());
            let rule = UsesPropertyBasedTestLibrary::default();
            let VerbosityOutcomes {
                verbose,
                not_verbose,
            } = execute_rule_against_project_dir_all_verbosities(dir.path(), &rule);
            assert_eq!(RuleOutcome::Success, verbose.outcome);
            assert_eq!(RuleOutcome::Success, not_verbose.outcome);
        }
    }

    fn write_package_cargo_toml(project_dir: &Path, extra_dev_dependency: &str) {
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

        writeln!(cargo_file, "{} = \"*\"", extra_dev_dependency)
            .expect("Could not write extra dev dep to Cargo.toml file");
    }

    fn write_clean_src_main_file(project_dir: &Path) {
        let src_dir = project_dir.join("src");
        create_dir_all(&src_dir).expect("Could not create src dir");
        let file_path = src_dir.join("main.rs");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(
            br##"//! Sample rust file for testing cargo-culture
fn hello() { println!("Hello"); }

fn main() { hello(); }

#[cfg(test)]
mod tests {
    use super::hello;
    #[test]
    fn hello_does_not_panic() {
        assert_eq!((), hello());
    }
}
        "##,
        ).expect("Could not write to target file");
    }
}
