extern crate cargo_culture;
extern crate structopt;

use cargo_culture::{check_culture, ExitCode, Opt};
use std::io::stdout;
use structopt::StructOpt;

fn main() {
    let Opt {
        manifest_path,
        verbose,
    } = Opt::from_args();
    std::process::exit(check_culture(manifest_path, verbose, &mut stdout()).exit_code())
}
