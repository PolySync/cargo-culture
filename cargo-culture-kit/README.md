# cargo-culture-kit

## Overview

Core library for the cargo-culture project that
provides the basic building blocks for running
straightforward checks on a Rust project repo.

The primary intended application of these checks
is within a command-line tool that contains
and enforces best practices across repositories
according to the needs of that organization.

The secondary envisioned application of these checks
is integrated into Rust tests. This case may appeal
to appeal to developers that want a rapid
deep-integration path, don't mind writing a bit of code,
and prefer to avoid the use of support programs.

## Getting Started

### Dependencies

`cargo-culture-kit` is a Rust project, and manages its dependencies with `cargo`,
available as part of the standard Rust toolchain.

* [rust](https://github.com/rust-lang-nursery/rustup.rs)

### Building

For local program development, you can build `cargo-culture-kit`
with:

* Download the project repository
  ```bash
  git clone https://github.com/PolySync/cargo-culture.git
  cd cargo-culture/cargo-culture-kit
  ```
* Execute a build
  ```bash
  cargo build
  ```

### Installation

You can include `cargo-culture-kit` in your Rust project
by adding to your Cargo.toml file.

* Added to the `[dependencies]` or ``[dev-dependencies]` section:
  ```toml
  cargo-culture-kit = "0.1"
  ```

## Usage

`check_culture_default` is the easiest way to get started,
as it provides a thin wrapper around the core `check_culture`
function in combination with the `Rule`s provided by the
`default_rules()` function.  `Rule` is the core trait of this crate. A `Rule` describes an idiom or best-practice
for projects and provides a means of evaluating whether that rule of thumb
is being upheld.

```rust
use cargo_culture_kit::{check_culture_default, IsSuccess, OutcomeStats};
use std::path::PathBuf;

let cargo_manifest = PathBuf::from("../cargo-culture/Cargo.toml");
let verbose = false;

let outcomes = check_culture_default(
    cargo_manifest, verbose, &mut std::io::stdout()
    )
    .expect("Unexpected trouble checking culture rules:");

let stats = OutcomeStats::from(outcomes);
assert!(stats.is_success());
assert_eq!(stats.fail_count, 0);
assert_eq!(stats.undetermined_count, 0);
```

### Examples

* An example of implementing your own `Rule`:
  ```rust
  use cargo_culture_kit::{CargoMetadata, Rule, RuleContext, RuleOutcome}
  #[derive(Clone, Debug, PartialEq)]
  struct IsProjectAtALuckyTime;

  impl Rule for IsProjectAtALuckyTime {
      fn description(&self) -> &str {
          "Should be lucky enough to only be tested at specific times."
      }

      fn evaluate(&self,
          _context: RuleContext,
      ) -> RuleOutcome {
          use std::time::{SystemTime, UNIX_EPOCH};
          let since_the_epoch = match SystemTime::now().duration_since(UNIX_EPOCH) {
              Ok(t) => t,
              Err(_) => return RuleOutcome::Undetermined,
          };
          if since_the_epoch.as_secs() % 2 == 0 {
              RuleOutcome::Success
          } else {
              RuleOutcome::Failure
          }
      }
  }
  ```

## Tests

The `cargo-culture-kit` tests are managed through the standard
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
