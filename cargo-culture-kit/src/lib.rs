//! cargo-culture-kit provides machinery for checking
//! project-level rules about Rust best practices.
//!
//! The primary function entry points are `check_culture` and
//! `check_culture_default`.
//!
//! The core trait is `Rule`, which represents a single project-level property
//! that has a clear description and can be checked.
//!
//! # Examples
//!
//! `check_culture_default` is the easiest way to get started,
//! as it provides a thin wrapper around the core `check_culture`
//! function in combination with the `Rule`s provided by the
//! `default_rules()` function.
//!
//! ```
//! use cargo_culture_kit::{check_culture_default, IsSuccess, OutcomeStats};
//! use std::path::PathBuf;
//!
//! let cargo_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
//! let verbose = false;
//!
//! let outcomes = check_culture_default(
//!     cargo_manifest, verbose, &mut std::io::stdout()
//!     )
//!     .expect("Unexpected trouble checking culture rules:");
//!
//! let stats = OutcomeStats::from(outcomes);
//! assert!(stats.is_success());
//! assert_eq!(stats.fail_count, 0);
//! assert_eq!(stats.undetermined_count, 0);
//! ```
//!
//! If you want to use a specific `Rule` or group of `Rule`s,
//! the `check_culture` function is the right place to look.
//!
//! ```
//! use cargo_culture_kit::{check_culture, IsSuccess, OutcomeStats,
//! HasLicenseFile}; use std::path::PathBuf;
//!
//! let rule = HasLicenseFile::default();
//! let cargo_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
//! let verbose = false;
//!
//! let outcomes = check_culture(
//!     cargo_manifest, verbose, &mut std::io::stdout(), &[&rule]
//!     )
//!     .expect("Unexpected trouble checking culture rules: ");
//!
//! let stats = OutcomeStats::from(outcomes);
//! assert!(stats.is_success());
//! assert_eq!(stats.success_count, 1);
//! assert_eq!(stats.fail_count, 0);
//! assert_eq!(stats.undetermined_count, 0);
//! ```
//!

#![deny(missing_docs)]

#[cfg(test)]
extern crate tempfile;

#[macro_use]
extern crate failure;

#[cfg(test)]
#[macro_use]
extern crate proptest;

#[macro_use]
extern crate lazy_static;

extern crate cargo_metadata;
extern crate colored;

extern crate regex;

mod file;

pub mod checklist;
pub mod exit_code;
pub mod rules;

pub use checklist::{
    filter_to_requested_rules_by_description, filter_to_requested_rules_from_checklist_file,
    find_extant_culture_file, FilterError, DEFAULT_CULTURE_CHECKLIST_FILE_NAME,
};
pub use exit_code::ExitCode;
pub use rules::{
    default_rules, BuildsCleanlyWithoutWarningsOrErrors, CargoMetadataReadable,
    HasContinuousIntegrationFile, HasContributingFile, HasLicenseFile, HasReadmeFile,
    HasRustfmtFile, PassesMultipleTests, Rule, RuleContext, RuleOutcome,
    UsesPropertyBasedTestLibrary,
};

pub use cargo_metadata::Metadata as CargoMetadata;
use colored::*;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Top-level error variants for what can go wrong with checking culture rules.
///
/// Note that individual rule outcomes for better or worse should *not* be
/// interpreted as erroneous.
#[derive(Debug, Clone, Eq, Fail, PartialEq, Hash)]
pub enum CheckError {
    #[fail(
        display = "There was an error while attempting to print {} to the output writer.", topic
    )]
    /// Failure during writing human-oriented textual content to an output
    /// `Write` instance.
    PrintOutputFailure {
        /// The sort of content that was failed to be written
        topic: &'static str,
    },
    /// Destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this hidden variant
    /// ensures users do not rely on exhaustive matching.
    #[doc(hidden)]
    #[fail(display = "A hidden variant to increase expansion flexibility")]
    __Nonexhaustive,
}

/// Execute a `check_culture` run using the set of rules available from
/// `default_rules`.
///
/// See `check_culture` for more details.
///
/// # Examples
///
/// ```
/// use cargo_culture_kit::{check_culture_default, IsSuccess, OutcomeStats};
/// use std::path::PathBuf;
///
/// let cargo_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
/// let verbose = false;
///
/// let outcomes = check_culture_default(
///                 cargo_manifest, verbose, &mut std::io::stdout())
///     .expect("Unexpected trouble checking culture rules:");
///
/// for (description, outcome) in &outcomes {
///     println!("For this project: {} had an outcome of {:?}", description, outcome);
/// }
///
/// let stats = OutcomeStats::from(outcomes);
/// assert!(stats.is_success());
/// assert_eq!(stats.fail_count, 0);
/// assert_eq!(stats.undetermined_count, 0);
/// ```
///
/// # Errors
///
/// Returns an error if the program cannot write to the supplied `print_output`
/// instance.
pub fn check_culture_default<P: AsRef<Path>, W: Write>(
    cargo_manifest_file_path: P,
    verbose: bool,
    print_output: &mut W,
) -> Result<OutcomesByDescription, CheckError> {
    let rules = default_rules();
    let rule_refs = rules.iter().map(|r| r.as_ref()).collect::<Vec<&Rule>>();
    check_culture(cargo_manifest_file_path, verbose, print_output, &rule_refs)
}

/// Given a set of `Rule`s, evaluate the rules
/// and produce a summary report of the rule outcomes.
///
/// Primary entry point for this library.
///
/// `cargo_manifest_file_path` should point to a project's extant `Cargo.toml`
/// file. Either a crate-level or a workspace-level toml file should work.
///
/// `verbose` controls whether or not to produce additional human-readable
/// reporting.
///
/// `print_output` is the `Write` instance where `Rule` evaluation summaries
/// are printed, as well as the location where `verbose` content may be dumped.
/// `&mut std::io::stdout()` is a common instance used by non-test applications.
///
/// `rules` is the complete set of `Rule` instances which will be evaluated for
/// the project specified by `cargo_manifest_file_path`.
///
/// # Examples
///
/// ```
/// use cargo_culture_kit::{check_culture, IsSuccess, OutcomeStats,
/// HasLicenseFile}; use std::path::PathBuf;
///
/// let rule = HasLicenseFile::default();
/// let cargo_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
/// let verbose = false;
///
/// let outcomes = check_culture(cargo_manifest, verbose, &mut
/// std::io::stdout(),     &[&rule])
///     .expect("Unexpected trouble checking culture rules: ");
///
/// let stats = OutcomeStats::from(outcomes);
/// assert!(stats.is_success());
/// assert_eq!(stats.success_count, 1);
/// assert_eq!(stats.fail_count, 0);
/// assert_eq!(stats.undetermined_count, 0);
/// ```
///
/// # Errors
///
/// Returns an error if the program cannot write to the supplied `print_output`
/// instance.
pub fn check_culture<P: AsRef<Path>, W: Write>(
    cargo_manifest_file_path: P,
    verbose: bool,
    print_output: &mut W,
    rules: &[&Rule],
) -> Result<OutcomesByDescription, CheckError> {
    let metadata_option =
        read_cargo_metadata(cargo_manifest_file_path.as_ref(), verbose, print_output)?;
    let outcomes = evaluate_rules(
        cargo_manifest_file_path.as_ref(),
        verbose,
        &metadata_option,
        print_output,
        rules,
    )?;
    print_outcome_stats(&outcomes, print_output)?;
    Ok(outcomes)
}

fn read_cargo_metadata<P: AsRef<Path>, W: Write>(
    cargo_manifest_file_path: P,
    verbose: bool,
    print_output: &mut W,
) -> Result<Option<CargoMetadata>, CheckError> {
    let manifest_path: PathBuf = cargo_manifest_file_path.as_ref().to_path_buf();
    let metadata_result = cargo_metadata::metadata(Some(manifest_path.as_ref()));
    match metadata_result {
        Ok(m) => Ok(Some(m)),
        Err(e) => {
            if verbose && writeln!(print_output, "cargo metadata problem: {}", e).is_err() {
                return Err(CheckError::PrintOutputFailure {
                    topic: "cargo metadata",
                });
            }
            Ok(None)
        }
    }
}

fn evaluate_rules<P: AsRef<Path>, W: Write, M: Borrow<Option<CargoMetadata>>>(
    cargo_manifest_file_path: P,
    verbose: bool,
    metadata: M,
    print_output: &mut W,
    rules: &[&Rule],
) -> Result<OutcomesByDescription, CheckError> {
    let mut outcomes = OutcomesByDescription::new();
    for rule in rules {
        let outcome = print_rule_evaluation(
            *rule,
            cargo_manifest_file_path.as_ref(),
            verbose,
            metadata.borrow(),
            print_output,
        );
        outcomes.insert(rule.description().to_owned(), outcome?);
    }
    Ok(outcomes)
}

fn print_outcome_stats<W: Write>(
    outcomes: &OutcomesByDescription,
    mut print_output: W,
) -> Result<(), CheckError> {
    let outcome_stats: OutcomeStats = outcomes.into();
    let conclusion = if outcome_stats.is_success() {
        "ok".green()
    } else {
        "FAILED".red()
    };
    if writeln!(
        print_output,
        "culture result: {}. {} passed. {} failed. {} undetermined.",
        conclusion,
        outcome_stats.success_count,
        outcome_stats.fail_count,
        outcome_stats.undetermined_count
    ).is_err()
    {
        return Err(CheckError::PrintOutputFailure {
            topic: "culture check summary",
        });
    };
    Ok(())
}

/// Map between the `description` of `Rule`s and the outcome of their execution
pub type OutcomesByDescription = HashMap<String, RuleOutcome>;

/// Trait for summarizing whether the outcome of culture
/// checking was a total success for any of
/// the various levels of outcome aggregation
pub trait IsSuccess {
    /// Convenience function to answer the simple question "is everything all
    /// right?" while providing no answer at all to the useful question
    /// "why or why not?"
    fn is_success(&self) -> bool;

    /// Panic if `is_success()` returns false for this instance
    fn assert_success(&self) {
        assert!(self.is_success());
    }
}

impl IsSuccess for RuleOutcome {
    fn is_success(&self) -> bool {
        if let RuleOutcome::Success = *self {
            true
        } else {
            false
        }
    }
}

impl IsSuccess for OutcomesByDescription {
    fn is_success(&self) -> bool {
        OutcomeStats::from(self).is_success()
    }
}

impl IsSuccess for OutcomeStats {
    fn is_success(&self) -> bool {
        RuleOutcome::from(self) == RuleOutcome::Success
    }
}

impl<T> From<T> for OutcomeStats
where
    T: Borrow<OutcomesByDescription>,
{
    fn from(full_outcomes: T) -> OutcomeStats {
        let mut stats = OutcomeStats::default();
        for outcome in full_outcomes.borrow().values() {
            match outcome {
                RuleOutcome::Success => stats.success_count += 1,
                RuleOutcome::Failure => stats.fail_count += 1,
                RuleOutcome::Undetermined => stats.undetermined_count += 1,
            }
        }
        stats
    }
}
impl<T> From<T> for RuleOutcome
where
    T: Borrow<OutcomesByDescription>,
{
    fn from(full_outcomes: T) -> Self {
        let stats: OutcomeStats = full_outcomes.into();
        (&stats).into()
    }
}

fn print_rule_evaluation<P: AsRef<Path>, W: Write, M: Borrow<Option<CargoMetadata>>>(
    rule: &Rule,
    cargo_manifest_file_path: P,
    verbose: bool,
    metadata: M,
    print_output: &mut W,
) -> Result<RuleOutcome, CheckError> {
    if print_output
        .write_all(rule.description().as_bytes())
        .and_then(|_| print_output.flush())
        .is_err()
    {
        return Err(CheckError::PrintOutputFailure {
            topic: "rule description",
        });
    }
    let outcome = rule.evaluate(RuleContext {
        cargo_manifest_file_path: cargo_manifest_file_path.as_ref(),
        verbose,
        metadata: metadata.borrow(),
        print_output,
    });
    if writeln!(print_output, " ... {}", summary_str(&outcome)).is_err() {
        return Err(CheckError::PrintOutputFailure {
            topic: "rule evaluation outcome",
        });
    }
    Ok(outcome)
}

fn summary_str<T: Borrow<RuleOutcome>>(outcome: T) -> colored::ColoredString {
    match *outcome.borrow() {
        RuleOutcome::Success => "ok".green(),
        RuleOutcome::Failure => "FAILED".red(),
        RuleOutcome::Undetermined => "UNDETERMINED".red(),
    }
}

/// Summary of result statistics generated from aggregating `RuleOutcome`s
/// results for multiple Rule evaluations
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OutcomeStats {
    /// The number of `RuleOutcome::Success` instances observed
    pub success_count: usize,
    /// The number of `RuleOutcome::Failure` instances observed
    pub fail_count: usize,
    /// The number of `RuleOutcome::Undetermined` instances observed
    pub undetermined_count: usize,
}

impl<'a> From<&'a OutcomeStats> for RuleOutcome {
    fn from(stats: &'a OutcomeStats) -> Self {
        match (
            stats.success_count,
            stats.fail_count,
            stats.undetermined_count,
        ) {
            (0, 0, 0) => RuleOutcome::Undetermined,
            (s, 0, 0) if s > 0 => RuleOutcome::Success,
            (_, 0, _) => RuleOutcome::Undetermined,
            (_, f, _) if f > 0 => RuleOutcome::Failure,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection::VecStrategy;
    use proptest::prelude::*;

    fn arb_rule_outcome() -> BoxedStrategy<RuleOutcome> {
        prop_oneof![
            Just(RuleOutcome::Success),
            Just(RuleOutcome::Undetermined),
            Just(RuleOutcome::Failure),
        ].boxed()
    }

    prop_compose! {
        fn arb_stats()(success in any::<usize>(),
                       fail in any::<usize>(),
                        undetermined in any::<usize>()) -> OutcomeStats {
            OutcomeStats {
                success_count: success,
                fail_count: fail,
                undetermined_count: undetermined
            }
        }
    }

    prop_compose! {
        fn arb_predetermined_rule()(fixed_outcome in arb_rule_outcome(),
                                    description in ".*") -> PredeterminedOutcomeRule {
            PredeterminedOutcomeRule { outcome: fixed_outcome,
            description: description.into_boxed_str() }
        }
    }

    prop_compose! {
        fn arb_rule()(rule in arb_predetermined_rule()) -> Box<Rule> {
            let b: Box<Rule> = Box::new(rule);
            b
        }
    }

    fn arb_vec_of_rules() -> VecStrategy<BoxedStrategy<Box<Rule>>> {
        prop::collection::vec(arb_rule(), 0..100)
    }

    #[derive(Clone, Debug, PartialEq)]
    struct PredeterminedOutcomeRule {
        outcome: RuleOutcome,
        description: Box<str>,
    }

    impl Rule for PredeterminedOutcomeRule {
        fn description(&self) -> &str {
            self.description.as_ref()
        }

        fn evaluate(&self, _context: RuleContext) -> RuleOutcome {
            self.outcome.clone()
        }
    }

    proptest! {
        #[test]
        fn outcome_stats_to_rule_outcome_never_panics(ref stats in arb_stats()) {
            let _rule_outcome:RuleOutcome = RuleOutcome::from(stats);
        }

        #[test]
        fn piles_of_fixed_outcome_rules_evaluable(ref verbose in any::<bool>(),
                                                  ref vec_of_rules in arb_vec_of_rules()) {
            let mut v:Vec<u8> = Vec::new();
            let _outcome:OutcomeStats = evaluate_rules(
                                           Path::new("./Cargo.toml"), *verbose, &None,
                                           &mut v,
                                           vec_of_rules.iter()
                                               .map(|r| r.as_ref())
                                               .collect::<Vec<&Rule>>()
                                               .as_slice()
                                           ).expect("Expect no trouble with eval").into();
        }
    }

    #[allow(dead_code)]
    #[derive(Clone, Debug, Default, PartialEq)]
    struct IsProjectAtALuckyTime;

    #[allow(dead_code)]
    impl Rule for IsProjectAtALuckyTime {
        fn description(&self) -> &str {
            "Should be lucky enough to only be tested at specific times."
        }

        fn evaluate(&self, _context: RuleContext) -> RuleOutcome {
            use std::time::{SystemTime, UNIX_EPOCH};
            let since_the_epoch = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(t) => t,
                Err(_) => return RuleOutcome::Undetermined,
            };
            if since_the_epoch.as_secs() % 2 == 0 {
                RuleOutcome::Success
            } else {
                RuleOutcome::Failure
            }
        }
    }

    #[test]
    fn sanity_check_a_silly_rule_for_the_readme() {
        let context = RuleContext {
            cargo_manifest_file_path: &PathBuf::from("Cargo.toml"),
            verbose: true,
            metadata: &None,
            print_output: &mut Vec::new(),
        };
        let _ = IsProjectAtALuckyTime::default().evaluate(context);
    }
}
