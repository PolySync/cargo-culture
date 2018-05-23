use regex::Regex;

use cargo_metadata::{DependencyKind, Metadata};
use file::{file_present, FilePresence};
use rule::*;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

#[derive(Default, Debug)]
pub struct CargoMetadataReadable;

impl Rule for CargoMetadataReadable {
    fn catch_phrase(&self) -> &'static str {
        "Should have a well-formed Cargo.toml file readable by `cargo metadata`"
    }

    fn evaluate(
        &self,
        _: &Path,
        _: bool,
        metadata: &Option<Metadata>,
        _: &mut Write,
    ) -> RuleOutcome {
        match *metadata {
            None => RuleOutcome::Failure,
            Some(_) => RuleOutcome::Success,
        }
    }
}

#[derive(Default, Debug)]
pub struct HasContinuousIntegrationFile;

impl Rule for HasContinuousIntegrationFile {
    fn catch_phrase(&self) -> &'static str {
        "Should have a file suggesting the use of a continuous integration system."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        _verbose: bool,
        _: &Option<Metadata>,
        _: &mut Write,
    ) -> RuleOutcome {
        let project_dir = {
            let mut project_dir = cargo_manifest_file_path.to_path_buf();
            project_dir.pop();
            project_dir
        };
        if !project_dir.is_dir() {
            return RuleOutcome::Undetermined;
        }
        let files = vec![
            "appveyor.yml",
            ".appveyor.yml",
            ".drone.yml",
            ".gitlab-ci.yml",
            ".travis.yml",
        ];
        for file in files {
            let ci_file_presence = file_present(&project_dir.clone().join(file));
            if let FilePresence::Present = ci_file_presence {
                return RuleOutcome::Success;
            }
        }
        // TODO - if all are undeterminable, report that
        // TODO - consider providing more verbose feedback for empties?
        RuleOutcome::Failure
    }
}

#[derive(Debug, Default)]
pub struct UsesPropertyBasedTestLibrary;

lazy_static! {
    static ref USES_PROPERTY_BASED_TEST_LIBRARY: Regex =
        Regex::new(r"^(?i)(proptest)|(quickcheck)|(suppositions).*")
            .expect("Failed to create UsesPropertyBasedTestLibrary regex.");
}

impl Rule for UsesPropertyBasedTestLibrary {
    fn catch_phrase(&self) -> &'static str {
        "Should be making an effort to use property based tests."
    }

    fn evaluate(
        &self,
        _: &Path,
        _: bool,
        metadata: &Option<Metadata>,
        _: &mut Write,
    ) -> RuleOutcome {
        match *metadata {
            None => RuleOutcome::Undetermined,
            Some(ref m) => {
                if m.packages.is_empty() {
                    return RuleOutcome::Undetermined;
                }
                for package in &m.packages {
                    let has_pbt_dep = package
                        .dependencies
                        .iter()
                        .filter(|d| d.kind == DependencyKind::Development)
                        .any(|d| USES_PROPERTY_BASED_TEST_LIBRARY.is_match(&d.name));
                    if !has_pbt_dep {
                        return RuleOutcome::Failure;
                    }
                }
                RuleOutcome::Success
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct BuildsCleanlyWithoutWarningsOrErrors;

lazy_static! {
    static ref WARNING_JSON: Regex = Regex::new(".*\"level\":\"warning\".*")
        .expect("Failed to create BuildsCleanlyWithoutWarningsOrErrors regex.");
}

fn clean_packages(
    cargo_command: &str,
    cargo_manifest_file_path: &Path,
    verbose: bool,
    metadata: &Option<Metadata>,
    print_output: &mut Write,
) -> bool {
    match *metadata {
        None => {
            if verbose {
                let _ = writeln!(
                    print_output,
                    "No metadata to discover which packages to clean."
                );
            }
            false
        }
        Some(ref m) if m.packages.is_empty() => {
            if verbose {
                let _ = writeln!(print_output, "No packages to clean.");
            }
            false
        }
        Some(ref m) => {
            let mut all_cleaned = true;
            for p in &m.packages {
                let cleaned = clean_package(
                    cargo_command,
                    &p.name,
                    cargo_manifest_file_path,
                    verbose,
                    print_output,
                );
                if !cleaned && verbose {
                    let _ = writeln!(print_output, "Could not clean package {} .", &p.name);
                }
                all_cleaned = all_cleaned && cleaned;
            }
            all_cleaned
        }
    }
}

fn clean_package(
    cargo_command: &str,
    package_name: &str,
    cargo_manifest_file_path: &Path,
    verbose: bool,
    print_output: &mut Write,
) -> bool {
    let mut clean_cmd = Command::new(&cargo_command);
    clean_cmd.arg("clean");
    clean_cmd
        .arg("--manifest-path")
        .arg(cargo_manifest_file_path);
    clean_cmd.arg("--package").arg(package_name);
    let clean_output = match clean_cmd.output() {
        Ok(o) => o,
        Err(e) => {
            if verbose {
                let _ = writeln!(print_output, "{}", e);
            }
            return false;
        }
    };
    clean_output.status.success()
}

fn get_cargo_command() -> String {
    ::std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"))
}

impl Rule for BuildsCleanlyWithoutWarningsOrErrors {
    fn catch_phrase(&self) -> &'static str {
        "Should `cargo clean` and `cargo build` without any warnings or errors."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        verbose: bool,
        metadata: &Option<Metadata>,
        print_output: &mut Write,
    ) -> RuleOutcome {
        let cargo = get_cargo_command();
        let packages_cleaned = clean_packages(
            &cargo,
            cargo_manifest_file_path,
            verbose,
            metadata,
            print_output,
        );
        if !packages_cleaned {
            return RuleOutcome::Failure;
        }

        let mut build_cmd = Command::new(&cargo);
        build_cmd.arg("build");
        build_cmd
            .arg("--manifest-path")
            .arg(cargo_manifest_file_path);
        build_cmd.arg("--message-format=json");
        let command_str = format!("{:?}", build_cmd);
        let build_output = match build_cmd.output() {
            Ok(o) => o,
            Err(_e) => {
                return RuleOutcome::Undetermined;
            }
        };
        if !build_output.status.success() {
            if verbose {
                // TODO - Resolve desired output stream for verbose content
                let _ = writeln!(print_output, "Build command `{}` failed", command_str);
                let _ = writeln!(
                    print_output,
                    "`{}` StdOut: {}",
                    command_str,
                    String::from_utf8(build_output.stdout)
                        .expect("Could not interpret `cargo build` stdout")
                );
                let _ = writeln!(
                    print_output,
                    "`{}` StdErr: {}",
                    command_str,
                    String::from_utf8(build_output.stderr)
                        .expect("Could not interpret `cargo build` stderr")
                );
            }
            return RuleOutcome::Failure;
        }
        let stdout = match from_utf8(&build_output.stdout) {
            Ok(stdout) => stdout,
            Err(e) => {
                if verbose {
                    // TODO - Resolve desired output stream for verbose content
                    let _ = writeln!(
                        print_output,
                        "Reading stdout for command `{}` failed : {}",
                        command_str, e
                    );
                }
                return RuleOutcome::Undetermined;
            }
        };

        if WARNING_JSON.is_match(stdout) {
            return RuleOutcome::Failure;
        }
        RuleOutcome::Success
    }
}

#[derive(Default, Debug)]
pub struct PassesMultipleTests;

impl Rule for PassesMultipleTests {
    fn catch_phrase(&self) -> &'static str {
        "Project should have multiple tests which pass."
    }

    fn evaluate(
        &self,
        cargo_manifest_file_path: &Path,
        _verbose: bool,
        _: &Option<Metadata>,
        _: &mut Write,
    ) -> RuleOutcome {
        let cargo = get_cargo_command();
        let mut test_cmd = Command::new(&cargo);
        test_cmd.arg("test");
        test_cmd
            .arg("--manifest-path")
            .arg(cargo_manifest_file_path);
        test_cmd.arg("--message-format").arg("json");
        test_cmd.env("CARGO_CULTURE_TEST_RECURSION_BUSTER", "true");
        match test_cmd.output() {
            Ok(_) => {
                // TODO - parse to confirm that the number of tests exceeds 1
                RuleOutcome::Success
            }
            Err(_) => RuleOutcome::Failure,
        }
    }
}
