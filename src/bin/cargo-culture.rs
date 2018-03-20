extern crate cargo_culture;
extern crate structopt;

use structopt::StructOpt;
use cargo_culture::*;
use std::io::stdout;

fn main() {
    let code = match check_culture(&Opt::from_args(), stdout()) {
        RuleOutcome::Success => {
            0
        }
        RuleOutcome::Failure => {
            1
        }
        RuleOutcome::Undetermined => {
            2
        }
    };
    std::process::exit(code)
}
