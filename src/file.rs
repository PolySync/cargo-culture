use std::convert::From;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use super::RuleOutcome;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum FilePresence {
    Absent,
    Empty,
    Present,
    Unknown,
}

impl From<FilePresence> for RuleOutcome {
    fn from(file_presence: FilePresence) -> Self {
        match file_presence {
            FilePresence::Absent => RuleOutcome::Failure,
            FilePresence::Empty => RuleOutcome::Failure,
            FilePresence::Present => RuleOutcome::Success,
            FilePresence::Unknown => RuleOutcome::Undetermined,
        }
    }
}

pub fn file_present(path: &Path) -> FilePresence {
    let metadata = match path.metadata() {
        Err(ref e) if e.kind() == ErrorKind::NotFound => return FilePresence::Absent,
        Err(_) => return FilePresence::Unknown,
        Ok(metadata) => metadata,
    };
    if metadata.len() == 0 {
        return FilePresence::Empty;
    }
    FilePresence::Present
}

pub fn shallow_scan_project_dir_for_file_name_match(
    regex: &Regex,
    manifest_path: &PathBuf,
) -> RuleOutcome {
    use std::fs::read_dir;
    let project_dir = {
        let mut project_dir = manifest_path.clone();
        project_dir.pop();
        project_dir
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
                if name_matches {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn file_present_follows_file_lifecycle() {
        let dir = TempDir::new("my_directory_prefix").unwrap();
        let file_path = dir.path().join("foo.txt");
        assert_eq!(FilePresence::Absent, file_present(&file_path));

        let mut f = File::create(&file_path).unwrap();
        f.sync_all().unwrap();
        assert_eq!(FilePresence::Empty, file_present(&file_path));
        f.write_all(b"Hello, world!").unwrap();
        f.sync_all().unwrap();
        assert_eq!(FilePresence::Present, file_present(&file_path));

        let _ = dir.close();
    }

    // TODO - force an IO error to observe the Unknown variant
    // TODO - explicitly handle directories vs non-directories
}
