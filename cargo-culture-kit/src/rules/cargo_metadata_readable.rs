use cargo_metadata::Metadata;
use std::io::Write;
use std::path::Path;
use super::{Rule, RuleOutcome};

#[derive(Default, Debug)]
pub struct CargoMetadataReadable;

impl Rule for CargoMetadataReadable {
    fn description(&self) -> &'static str {
        "Should have a well-formed Cargo.toml file readable by `cargo metadata`"
    }

    fn evaluate(
        &self,
        _: &Path,
        _: bool,
        metadata: &Option<Metadata>,
        _: &mut Write,
    ) -> RuleOutcome {
        match *metadata {
            None => RuleOutcome::Failure,
            Some(_) => RuleOutcome::Success,
        }
    }
}
