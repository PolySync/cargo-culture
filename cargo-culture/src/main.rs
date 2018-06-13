extern crate cargo_culture_kit;

#[macro_use]
extern crate structopt;

#[cfg(test)]
#[macro_use]
extern crate proptest;

#[cfg(test)]
extern crate tempfile;

use cargo_culture_kit::{check_culture, check_culture_default, default_rules,
                        filter_to_requested_rules_from_checklist_file, find_extant_culture_file,
                        CheckError, ExitCode, OutcomesByDescription, Rule};
use std::io::stdout;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug, PartialEq)]
#[structopt(bin_name = "cargo")]
pub enum Opt {
    #[structopt(name = "culture")]
    Culture {
        /// The location of the Cargo manifest for the project to check
        #[structopt(long = "manifest-path", parse(from_os_str), default_value = "./Cargo.toml")]
        manifest_path: PathBuf,

        /// The file location of the line-separated list of Rule descriptions
        /// to check for this project
        #[structopt(long = "culture-checklist-path", parse(from_os_str))]
        culture_checklist_file_path: Option<PathBuf>,

        /// If present, emit extraneous explanations and superfluous details
        #[structopt(short = "v", long = "verbose")]
        verbose: bool,
    },
}

fn main() {
    std::process::exit(
        check_culture_cli()
            .map_err(|e| {
                println!("{}", e);
                e
            })
            .exit_code(),
    )
}

fn check_culture_cli() -> Result<OutcomesByDescription, CheckError> {
    let Opt::Culture {
        manifest_path,
        culture_checklist_file_path,
        verbose,
    } = Opt::from_args();
    match culture_checklist_file_path {
        Some(ref f) if f.is_file() => check_culture_from_checklist(&manifest_path, verbose, f),
        Some(f) => Err(CheckError::UnderspecifiedRules(format!(
            "Could not find requested rules checklist file {:?}",
            f
        ))),
        None => match find_extant_culture_file(&PathBuf::from("./.culture")) {
            None => check_culture_default(manifest_path, verbose, &mut stdout()),
            Some(ref f) => check_culture_from_checklist(&manifest_path, verbose, f),
        },
    }
}

fn check_culture_from_checklist(
    manifest_path: &Path,
    verbose: bool,
    extant_rule_checklist_file: &Path,
) -> Result<OutcomesByDescription, CheckError> {
    assert!(extant_rule_checklist_file.is_file());
    let rules = default_rules();
    let rules_refs = rules.iter().map(|r| r.as_ref()).collect::<Vec<&Rule>>();
    let filtered_rules =
        filter_to_requested_rules_from_checklist_file(extant_rule_checklist_file, &rules_refs)?;
    check_culture(manifest_path, verbose, &mut stdout(), &filtered_rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::env;
    use std::ffi::OsString;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn check_culture_from_extant_checklist() {
        let dir = tempdir().expect("Failed to make a temp dir");
        write_package_cargo_toml(dir.path());
        write_clean_src_main_file(dir.path());
        let checklist_path = dir.path().join(".culture");
        let mut checklist_file = File::create(&checklist_path).expect("Could not make target file");
        let selected_rule = cargo_culture_kit::CargoMetadataReadable::default();
        let lone_rule_description = selected_rule.description();
        checklist_file
            .write_all(format!("{}", lone_rule_description).as_bytes())
            .expect("Could not write to checklist file");
        let outcomes =
            check_culture_from_checklist(&dir.path().join("Cargo.toml"), false, &checklist_path)
                .expect("Should pass scrutiny");
        assert_eq!(1, outcomes.len());
        assert_eq!(
            Some(&cargo_culture_kit::RuleOutcome::Success),
            outcomes.get(lone_rule_description)
        );
    }

    fn write_package_cargo_toml(project_dir: &Path) {
        let cargo_path = project_dir.join("Cargo.toml");
        let mut cargo_file = File::create(cargo_path).expect("Could not make target file");
        cargo_file
            .write_all(
                br##"[package]
name = "kid"
version = "0.1.0"
authors = []

[dependencies]

[dev-dependencies]
        "##,
            )
            .expect("Could not write to Cargo.toml file");
    }

    fn write_clean_src_main_file(project_dir: &Path) {
        let src_dir = project_dir.join("src");
        create_dir_all(&src_dir).expect("Could not create src dir");
        let file_path = src_dir.join("main.rs");
        let mut file = File::create(file_path).expect("Could not make target file");
        file.write_all(
            br##"//! Sample rust file for testing cargo-culture
fn hello() { println!("Hello"); }

fn main() { hello(); }

#[cfg(test)]
mod tests {
    use super::hello;
    #[test]
    fn hello_does_not_panic() {
        assert_eq!((), hello());
    }
}
        "##,
        ).expect("Could not write to target file");
    }

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
                    assert_eq!(
                        Opt::Culture {
                            manifest_path: path.clone(),
                            culture_checklist_file_path: None,
                            verbose: *verbose},
                        o)
                },
                Err(e) => panic!("{}", e),
            }
        }
    }
}
