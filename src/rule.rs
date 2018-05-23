use cargo_metadata::Metadata;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(name = "cargo-culture")]
pub struct Opt {
    #[structopt(long = "manifest-path", parse(from_os_str), default_value = "./Cargo.toml")]
    pub manifest_path: PathBuf,

    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuleOutcome {
    Success,
    Failure,
    Undetermined,
}

pub trait Rule: Debug {
    fn catch_phrase(&self) -> &str;
    fn evaluate(&self, opt: &Opt, metadata: &Option<Metadata>) -> RuleOutcome;
}
