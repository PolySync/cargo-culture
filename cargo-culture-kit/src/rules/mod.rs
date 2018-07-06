//! Provides the `Rule` trait and several implementations,
//! available through the `default_rules()` function.
mod builds_cleanly_without_warnings_or_errors;
mod cargo_metadata_readable;
mod has_continuous_integration_file;
mod has_contributing_file;
mod has_license_file;
mod has_readme_file;
mod has_rustfmt_file;
mod passes_multiple_tests;
mod under_source_control;
mod uses_property_based_test_library;

pub use self::builds_cleanly_without_warnings_or_errors::BuildsCleanlyWithoutWarningsOrErrors;
pub use self::cargo_metadata_readable::CargoMetadataReadable;
pub use self::has_continuous_integration_file::HasContinuousIntegrationFile;
pub use self::has_contributing_file::HasContributingFile;
pub use self::has_license_file::HasLicenseFile;
pub use self::has_readme_file::HasReadmeFile;
pub use self::has_rustfmt_file::HasRustfmtFile;
pub use self::passes_multiple_tests::PassesMultipleTests;
pub use self::under_source_control::UnderSourceControl;
pub use self::uses_property_based_test_library::UsesPropertyBasedTestLibrary;

use cargo_metadata::Metadata;
use std::fmt::Debug;
use std::io::Write;
use std::path::Path;

/// The result of a `Rule.evaluate` call.
///
/// Currently represented as a tri-valued flat enum rather than a `Result<bool,
/// Error>` to reduce the temptation to use a fancy error management scheme.
/// This is also to bring attention to 3rd party implementers that a
/// `RuleOutcome::Failure` is not an anomalous situation from the operational
/// standpoint of a `Rule` evaluation, and is distinct from a `RuleOutcome::
/// Undetermined` value.
#[derive(Clone, Debug, PartialEq)]
pub enum RuleOutcome {
    /// The Rule's `description` is definitely true for this project
    Success,
    /// The Rule's `description` definitely is not upheld for this project
    Failure,
    /// Something went wrong in the process of determining whether the Rule was
    /// upheld or not for this project. Let's admit that we don't know for
    /// sure one way or the other.
    Undetermined,
}

/// The core trait of this crate. A `Rule` describes an idiom or best-practice
/// for projects and provides a means of evaluating whether that rule of thumb
/// is being upheld.
pub trait Rule: Debug {
    /// The central tenet of this `Rule`. Serves as a **unique identifier** for
    /// Rule instances, as well as a human-readable summary of what this
    /// `Rule` means for a given project.
    fn description(&self) -> &str;

    /// Does the Rust project found at `cargo_manifest_path` uphold this
    /// `Rule`, as summarized in the `description`?
    fn evaluate(&self, context: RuleContext) -> RuleOutcome;
}

/// Parameter struct for the `Rule::evaluate` method.
/// Should provide the minimum information necessary for
/// project-level quality and completeness checks to be run.
pub struct RuleContext<'a> {
    /// The path of the Cargo.toml project file for the Rust
    /// project currently under evaluation.
    pub cargo_manifest_file_path: &'a Path,
    /// When true, `Rule` implementations should supply additional
    /// human-reader-oriented content by writing to `print_output`
    pub verbose: bool,
    /// Pre-parsed cargo metadata for the current project under evaluation.
    /// Ought to be `None` only when the cargo metadata retrieval or parsing
    /// fails.
    pub metadata: &'a Option<Metadata>,
    /// Output `Write` implementation intended for supplying optional
    /// textual content visible to the end-user.  `Rule` implementations
    /// may make use of this as they wish, the default convention is to only
    /// write extra content when `verbose == true`
    pub print_output: &'a mut Write,
}

/// Constructs new instances of the default `Rule`s
/// recommended as a starting point by the project maintainers.
pub fn default_rules() -> Vec<Box<Rule>> {
    vec![
        Box::new(CargoMetadataReadable::default()),
        Box::new(HasContributingFile::default()),
        Box::new(HasLicenseFile::default()),
        Box::new(HasReadmeFile::default()),
        Box::new(HasRustfmtFile::default()),
        Box::new(HasContinuousIntegrationFile::default()),
        Box::new(BuildsCleanlyWithoutWarningsOrErrors::default()),
        Box::new(PassesMultipleTests::default()),
        Box::new(UnderSourceControl::default()),
        Box::new(UsesPropertyBasedTestLibrary::default()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn default_rules_all_have_unique_descriptions() {
        let rules = default_rules();
        let mut set = HashSet::new();
        for r in &rules {
            set.insert(r.description().to_string());
        }
        assert_eq!(rules.len(), set.len());
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::{Rule, RuleContext, RuleOutcome};
    use cargo_metadata;
    use std::path::Path;

    pub struct VerbosityOutcomes {
        pub verbose: OutcomeCapture,
        pub not_verbose: OutcomeCapture,
    }

    pub struct OutcomeCapture {
        pub outcome: RuleOutcome,
        pub print_output: Vec<u8>,
    }

    pub fn execute_rule_against_project_dir_all_verbosities(
        project_dir: &Path,
        rule: &Rule,
    ) -> VerbosityOutcomes {
        VerbosityOutcomes {
            verbose: execute_rule_against_project_dir(project_dir, rule, true),
            not_verbose: execute_rule_against_project_dir(project_dir, rule, false),
        }
    }

    pub fn execute_rule_against_project_dir(
        project_dir: &Path,
        rule: &Rule,
        verbose: bool,
    ) -> OutcomeCapture {
        let cargo_manifest_file_path = project_dir.join("Cargo.toml");
        let metadata = cargo_metadata::metadata(Some(cargo_manifest_file_path.as_ref()))
            .map_err(|e| {
                println!("cargo_metadata error: {:?}", e);
                e
            })
            .ok();
        let mut print_output: Vec<u8> = Vec::new();
        let outcome = rule.evaluate(RuleContext {
            cargo_manifest_file_path: &cargo_manifest_file_path,
            verbose,
            metadata: &metadata,
            print_output: &mut print_output,
        });
        OutcomeCapture {
            outcome,
            print_output,
        }
    }
}
