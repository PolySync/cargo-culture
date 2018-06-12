use super::{Rule, RuleOutcome};
use cargo_metadata::Metadata;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[derive(Default, Debug)]
pub struct PassesMultipleTests;

impl Rule for PassesMultipleTests {
    fn description(&self) -> &'static str {
        "Project should have multiple tests which pass."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        _verbose: bool,
        _: &Option<Metadata>,
        _: &mut Write,
    ) -> RuleOutcome {
        let cargo = get_cargo_command();
        let mut test_cmd = Command::new(&cargo);
        test_cmd.arg("test");
        test_cmd
            .arg("--manifest-path")
            .arg(cargo_manifest_file_path);
        test_cmd.arg("--message-format").arg("json");
        test_cmd.env("CARGO_CULTURE_TEST_RECURSION_BUSTER", "true");
        match test_cmd.output() {
            Ok(_) => {
                // TODO - parse to confirm that the number of tests exceeds 1
                RuleOutcome::Success
            }
            Err(_) => RuleOutcome::Failure,
        }
    }
}

fn get_cargo_command() -> String {
    ::std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"))
}
