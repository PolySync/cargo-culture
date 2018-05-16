extern crate cargo_culture;
extern crate structopt;

use cargo_culture::{check_culture, ExitCode, Opt};
use std::io::stdout;
use structopt::StructOpt;

fn main() {
    std::process::exit(check_culture(&Opt::from_args(), stdout()).exit_code())
}
