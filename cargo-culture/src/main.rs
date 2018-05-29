extern crate cargo_culture_kit;

#[macro_use]
extern crate structopt;

use cargo_culture_kit::{check_culture, ExitCode};
use std::io::stdout;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(bin_name = "cargo")]
pub enum Opt {
    #[structopt(name = "culture")]
    Culture {
        /// The location of the Cargo manifest for the project to check
        #[structopt(long = "manifest-path", parse(from_os_str), default_value = "./Cargo.toml")]
        manifest_path: PathBuf,

        /// If present, emit extraneous explanations and superfluous details
        #[structopt(short = "v", long = "verbose")]
        verbose: bool,
    },
}

fn main() {
    let Opt::Culture {
        manifest_path,
        verbose,
    } = Opt::from_args();
    std::process::exit(check_culture(manifest_path, verbose, &mut stdout()).exit_code())
}
