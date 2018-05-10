use regex::Regex;
use std::fmt;

use rule::*;
use file::{file_present, shallow_scan_project_dir_for_file_name_match};
use cargo_metadata::Metadata;

// TODO - move Regex instances to lazy_static blocks

pub struct HasContributingFile {
    regex: Regex,
}

impl fmt::Debug for HasContributingFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HasContributingFile")
    }
}

impl Default for HasContributingFile {
    fn default() -> Self {
        HasContributingFile {
            regex: Regex::new(r"^(?i)CONTRIBUTING")
                .expect("Failed to create HasContributingFile regex."),
        }
    }
}

impl Rule for HasContributingFile {
    fn catch_phrase(&self) -> &str {
        "Should have a CONTRIBUTING file in the project root directory."
    }

    fn evaluate(&self, opt: &Opt, _: &Option<Metadata>) -> RuleOutcome {
        shallow_scan_project_dir_for_file_name_match(&self.regex, &opt.manifest_path)
    }
}

pub struct HasLicenseFile {
    regex: Regex,
}

impl fmt::Debug for HasLicenseFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HasLicenseFile")
    }
}

impl Default for HasLicenseFile {
    fn default() -> Self {
        HasLicenseFile {
            regex: Regex::new(r"^(?i)LICENSE").expect("Failed to create HasLicenseFile regex."),
        }
    }
}

impl Rule for HasLicenseFile {
    fn catch_phrase(&self) -> &'static str {
        "Should have a LICENSE file in the project root directory."
    }

    fn evaluate(&self, opt: &Opt, _: &Option<Metadata>) -> RuleOutcome {
        shallow_scan_project_dir_for_file_name_match(&self.regex, &opt.manifest_path)
    }
}

#[derive(Default, Debug)]
pub struct HasReadmeFile;

impl Rule for HasReadmeFile {
    fn catch_phrase(&self) -> &'static str {
        "Should have a README.md file in the project root directory."
    }

    fn evaluate(&self, opt: &Opt, _: &Option<Metadata>) -> RuleOutcome {
        let mut path = opt.manifest_path.clone();
        path.pop();
        file_present(&path.join("README.md")).into()
    }
}
