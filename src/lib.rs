#[cfg(test)]
extern crate tempdir;

#[cfg(test)]
#[macro_use]
extern crate proptest;

#[macro_use]
extern crate structopt;

#[macro_use]
extern crate clap;

extern crate cargo_metadata;
extern crate colored;

extern crate regex;

mod file;
mod rule;
mod collaboration;
mod build_infra;


use build_infra::*;
use collaboration::*;
pub use rule::*;

use colored::*;
use std::path::PathBuf;
use std::io::Write;


pub fn check_culture<W: Write>(opt: &Opt, mut output: W) -> RuleOutcome {
    let rules: Vec<Box<Rule>> = vec![
        Box::new(HasReadmeFile::default()),
        Box::new(HasContributingFile::default()),
        Box::new(HasLicenseFile::default()),
        Box::new(CargoMetadataReadable::default()),
        Box::new(BuildsCleanlyWithoutWarningsOrErrors::default()),
        Box::new(HasContinuousIntegrationFile::default()),
        Box::new(UsesPropertyBasedTestLibrary::default()),
        Box::new(PassesMultipleTests::default()),
    ];

    // TODO - will need to do some more forgiving custom metadata parsing to deal with changes
    // in cargo metadata format -- the current crate assumes you're on a recent nightly, where workspace_root has been added
    let manifest_path: PathBuf = opt.manifest_path.clone();
    let metadata_result = cargo_metadata::metadata(Some(manifest_path.as_ref()));
    let metadata_option = match metadata_result {
        Ok(m) => Some(m),
        Err(e) => {
            if opt.verbose {
                writeln!(output, "{}", e).expect("Error reporting project's `cargo metadata`");
            }
            None
        }
    };

    let mut success_count = 0;
    let mut fail_count = 0;
    let mut unknown_count = 0;
    for rule in rules.into_iter() {
        let outcome = rule.evaluate(&opt, &metadata_option);
        print_outcome(&*rule, &outcome, &mut output);
        match outcome {
            RuleOutcome::Success => {
                success_count += 1;
            }
            RuleOutcome::Failure => {
                fail_count += 1;
            }
            RuleOutcome::Undetermined => {
                unknown_count += 1;
            }
        }
    }
    let any_bad = fail_count > 0 || unknown_count > 0;
    let conclusion = if any_bad { "FAILED".red() } else { "ok".green() };
    writeln!(output, "culture result: {}. {} passed. {} failed. {} undetermined.",
             conclusion,
             success_count, fail_count, unknown_count)
        .expect("Error reporting culture check summary.");

    match (success_count, fail_count, unknown_count) {
        (_, 0, 0) => RuleOutcome::Success,
        (_, 0, _) => RuleOutcome::Undetermined,
        (_, f, _) if f > 0 => RuleOutcome::Failure,
        _ => unreachable!()
    }
}

fn print_outcome(rule: &Rule, outcome: &RuleOutcome, output: &mut Write) {
    writeln!(output, "{} ... {}", rule.catch_phrase(), summary_str(outcome))
        .expect("Could not write rule outcome");
}

fn summary_str(outcome: &RuleOutcome) -> colored::ColoredString {
    match *outcome {
        RuleOutcome::Success => {
            "ok".green()
        }
        RuleOutcome::Failure => {
            "FAILED".red()
        }
        RuleOutcome::Undetermined => {
            "UNDETERMINED".red()
        }
    }
}
