# cargo-culture

Automated opinionated checks for Rust project compliance with conventions and intention towards excellence.
## How?

```
// TODO - do we need to specify nightly here?
cargo install cargo-culture

cargo culture
```

## Why?

This tool simulates having an experienced Rustacean engineer do a quick sanity-check over your project.

The rules were developed with open-source collaboration and a safety-first keep-trying-hard attitude in mind.

`cargo-culture` is subjective. It's okay if you don't agree with all of its suggestions, just like you might not
want to take 100% of your mentor's nitpicks to heart.

## Current Rules

A good project...

* Should include a well-formed Cargo.toml readable by `cargo metadata`
* Should compile without warnings or errors.
* Should have a README.md file in the root directory.
* Should contain a LICENSE file in the project root directory.
* Should contain a CONTRIBUTING file in the project root directory.
* Should contain a file suggesting the use of a continuous integration system.
* Should have multiple tests which pass.
* Should be making an effort to use property based tests.

## Future Rules

A great project...

* Should have 100% expression test coverage.
* Should be fuzz-testing all of its binaries.
* Should contain some benchmarks.
* Should have documentation for its public interfaces.

