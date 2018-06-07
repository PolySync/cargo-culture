use regex::Regex;

use cargo_metadata::Metadata;
use file::{is_file_present, search_manifest_and_workspace_dir_for_file_name_match};
use rule::*;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Default)]
pub struct HasContributingFile;

lazy_static! {
    static ref HAS_CONTRIBUTING_FILE: Regex =
        Regex::new(r"^(?i)CONTRIBUTING").expect("Failed to create HasContributingFile regex.");
}

impl Rule for HasContributingFile {
    fn description(&self) -> &str {
        "Should have a CONTRIBUTING file in the project directory."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        _verbose: bool,
        metadata: &Option<Metadata>,
        _print_output: &mut Write,
    ) -> RuleOutcome {
        search_manifest_and_workspace_dir_for_file_name_match(
            &HAS_CONTRIBUTING_FILE,
            cargo_manifest_file_path,
            metadata,
        )
    }
}

#[derive(Debug, Default)]
pub struct HasLicenseFile;

lazy_static! {
    static ref HAS_LICENSE_FILE: Regex =
        Regex::new(r"^(?i)LICENSE").expect("Failed to create HasLicenseFile regex.");
}

impl Rule for HasLicenseFile {
    fn description(&self) -> &'static str {
        "Should have a LICENSE file in the project directory."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        _verbose: bool,
        metadata: &Option<Metadata>,
        _print_output: &mut Write,
    ) -> RuleOutcome {
        search_manifest_and_workspace_dir_for_file_name_match(
            &HAS_LICENSE_FILE,
            cargo_manifest_file_path,
            metadata,
        )
    }
}

#[derive(Debug, Default)]
pub struct HasReadmeFile;

impl Rule for HasReadmeFile {
    fn description(&self) -> &'static str {
        "Should have a README.md file in the project directory."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        _verbose: bool,
        _metadata: &Option<Metadata>,
        _print_output: &mut Write,
    ) -> RuleOutcome {
        let mut path = cargo_manifest_file_path.to_path_buf();
        path.pop();
        is_file_present(&path.join("README.md")).into()
    }
}

#[derive(Debug, Default)]
pub struct HasRustfmtFile;

lazy_static! {
    static ref HAS_RUSTFMT_FILE: Regex =
        Regex::new(r"^\.?(legacy-)?rustfmt.toml$").expect("Failed to create HasRustfmtFile regex.");
}

impl Rule for HasRustfmtFile {
    fn description(&self) -> &'static str {
        "Should have a rustfmt.toml file in the project directory."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        _verbose: bool,
        metadata: &Option<Metadata>,
        _print_output: &mut Write,
    ) -> RuleOutcome {
        search_manifest_and_workspace_dir_for_file_name_match(
            &HAS_RUSTFMT_FILE,
            cargo_manifest_file_path,
            metadata,
        )
    }
}
