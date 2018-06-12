use cargo_metadata::Metadata;
use regex::Regex;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;
use super::{Rule, RuleOutcome};

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
    fn description(&self) -> &'static str {
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
                if let Ok(s) = String::from_utf8(build_output.stdout) {
                    let _ = writeln!(print_output, "`{}` StdOut: {}", command_str, s);
                }
                if let Ok(s) = String::from_utf8(build_output.stderr) {
                    let _ = writeln!(print_output, "`{}` StdErr: {}", command_str, s);
                }
            }
            return RuleOutcome::Failure;
        }
        let stdout = match from_utf8(&build_output.stdout) {
            Ok(stdout) => stdout,
            Err(e) => {
                if verbose {
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
