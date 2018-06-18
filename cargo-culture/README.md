# cargo-culture

## Overview

Cargo subcommand command that provides easy access to
the cargo-culture-kit's default rules and provides an
exemplar for the development of rule-checking binaries.

## Getting Started

### Dependencies

`cargo-culture` is a Rust project, and manages its dependencies with `cargo`,
available as part of the standard Rust toolchain.

* [rust](https://github.com/rust-lang-nursery/rustup.rs)

### Building

For local program development, you can build `cargo-culture`
with:

* Download the project repository
  ```bash
  git clone https://github.com/PolySync/cargo-culture.git
  cd cargo-culture/cargo-culture
  ```
* Execute a build
  ```bash
  cargo build
  ```

### Installation

You can install `cargo-culture` directly from [crates.io](https://crates.io).

* Use cargo's built-in binary installation command
  ```bash
  cargo install cargo-culture
  ```
* Alternately, you may install from a local clone of the
  repository with:
  ```bash
  git clone https://github.com/PolySync/cargo-culture.git
  cd cargo-culture/cargo-culture
  cargo install
  ```

## Usage

The easiest way to use `cargo-culture` is simply to run it from the
root directory of a Rust project:

```bash
cd my_rust_project

cargo culture
```

* More detailed usage:
  ```bash
  $ cargo culture --help

  USAGE:
      cargo culture [FLAGS] [OPTIONS]

  FLAGS:
      -h, --help       Prints help information
      -V, --version    Prints version information
      -v, --verbose    If present, emit extraneous explanations and superfluous details

  OPTIONS:
          --culture-checklist-path <culture_checklist_file_path>
              The file location of the line-separated list of Rule descriptions to check for this project

          --manifest-path <manifest_path>
              The location of the Cargo manifest for the project to check [default: ./Cargo.toml]
  ```

### Examples

* Running `cargo culture` with the default rules for a Rust project
  that needs a bit of work may look like the following:
  ```bash
  $ cargo culture
  Should have a well-formed Cargo.toml file readable by `cargo metadata` ... ok
  Should have a CONTRIBUTING file in the project directory. ... FAILED
  Should have a LICENSE file in the project directory. ... ok
  Should have a README.md file in the project directory. ... ok
  Should have a rustfmt.toml file in the project directory. ... FAILED
  Should have a file suggesting the use of a continuous integration system. ... FAILED
  Should `cargo clean` and `cargo build` without any warnings or errors. ... ok
  Should have multiple tests which pass. ... ok
  Should be making an effort to use property based tests. ... ok
  culture result: FAILED. 6 passed. 3 failed. 0 undetermined.
  ```
* You can execute `cargo culture` checks against projects not in the
  current working directory with the `--manifest-path` option.
  ```bash
  cargo culture --manifest-path $HOME/some/other/project/Cargo.toml
  ```
* To apply only a subset of available rules, you can supply a `.culture`
  file in your project directory. This file should contain a line-separated
  list of `Rule` descriptions.

  ```bash
  $ cat > .culture << EOL
  Should have a LICENSE file in the project directory.
  Should have a README.md file in the project directory.
  EOL

  $ cargo culture
  Should have a LICENSE file in the project directory. ... ok
  Should have a README.md file in the project directory. ... ok
  culture result: ok. 2 passed. 0 failed. 0 undetermined.
  ```
* You can also select a subset of rules from a checklist
  file at a specific location by providing the
  `--culture-checklist-path` option.
  ```bash
  $ cat > my_culture_checklist.txt << EOL
  Should have a LICENSE file in the project directory.
  Should have a README.md file in the project directory.
  EOL

  $ cargo culture --culture-checklist-path my_culture_checklist.txt
  Should have a LICENSE file in the project directory. ... ok
  Should have a README.md file in the project directory. ... ok
  culture result: ok. 2 passed. 0 failed. 0 undetermined.
  ```

## Tests

The `cargo-culture` tests are managed through the standard
cargo-integrated Rust test framework, with additional enhancement
through the [proptest](https://github.com/AltSysrq/proptest) property based testing library.

### Building

To build but not run the tests:

```bash
cargo build --tests
```

### Running

To both build and run the tests:

```bash
cargo test
```

# License

© 2018, PolySync Technologies, Inc.

* Zack Pierce [email](mailto:zachary.pierc.e@gmail.com)

Please see the [LICENSE](./LICENSE) file for more details
