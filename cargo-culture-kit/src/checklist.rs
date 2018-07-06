//! Helper functions related to the interpretation and filtering of `Rule`
//! description checklists.
//!
//! These checklists can be encoded as a line-delimited file of `Rule`
//! descriptions.
use super::Rule;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// The default name for a culture Rule checklist file,
/// used when searching for a checklist file
pub const DEFAULT_CULTURE_CHECKLIST_FILE_NAME: &str = ".culture";

/// Errors specific to filtering down a set of `Rule`s based on a checklist
/// of `Rule` descriptions.
#[derive(Debug, Clone, Eq, Fail, PartialEq, Hash)]
pub enum FilterError {
    #[fail(
        display = "There was an error while attempting to read the checklist of rules to check: {}",
        _0
    )]
    /// Covers failures in reading a checklist file of `Rule` descriptions that
    /// could be used to specified the set of rules to evaluate.
    RuleChecklistReadError(String),
    #[fail(
        display = "A described rule specified was not in the available set of Rule implementations: {}",
        rule_description
    )]
    /// An error while filtering the set of `Rule`s to run. The most common
    /// cause of this is when a checklist of `Rule` descriptions includes a
    /// description of a `Rule` that does not match any of the available
    /// `Rule` instances.
    RequestedRuleNotFound {
        /// The problematic description for which a matching `Rule` was not
        /// found.
        rule_description: String,
    },
    /// Destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this hidden variant
    /// ensures users do not rely on exhaustive matching.
    #[doc(hidden)]
    #[fail(display = "A hidden variant to increase expansion flexibility")]
    __Nonexhaustive,
}

/// If the supplied `initial_culture_file` path is an extant file, just return
/// that.
///
/// Otherwise, search the specified path and its ancestor directories for a file
/// with a name matching the `DEFAULT_CULTURE_CHECKLIST_FILE_NAME`
pub fn find_extant_culture_file(initial_culture_file: &Path) -> Option<PathBuf> {
    let first_dir = if initial_culture_file.is_file() {
        return Some(PathBuf::from(initial_culture_file));
    } else if initial_culture_file.is_dir() {
        Some(initial_culture_file)
    } else {
        initial_culture_file.parent()
    };
    let mut p: Option<&Path> = first_dir;
    loop {
        p = match p {
            Some(dir) => {
                let potential_culture_file = dir.join(DEFAULT_CULTURE_CHECKLIST_FILE_NAME);
                if potential_culture_file.is_file() {
                    return Some(potential_culture_file);
                } else {
                    dir.parent()
                }
            }
            None => return None,
        }
    }
}

/// Produces a filtered subset of the provided `Rule`s by
/// matching their `description`s to the lines of the
/// the file specified by `culture_checklist_file_path`.
///
/// # Errors
///
/// Returns a `FilterError::RuleChecklistReadError` error when one of the lines
/// of the file does not match any of the provided `Rule` descriptions.
// TODO - probably ought to switch available_rules to be an IntoIterator of
// some kind to reduce pointless map-to-as_ref-and-collects
pub fn filter_to_requested_rules_from_checklist_file<'path, 'rules>(
    culture_checklist_file_path: &'path Path,
    available_rules: &'rules [&Rule],
) -> Result<Vec<&'rules Rule>, FilterError> {
    let f = match File::open(culture_checklist_file_path) {
        Ok(f) => f,
        Err(_) => {
            return Err(FilterError::RuleChecklistReadError(format!(
                "Could not open the culture checklist file, {}",
                culture_checklist_file_path.display()
            )))
        }
    };
    let content = BufReader::new(&f);
    let mut descriptions: Vec<String> = Vec::new();
    for line in content.lines() {
        match line {
            Ok(ref l) if !l.is_empty() => descriptions.push(l.to_string()),
            Ok(_) => (),
            Err(_) => {
                return Err(FilterError::RuleChecklistReadError(format!(
                    "Difficulty reading lines of the culture checklist file, {}",
                    culture_checklist_file_path.display()
                )))
            }
        }
    }
    let description_refs = descriptions
        .iter()
        .map(|d| d.as_ref())
        .collect::<Vec<&str>>();
    filter_to_requested_rules_by_description(available_rules, description_refs.as_slice())
}

/// Produces a filtered subset of the provided `Rule`s by
/// matching their `description`s to the members of the
/// the `desired_rule_descriptions` slice.
///
/// # Errors
///
/// Returns a `FilterError::RuleChecklistReadError` error when one of the lines
/// of the file does not match any of the provided `Rule` descriptions.
pub fn filter_to_requested_rules_by_description<'r, 'd>(
    available_rules: &'r [&Rule],
    desired_rule_descriptions: &'d [&str],
) -> Result<Vec<&'r Rule>, FilterError> {
    let mut rules: Vec<&Rule> = Vec::with_capacity(desired_rule_descriptions.len());
    // Given the expected number of rules applied will be low (sub-hundreds), we
    // stick with simplistic and ordered slices rather than using more optimal
    // data structures
    for description in desired_rule_descriptions {
        match available_rules
            .iter()
            .find(|r| &r.description() == description)
        {
            Some(r) => rules.push(*r),
            None => {
                return Err(FilterError::RequestedRuleNotFound {
                    rule_description: description.to_string(),
                })
            }
        };
    }
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::super::{HasLicenseFile, HasReadmeFile};
    use super::*;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn find_extant_file_direct_file_success() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join(DEFAULT_CULTURE_CHECKLIST_FILE_NAME);
        let rule = HasReadmeFile::default();
        let mut file = File::create(&file_path).expect("Could not make target file");
        file.write_all(rule.description().as_bytes())
            .expect("Could not write to target file");

        let found = find_extant_culture_file(&file_path);

        assert_eq!(
            Some(PathBuf::from(
                dir.path().join(DEFAULT_CULTURE_CHECKLIST_FILE_NAME)
            )),
            found
        );
    }

    #[test]
    fn find_extant_file_direct_file_alternate_name_success() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("my_custom_checklist.txt");
        let rule = HasReadmeFile::default();
        let mut file = File::create(&file_path).expect("Could not make target file");
        file.write_all(rule.description().as_bytes())
            .expect("Could not write to target file");

        let found = find_extant_culture_file(&file_path);

        assert_eq!(
            Some(PathBuf::from(dir.path().join("my_custom_checklist.txt"))),
            found
        );
    }

    #[test]
    fn find_extant_file_from_dir_success() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join(DEFAULT_CULTURE_CHECKLIST_FILE_NAME);
        let rule = HasReadmeFile::default();
        let mut file = File::create(&file_path).expect("Could not make target file");
        file.write_all(rule.description().as_bytes())
            .expect("Could not write to target file");

        let found = find_extant_culture_file(dir.path());

        assert_eq!(
            Some(PathBuf::from(
                dir.path().join(DEFAULT_CULTURE_CHECKLIST_FILE_NAME)
            )),
            found
        );
    }

    #[test]
    fn find_extant_file_from_dir_ancestor_success() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let subdir = dir.path().join("kid").join("grandkid");
        create_dir_all(&subdir).expect("Could not make subdirs");

        let file_path = dir.path().join(DEFAULT_CULTURE_CHECKLIST_FILE_NAME);
        let rule = HasReadmeFile::default();
        let mut file = File::create(&file_path).expect("Could not make target file");
        file.write_all(rule.description().as_bytes())
            .expect("Could not write to target file");

        let found = find_extant_culture_file(&subdir);

        assert_eq!(Some(file_path), found);
    }

    #[test]
    fn find_extant_none_when_absent_file() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join(DEFAULT_CULTURE_CHECKLIST_FILE_NAME);
        let found = find_extant_culture_file(&file_path);
        assert_eq!(None, found);
    }

    #[test]
    fn find_extant_none_for_dir_when_absent_file() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let found = find_extant_culture_file(dir.path());
        assert_eq!(None, found);
    }

    #[test]
    fn filter_by_file_error_when_absent_file() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join(DEFAULT_CULTURE_CHECKLIST_FILE_NAME);
        let rule_a = HasReadmeFile::default();
        let rule_b = HasLicenseFile::default();
        if let Err(e) =
            filter_to_requested_rules_from_checklist_file(&file_path, &[&rule_a, &rule_b])
        {
            match e {
                FilterError::RuleChecklistReadError(_) => println!("As expected"),
                _ => panic!("Unexpected error kind"),
            }
        } else {
            panic!("Expected an error due to a lack of a checklist file");
        }
    }

    #[test]
    fn filter_by_file_restricts_to_specified_rules() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join(DEFAULT_CULTURE_CHECKLIST_FILE_NAME);
        let rule_a = HasReadmeFile::default();
        let rule_b = HasLicenseFile::default();
        let raw_rules: &[&Rule] = &[&rule_a, &rule_b];

        let mut file = File::create(&file_path).expect("Could not make target file");
        file.write_all(rule_a.description().as_bytes())
            .expect("Could not write to target file");

        let filtered_rules = filter_to_requested_rules_from_checklist_file(&file_path, raw_rules)
            .expect("Filtering should work when the file is present");

        // Two rules enter, one rule leaves
        assert_eq!(1, filtered_rules.len());
        assert_eq!(
            rule_a.description(),
            filtered_rules.first().unwrap().description()
        );
    }

    #[test]
    fn filter_by_file_errors_when_requested_rule_not_found() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join(DEFAULT_CULTURE_CHECKLIST_FILE_NAME);
        let rule_a = HasReadmeFile::default();
        let rule_b = HasLicenseFile::default();
        let raw_rules: &[&Rule] = &[&rule_a, &rule_b];

        let mut file = File::create(&file_path).expect("Could not make target file");
        let silly_rule = b"Every function in the project should halt given reasonable inputs.";
        file.write_all(silly_rule)
            .expect("Could not write to target file");

        if let Err(e) = filter_to_requested_rules_from_checklist_file(&file_path, raw_rules) {
            let _s = ::std::str::from_utf8(silly_rule)
                .expect("Should be able to stringify silly rule")
                .to_string();
            match e {
                FilterError::RequestedRuleNotFound {
                    rule_description: _s,
                } => println!("As expected"),
                _ => panic!("Unexpected error kind"),
            }
        } else {
            panic!("Expected an error due to a lack of a checklist file");
        }
    }
}
