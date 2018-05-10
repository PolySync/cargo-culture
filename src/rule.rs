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

impl RuleOutcome {
    pub fn to_exit_code(&self) -> i32 {
        match *self {
            RuleOutcome::Success => 0,
            RuleOutcome::Failure => 1,
            RuleOutcome::Undetermined => 2,
        }
    }
}

pub trait Rule: Debug {
    fn catch_phrase(&self) -> &str;
    fn evaluate(&self, opt: &Opt, metadata: &Option<Metadata>) -> RuleOutcome;
}
