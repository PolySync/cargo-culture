extern crate cargo_culture_kit;

#[macro_use]
extern crate structopt;

#[cfg(test)]
#[macro_use]
extern crate proptest;

use cargo_culture_kit::{check_culture, ExitCode};
use std::io::stdout;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::env;
    use std::ffi::OsString;

    fn arb_basedir() -> BoxedStrategy<PathBuf> {
        let here = env::current_dir().unwrap_or(PathBuf::from("/test/directory"));
        let temp = env::temp_dir();
        let home = env::home_dir().unwrap_or(PathBuf::from("/home/user"));
        prop_oneof![
            Just(here),
            Just(temp),
            Just(home),
            Just(PathBuf::from(".")),
            Just(PathBuf::from("..")),
            Just(PathBuf::from("../../..")),
            Just(PathBuf::new()),
        ].boxed()
    }

    prop_compose! {
        fn arb_pathbuf()(ref base in arb_basedir(), ref segment in "[a-zA-Z0-9]+") -> PathBuf {
            base.clone().join(segment)
        }
    }

    proptest! {
        #[test]
        fn opt_parseable_from_arbitrary_inputs(
                                               ref path in arb_pathbuf(),
                                               ref verbose in any::<bool>()) {
            let mut v:Vec<OsString> = vec![
                "cargo".into(),
                "culture".into(),
                "--manifest-path".into(),
                path.into()
            ];
            if *verbose {
                v.push("--verbose".into());
            }
            let result = Opt::from_iter_safe(v);
            match result {
                Ok(o) => {
                    assert_eq!(Opt::Culture { manifest_path: path.clone(), verbose: *verbose}, o)
                },
                Err(e) => panic!("{}", e),
            }
        }
    }
}
