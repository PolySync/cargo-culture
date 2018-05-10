extern crate cargo_culture;
extern crate structopt;

use structopt::StructOpt;
use cargo_culture::{check_culture, Opt};
use std::io::stdout;

fn main() {
    std::process::exit(check_culture(&Opt::from_args(), stdout()).to_exit_code())
}
