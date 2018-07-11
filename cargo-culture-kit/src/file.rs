//! File discovery and inspection utilities for use in implementing `Rule`s
use super::RuleOutcome;
use cargo_metadata::Metadata as CargoMetadata;
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
    maybe_metadata: &Option<CargoMetadata>,
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
    metadata: &CargoMetadata,
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
    use super::super::rules::test_support::*;
    use super::*;
    use cargo_metadata::metadata;
    use proptest::prelude::*;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use tempfile::tempdir;

    proptest! {
        #[test]
        fn shallow_scan_finds_arb_file(ref file_name in "[a-zA-Z0-9]+") {
            let dir = tempdir().expect("Failed to make a temp dir");
            let mut s = String::from("^");
            s.push_str(file_name);
            let r = Regex::new(&s).expect("Could not make trivial prefix regex");
            // Ignore false positives regarding the Cargo.toml file
            prop_assume!(!r.is_match("Cargo.toml"));
            let manifest_path = &dir.path().join("Cargo.toml");
            prop_assert_eq!(
                RuleOutcome::Failure,
                shallow_scan_project_dir_for_nonempty_file_name_match(&r, manifest_path)
            );
            let file_path = dir.path().join(file_name);
            let mut f = File::create(&file_path).expect("Could not create temp file");
            f.sync_all()
                .expect("Could not sync temp file state initially");
            prop_assert_eq!(
                RuleOutcome::Failure,
                shallow_scan_project_dir_for_nonempty_file_name_match(&r, manifest_path)
            );
            f.write_all(b"Hello, world!")
                .expect("Could not write to temp file");
            f.sync_all()
                .expect("Could not sync temp file state after write");
            prop_assert_eq!(
                RuleOutcome::Success,
                shallow_scan_project_dir_for_nonempty_file_name_match(&r, manifest_path)
            );
        }
        #[test]
        fn search_manifest_and_workspace_dir_for_nonempty_file_name_match_file_lifecycle(
                ref file_name in "[a-zA-Z0-9]+",
                ref in_kid in any::<bool>()) {

            let base_dir = tempdir().expect("Failed to make a temp dir");
            let workspace_manifest_path = base_dir.path().join("Cargo.toml");
            create_workspace_cargo_toml(&workspace_manifest_path);
            let subproject_dir = base_dir.path().join("kid");
            let child_manifest_path = subproject_dir.join("Cargo.toml");
            create_dir_all(&subproject_dir).expect("Could not create subproject dir");
            write_package_cargo_toml(&subproject_dir, None);
            write_clean_src_main_file(&subproject_dir);
            let mut s = String::from("^");
            s.push_str(file_name);
            let r = Regex::new(&s).expect("Could not make trivial prefix regex");
            // Ignore false positives regarding the Cargo.toml file
            prop_assume!(!r.is_match("Cargo.toml"));
            let metadata = Some(metadata(Some(&child_manifest_path)).expect("Could not get test cargo manifest"));

            prop_assert_eq!(
                RuleOutcome::Failure,
                search_manifest_and_workspace_dir_for_nonempty_file_name_match(&r, &workspace_manifest_path, &metadata)
            );
            prop_assert_eq!(
                RuleOutcome::Failure,
                search_manifest_and_workspace_dir_for_nonempty_file_name_match(&r, &child_manifest_path, &metadata)
            );

            let target_file_path = if *in_kid {
                subproject_dir.join(file_name)
            } else {
                base_dir.path().join(file_name)
            };
            let mut target_file =
                File::create(&target_file_path).expect("Could not make target file");
            target_file
                .write_all(b"Hello, I am a target file.")
                .expect("Could not write to target file");
            target_file.sync_all()
                .expect("Could not sync temp file state initially");

            prop_assert_eq!(
                RuleOutcome::Success,
                search_manifest_and_workspace_dir_for_nonempty_file_name_match(&r, &child_manifest_path, &metadata)
            );

            prop_assert_eq!(
                if *in_kid { RuleOutcome::Failure } else { RuleOutcome::Success },
                search_manifest_and_workspace_dir_for_nonempty_file_name_match(&r, &workspace_manifest_path, &metadata)
            );
        }
    }

    #[test]
    fn shallow_scan_follows_file_lifecycle() {
        let dir = tempdir().expect("Failed to make a temp dir");
        let file_path = dir.path().join("foo.txt");
        let r = Regex::new(r"^fo").expect("Could not make trivial prefix regex");
        let manifest_path = &dir.path().join("Cargo.toml");
        assert_eq!(
            RuleOutcome::Failure,
            shallow_scan_project_dir_for_nonempty_file_name_match(&r, manifest_path)
        );

        let mut f = File::create(&file_path).expect("Could not create temp file");
        f.sync_all()
            .expect("Could not sync temp file state initially");

        assert_eq!(
            RuleOutcome::Failure,
            shallow_scan_project_dir_for_nonempty_file_name_match(&r, manifest_path)
        );

        f.write_all(b"Hello, world!")
            .expect("Could not write to temp file");
        f.sync_all()
            .expect("Could not sync temp file state after write");
        assert_eq!(
            RuleOutcome::Success,
            shallow_scan_project_dir_for_nonempty_file_name_match(&r, manifest_path)
        );

        let _ = dir.close();
    }
}
