# cargo-culture

## Overview

Automated opinionated checks for Rust project compliance
with useful conventions and intention towards excellence.

This tool simulates having an experienced Rustacean engineer do
a quick sanity-check review of your project, available with
the convenience of a `cargo` subcommand, `cargo culture`.

The rules it checks were developed with open-source collaboration
and a safety-first keep-trying-hard attitude in mind.
`cargo-culture`'s default rules assess project-level characteristics,
not code style or detailed code quality. If you're interested in
checking those, try [rustfmt](https://github.com/rust-lang-nursery/rustfmt)
and [clippy](https://github.com/rust-lang-nursery/rust-clippy) instead.

`cargo-culture` is subjective. It's okay if you don't agree with
all of its suggestions, just like you might not want to take
100% of your mentor's nitpicks to heart.

In addition to a command-line tool, this project supplies
a library, `cargo-culture-kit`, which organizations can
use to create their own customized rule-sets and apply
them consistently across their repositories.


## Getting Started

### Dependencies

`cargo-culture` is a Rust project, and manages its dependencies with `cargo`,
available as part of the standard Rust toolchain.

* [rust](https://github.com/rust-lang-nursery/rustup.rs)

### Building

This repository is organized as a Cargo workspace, split between
[cargo-culture-kit](./cargo-culture-kit), the core library for rule-checking,
and [cargo-culture](./cargo-culture), a command-line program thinly wrapping
the library. Both can be built from the project root directory.

* Download the project repository
  ```bash
  git clone https://github.com/PolySync/cargo-culture.git
  cd cargo-culture
  ```
* Execute a full-workspace build
  ```bash
  cargo build
  ```

### Installation

You can install `cargo-culture` directly from [crates.io](https://crates.io).

* Use cargo's built-in binary installation command
  ```bash
  cargo install cargo-culture
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
* If you wish to develop your own set of rules to apply
  either through a binary tool or as part of a test suite,
  you can directly use the [cargo-culture-kit](./cargo-culture-kit)
  library by adding it to your Cargo.toml file dependencies:
  ```toml
  cargo-culture-kit = "0.1"
  ```

## Tests

The `cargo-culture` projects tests are managed through the standard
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
