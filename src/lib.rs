#[cfg(test)]
extern crate tempdir;

#[cfg(test)]
#[macro_use]
extern crate proptest;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate structopt;

extern crate cargo_metadata;
extern crate colored;

extern crate regex;

mod build_infra;
mod collaboration;
mod file;
pub mod rule;

use build_infra::*;
use collaboration::*;
pub use rule::*;

use cargo_metadata::Metadata;
use colored::*;
use std::borrow::Borrow;
use std::io::Write;
use std::path::PathBuf;

pub fn default_rules() -> Vec<Box<Rule>> {
    vec![
        Box::new(CargoMetadataReadable::default()),
        Box::new(HasContributingFile::default()),
        Box::new(HasLicenseFile::default()),
        Box::new(HasReadmeFile::default()),
        Box::new(HasRustfmtFile::default()),
        Box::new(BuildsCleanlyWithoutWarningsOrErrors::default()),
        Box::new(HasContinuousIntegrationFile::default()),
        Box::new(UsesPropertyBasedTestLibrary::default()),
        Box::new(PassesMultipleTests::default()),
    ]
}

pub fn check_culture<O: Borrow<Opt>, W: Write>(opt: O, mut print_output: W) -> OutcomeStats {
    let rules: Vec<Box<Rule>> = default_rules();

    let metadata_option = read_cargo_metadata(opt.borrow(), &mut print_output);
    let outcome_stats = evaluate_rules(
        opt.borrow(),
        &mut print_output,
        rules.as_slice(),
        &metadata_option,
    );
    let conclusion = if outcome_stats.is_success() {
        "ok".green()
    } else {
        "FAILED".red()
    };
    writeln!(
        print_output,
        "culture result: {}. {} passed. {} failed. {} undetermined.",
        conclusion,
        outcome_stats.success_count,
        outcome_stats.fail_count,
        outcome_stats.unknown_count
    ).expect("Error reporting culture check summary.");

    outcome_stats
}

fn read_cargo_metadata<O: Borrow<Opt>, W: Write>(opt: O, print_output: &mut W) -> Option<Metadata> {
    // TODO - will need to do some more forgiving custom metadata parsing to deal
    // with changes in cargo metadata format -- the current crate assumes
    // you're on a recent nightly, where workspace_root has been added
    let manifest_path: PathBuf = opt.borrow().manifest_path.clone();
    let metadata_result = cargo_metadata::metadata(Some(manifest_path.as_ref()));
    match metadata_result {
        Ok(m) => Some(m),
        Err(e) => {
            if opt.borrow().verbose {
                writeln!(print_output, "{}", e)
                    .expect("Error reporting project's `cargo metadata`");
            }
            None
        }
    }
}

pub fn evaluate_rules<O: Borrow<Opt>, W: Write, M: Borrow<Option<Metadata>>>(
    opt: O,
    print_output: &mut W,
    rules: &[Box<Rule>],
    metadata: M,
) -> OutcomeStats {
    let mut stats = OutcomeStats::empty();
    for rule in rules {
        let outcome = rule.evaluate(opt.borrow(), metadata.borrow());
        print_outcome(rule.as_ref(), &outcome, print_output);
        match outcome {
            RuleOutcome::Success => stats.success_count += 1,
            RuleOutcome::Failure => stats.fail_count += 1,
            RuleOutcome::Undetermined => stats.unknown_count += 1,
        }
    }
    stats
}

fn print_outcome(rule: &Rule, outcome: &RuleOutcome, output: &mut Write) {
    writeln!(
        output,
        "{} ... {}",
        rule.catch_phrase(),
        summary_str(outcome)
    ).expect("Could not write rule outcome");
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

impl<T> From<T> for RuleOutcome
where
    T: Borrow<OutcomeStats>,
{
    fn from(stats: T) -> Self {
        let s = stats.borrow();
        match (s.success_count, s.fail_count, s.unknown_count) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection::VecStrategy;
    use proptest::prelude::*;
    use std::path::PathBuf;

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
                                    catch_phrase in ".*") -> PredeterminedOutcomeRule {
            PredeterminedOutcomeRule { outcome: fixed_outcome,
            catch_phrase: catch_phrase.into_boxed_str() }
        }
    }

    prop_compose! {
        fn arb_rule()(rule in arb_predetermined_rule()) -> Box<Rule> {
            let b: Box<Rule> = Box::new(rule);
            b
        }
    }

    prop_compose! {
        fn arb_opt()(verbose in any::<bool>()) -> Opt {
            // TODO - generate manifest_path properly
            Opt {
                verbose: verbose, manifest_path: PathBuf::from("./Cargo.toml")
            }
        }
    }

    fn arb_vec_of_rules() -> VecStrategy<BoxedStrategy<Box<Rule>>> {
        prop::collection::vec(arb_rule(), 0..100)
    }

    #[derive(Clone, Debug, PartialEq)]
    struct PredeterminedOutcomeRule {
        outcome: RuleOutcome,
        catch_phrase: Box<str>,
    }

    impl Rule for PredeterminedOutcomeRule {
        fn catch_phrase(&self) -> &str {
            self.catch_phrase.as_ref()
        }

        fn evaluate(&self, _: &Opt, _: &Option<Metadata>) -> RuleOutcome {
            self.outcome.clone()
        }
    }

    proptest! {
        #[test]
        fn outcome_stats_to_rule_outcome_never_panics(ref stats in arb_stats()) {
            let _rule_outcome:RuleOutcome = RuleOutcome::from(stats);
        }

        #[test]
        fn piles_of_fixed_outcome_rules_evaluable(ref opt in arb_opt(),
                                                  ref vec_of_rules in arb_vec_of_rules()) {
            let mut v:Vec<u8> = Vec::new();
            let _outcome:RuleOutcome = evaluate_rules(opt, &mut v, vec_of_rules, &None).into();
        }
    }
}
