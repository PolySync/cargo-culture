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

mod checklist;
mod file;
pub mod rules;

pub use checklist::{filter_to_requested_rules_by_description,
                    filter_to_requested_rules_from_checklist_file, find_extant_culture_file};
pub use rules::{default_rules, BuildsCleanlyWithoutWarningsOrErrors, CargoMetadataReadable,
                HasContinuousIntegrationFile, HasContributingFile, HasLicenseFile, HasReadmeFile,
                HasRustfmtFile, PassesMultipleTests, Rule, RuleOutcome,
                UsesPropertyBasedTestLibrary};

use cargo_metadata::Metadata;
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
    #[fail(display = "There was an error while attempting to resolve the desired set of rules to check: {}",
           _0)]
    UnderspecifiedRules(String),
    #[fail(display = "A described rule specified was not in the available set of Rule implementations: {}",
           rule_description)]
    RequestedRuleNotFound { rule_description: String },
    #[fail(display = "There was an error while attempting to print content to the output writer: {}",
           _0)]
    PrintOutputFailure(String),
}

/// Execute a `check_culture` run using the set of rules available from
/// `default_rules`.
///
/// See `check_culture` for more details.
pub fn check_culture_default<P: AsRef<Path>, W: Write>(
    cargo_manifest_file_path: P,
    verbose: bool,
    print_output: &mut W,
) -> Result<OutcomesByDescription, CheckError> {
    let rules = default_rules();
    let rule_refs = rules.iter().map(|r| r.as_ref()).collect::<Vec<&Rule>>();
    let descriptions: Vec<&str> = rules.iter().map(|r| r.description()).collect();
    let filtered_rules = filter_to_requested_rules_by_description(&rule_refs, &descriptions)?;
    check_culture(
        cargo_manifest_file_path,
        verbose,
        print_output,
        &filtered_rules,
    )
}

/// Given a set of `Rule`s, evaluate the rules
/// and produce a summary report of the rule outcomes.
///
/// Primary entry point for this library.
///
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
) -> Result<Option<Metadata>, CheckError> {
    let manifest_path: PathBuf = cargo_manifest_file_path.as_ref().to_path_buf();
    let metadata_result = cargo_metadata::metadata(Some(manifest_path.as_ref()));
    match metadata_result {
        Ok(m) => Ok(Some(m)),
        Err(e) => {
            if verbose && writeln!(print_output, "{}", e).is_err() {
                return Err(CheckError::PrintOutputFailure(
                    "Error reporting project's `cargo metadata`".to_string(),
                ));
            }
            Ok(None)
        }
    }
}

pub fn evaluate_rules<P: AsRef<Path>, W: Write, M: Borrow<Option<Metadata>>>(
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
        outcome_stats.unknown_count
    ).is_err()
    {
        return Err(CheckError::PrintOutputFailure(
            "Error printing culture check summary.".to_string(),
        ));
    };
    Ok(())
}

/// Map between the `description` of `Rule`s and the outcome of their execution
pub type OutcomesByDescription = HashMap<String, RuleOutcome>;

impl<T> From<T> for OutcomeStats
where
    T: Borrow<OutcomesByDescription>,
{
    fn from(full_outcomes: T) -> OutcomeStats {
        let mut stats = OutcomeStats::empty();
        for outcome in full_outcomes.borrow().values() {
            match outcome {
                RuleOutcome::Success => stats.success_count += 1,
                RuleOutcome::Failure => stats.fail_count += 1,
                RuleOutcome::Undetermined => stats.unknown_count += 1,
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

fn print_rule_evaluation<P: AsRef<Path>, W: Write, M: Borrow<Option<Metadata>>>(
    rule: &Rule,
    cargo_manifest_file_path: P,
    verbose: bool,
    metadata: M,
    print_output: &mut W,
) -> Result<RuleOutcome, CheckError> {
    if print_output
        .write_all(rule.description().as_bytes())
        .is_err()
    {
        return Err(CheckError::PrintOutputFailure(
            "Could not write rule name".to_string(),
        ));
    }
    if print_output.flush().is_err() {
        return Err(CheckError::PrintOutputFailure(
            "Could not flush output".to_string(),
        ));
    }
    let outcome = rule.evaluate(
        cargo_manifest_file_path.as_ref(),
        verbose,
        metadata.borrow(),
        print_output,
    );
    writeln!(print_output, " ... {}", summary_str(&outcome)).expect("Could not write rule outcome");
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
/// results
#[derive(Clone, Debug, PartialEq)]
pub struct OutcomeStats {
    pub success_count: usize,
    pub fail_count: usize,
    pub unknown_count: usize,
}

impl OutcomeStats {
    /// Convenience function to answer the simple question "is everything all
    /// right?" while providing no answer at all to the useful question
    /// "why or why not?"
    pub fn is_success(&self) -> bool {
        RuleOutcome::from(self) == RuleOutcome::Success
    }
}

impl<'a> From<&'a OutcomeStats> for RuleOutcome {
    fn from(stats: &'a OutcomeStats) -> Self {
        match (stats.success_count, stats.fail_count, stats.unknown_count) {
            (0, 0, 0) => RuleOutcome::Undetermined,
            (s, 0, 0) if s > 0 => RuleOutcome::Success,
            (_, 0, _) => RuleOutcome::Undetermined,
            (_, f, _) if f > 0 => RuleOutcome::Failure,
            _ => unreachable!(),
        }
    }
}

impl OutcomeStats {
    pub fn empty() -> Self {
        OutcomeStats {
            success_count: 0,
            fail_count: 0,
            unknown_count: 0,
        }
    }
}

/// A means of genericizing expected process exit code
pub trait ExitCode {
    fn exit_code(&self) -> i32;
}

impl ExitCode for RuleOutcome {
    fn exit_code(&self) -> i32 {
        match *self {
            RuleOutcome::Success => 0,
            RuleOutcome::Failure => 1,
            RuleOutcome::Undetermined => 2,
        }
    }
}

impl ExitCode for OutcomeStats {
    fn exit_code(&self) -> i32 {
        RuleOutcome::from(self).exit_code()
    }
}

impl ExitCode for OutcomesByDescription {
    fn exit_code(&self) -> i32 {
        OutcomeStats::from(self).exit_code()
    }
}

impl ExitCode for CheckError {
    fn exit_code(&self) -> i32 {
        match *self {
            CheckError::UnderspecifiedRules(_) => 3,
            CheckError::RequestedRuleNotFound { .. } => 4,
            CheckError::PrintOutputFailure(_) => 5,
        }
    }
}

impl<T, E> ExitCode for Result<T, E>
where
    T: ExitCode,
    E: ExitCode,
{
    fn exit_code(&self) -> i32 {
        match self {
            Ok(ref r) => r.exit_code(),
            Err(e) => e.exit_code(),
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
                        unknown in any::<usize>()) -> OutcomeStats {
            OutcomeStats { success_count: success, fail_count: fail, unknown_count: unknown }
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

        fn evaluate(&self, _: &Path, _: bool, _: &Option<Metadata>, _: &mut Write) -> RuleOutcome {
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
}
