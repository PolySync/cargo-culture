use cargo_metadata::Metadata;
use std::fmt::Debug;
use std::io::Write;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub enum RuleOutcome {
    Success,
    Failure,
    Undetermined,
}

pub trait Rule: Debug {
    fn description(&self) -> &str;
    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        verbose: bool,
        metadata: &Option<Metadata>,
        print_output: &mut Write,
    ) -> RuleOutcome;
}
