use cargo_metadata::{DependencyKind, Metadata};
use regex::Regex;
use std::io::Write;
use std::path::Path;
use super::{Rule, RuleOutcome};

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

    fn evaluate(
        &self,
        _: &Path,
        _: bool,
        metadata: &Option<Metadata>,
        _: &mut Write,
    ) -> RuleOutcome {
        match *metadata {
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
