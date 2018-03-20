use cargo_metadata::Metadata;
use std::path::PathBuf;

arg_enum! {
    #[derive(Debug)]
    pub enum Color {
        Never,
        Always,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "cargo-culture")]
pub struct Opt {
    #[structopt(long = "manifest-path", parse(from_os_str), default_value = "./Cargo.toml")]
    pub manifest_path: PathBuf,

    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,

    #[structopt(raw(possible_values = "&Color::variants()", case_insensitive = "true"), default_value = "Always")]
    pub color: Color,
}


#[derive(Debug, PartialEq)]
pub enum RuleOutcome {
    Success,
    Failure,
    Undetermined,
}


pub trait Rule {
    fn catch_phrase(&self) -> &'static str;
    fn evaluate(&self, opt: &Opt, metadata: &Option<Metadata>) -> RuleOutcome;


}
