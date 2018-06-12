//! Helper functions related to the interpretation and filtering of `Rule` description checklists
use super::CheckError;
use super::Rule;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub const DEFAULT_CULTURE_CHECKLIST_FILE_NAME: &str = ".culture";

pub fn find_extant_culture_file(initial_culture_file: &Path) -> Option<PathBuf> {
    if initial_culture_file.is_file() {
        return Some(PathBuf::from(initial_culture_file));
    }
    let mut p: Option<&Path> = initial_culture_file.parent();
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

// TODO - probably ought to switch available_rules to be an IntoIterator of
// some kind to reduce pointless map-to-as_ref-and-collects
pub fn filter_to_requested_rules_from_checklist_file<'path, 'rules>(
    culture_checklist_file_path: &'path Path,
    available_rules: &'rules [&Rule],
) -> Result<Vec<&'rules Rule>, CheckError> {
    let f = match File::open(culture_checklist_file_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(CheckError::UnderspecifiedRules(format!(
                "Difficulty opening culture checklist file. {:?}",
                e
            )))
        }
    };
    let content = BufReader::new(&f);
    let mut descriptions: Vec<String> = Vec::new();
    for line in content.lines() {
        match line {
            Ok(ref l) if !l.is_empty() => descriptions.push(l.to_string()),
            Ok(_) => (),
            Err(e) => {
                return Err(CheckError::UnderspecifiedRules(format!(
                    "Difficulty reading culture checklist file. {:?}",
                    e
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

pub fn filter_to_requested_rules_by_description<'r, 'd>(
    available_rules: &'r [&Rule],
    desired_rule_descriptions: &'d [&str],
) -> Result<Vec<&'r Rule>, CheckError> {
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
                return Err(CheckError::RequestedRuleNotFound {
                    rule_description: description.to_string(),
                })
            }
        };
    }
    Ok(rules)
}
