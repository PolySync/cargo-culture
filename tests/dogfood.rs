extern crate cargo_culture_kit;

use cargo_culture_kit::*;

use std::io::stderr;
use std::path::PathBuf;

#[test]
fn cargo_culture_project_should_pass_its_own_scrutiny() {
    match ::std::env::var("CARGO_CULTURE_TEST_RECURSION_BUSTER") {
        Ok(_) => println!("Don't recurse infinitely."),
        Err(_) => {
            println!("About to dogfood self with a check_culture");
            let outcome = check_culture(PathBuf::from("./Cargo.toml"), false, &mut stderr());

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
