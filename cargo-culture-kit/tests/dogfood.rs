extern crate cargo_culture_kit;

use cargo_culture_kit::*;

use std::io::stderr;
use std::path::{Path, PathBuf};

#[test]
fn cargo_culture_kit_project_should_pass_its_own_scrutiny() {
    let path = PathBuf::from("./Cargo.toml")
        .canonicalize()
        .expect("Could not canonicalize path");
    assert_checks_default_culture(&path);
}

#[test]
fn cargo_culture_workspace_should_pass_its_own_scrutiny() {
    let path = PathBuf::from("../Cargo.toml")
        .canonicalize()
        .expect("Could not canonicalize path");
    assert_checks_default_culture(&path);
}

#[test]
fn cargo_culture_binary_should_pass_its_own_scrutiny() {
    let path = PathBuf::from("../cargo-culture/Cargo.toml")
        .canonicalize()
        .expect("Could not canonicalize path");
    assert_checks_default_culture(&path);
}

fn assert_checks_default_culture(cargo_manifest_file_path: &Path) {
    match ::std::env::var("CARGO_CULTURE_TEST_RECURSION_BUSTER") {
        Ok(_) => println!("Don't recurse infinitely."),
        Err(_) => {
            println!(
                "About to dogfood self with a check_culture, using the manifest at: {:?}",
                cargo_manifest_file_path
            );
            let outcome = check_culture_default(cargo_manifest_file_path, false, &mut stderr())
                .expect("Should have no errors running the checks")
                .into();

            assert_eq!(
                OutcomeStats {
                    success_count: 9,
                    fail_count: 0,
                    unknown_count: 0,
                },
                outcome
            );
        }
    }
}
