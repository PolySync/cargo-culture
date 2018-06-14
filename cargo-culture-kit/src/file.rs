//! File discovery and inspection utilities for use in implementing `Rule`s
use super::RuleOutcome;
use cargo_metadata::Metadata;
use regex::Regex;
use std::convert::From;
use std::path::{Path, PathBuf};

pub fn shallow_scan_project_dir_for_nonempty_file_name_match(
    regex: &Regex,
    manifest_file_path: &Path,
) -> RuleOutcome {
    use std::fs::read_dir;
    let project_dir = {
        let mut p = manifest_file_path.to_path_buf();
        p.pop();
        p
    };
    if !project_dir.is_dir() {
        return RuleOutcome::Undetermined;
    }
    let mut entry_unreadable = false;
    let dir = match read_dir(project_dir) {
        Ok(d) => d,
        Err(_) => {
            return RuleOutcome::Undetermined;
        }
    };

    for entry in dir {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                let name_matches = path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| regex.is_match(name))
                    .unwrap_or(false);
                if name_matches && path.metadata().ok().map(|m| m.len() > 0).unwrap_or(false) {
                    return RuleOutcome::Success;
                }
            }
            Err(_) => {
                entry_unreadable = true;
            }
        }
    }
    if entry_unreadable {
        RuleOutcome::Undetermined
    } else {
        RuleOutcome::Failure
    }
}

pub fn search_manifest_and_workspace_dir_for_nonempty_file_name_match(
    regex: &Regex,
    manifest_path: &Path,
    maybe_metadata: &Option<Metadata>,
) -> RuleOutcome {
    let outcome_in_given_manifest_path =
        shallow_scan_project_dir_for_nonempty_file_name_match(regex, manifest_path);
    if let RuleOutcome::Success = outcome_in_given_manifest_path {
        return RuleOutcome::Success;
    }
    // If the given manifest path didn't contain the desired file name,
    // and Some(Metadata) is available, try looking in the given Metadata's
    // workspace
    match maybe_metadata {
        Some(ref metadata) => {
            match search_metadata_workspace_root_for_file_name_match(regex, metadata) {
                RuleOutcome::Success => RuleOutcome::Success,
                RuleOutcome::Failure | RuleOutcome::Undetermined => outcome_in_given_manifest_path,
            }
        }
        _ => outcome_in_given_manifest_path,
    }
}

fn search_metadata_workspace_root_for_file_name_match(
    regex: &Regex,
    metadata: &Metadata,
) -> RuleOutcome {
    if metadata.workspace_root.is_empty() {
        return RuleOutcome::Undetermined;
    }
    let workspace_manifest_path = PathBuf::from(&metadata.workspace_root).join("Cargo.toml");
    if !workspace_manifest_path.is_file() {
        return RuleOutcome::Undetermined;
    }
    shallow_scan_project_dir_for_nonempty_file_name_match(regex, &workspace_manifest_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    // TODO - some more direct tests of the search functions,
    // currently mostly tested indirectly through the

    #[test]
    fn file_present_follows_file_lifecycle() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("foo.txt");
        let r = Regex::new(r"^fo").unwrap();
        let manifest_path = &dir.path().join("Cargo.toml");
        assert_eq!(
            RuleOutcome::Failure,
            shallow_scan_project_dir_for_nonempty_file_name_match(&r, manifest_path)
        );

        let mut f = File::create(&file_path).unwrap();
        f.sync_all().unwrap();

        assert_eq!(
            RuleOutcome::Failure,
            shallow_scan_project_dir_for_nonempty_file_name_match(&r, manifest_path)
        );

        f.write_all(b"Hello, world!").unwrap();
        f.sync_all().unwrap();
        assert_eq!(
            RuleOutcome::Success,
            shallow_scan_project_dir_for_nonempty_file_name_match(&r, manifest_path)
        );

        let _ = dir.close();
    }

    // TODO - force an IO error to observe the Undetermined variant
    // TODO - explicitly handle directories vs non-directories
}
