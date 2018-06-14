//! Program exit code hinting through the `ExitCode` trait

use super::CheckError;
use super::OutcomeStats;
use super::OutcomesByDescription;
use super::RuleOutcome;
use checklist::FilterError;
use failure;

/// A means of genericizing expected process exit code
/// Once the `std::process::Termination` trait hits stable,
/// this trait may be deprecated or abstracted away.
/// See: https://github.com/rust-lang/rust/issues/43301
pub trait ExitCode {
    /// The
    fn exit_code(&self) -> i32;
}

impl ExitCode for RuleOutcome {
    fn exit_code(&self) -> i32 {
        match *self {
            RuleOutcome::Success => 0,
            RuleOutcome::Failure => 1,
            RuleOutcome::Undetermined => 2,
        }
    }
}

impl ExitCode for OutcomeStats {
    fn exit_code(&self) -> i32 {
        RuleOutcome::from(self).exit_code()
    }
}

impl ExitCode for OutcomesByDescription {
    fn exit_code(&self) -> i32 {
        OutcomeStats::from(self).exit_code()
    }
}

impl ExitCode for CheckError {
    fn exit_code(&self) -> i32 {
        match *self {
            CheckError::PrintOutputFailure(_) => 10,
        }
    }
}

impl ExitCode for FilterError {
    fn exit_code(&self) -> i32 {
        match *self {
            FilterError::RuleChecklistReadError(_) => 20,
            FilterError::RequestedRuleNotFound { .. } => 21,
        }
    }
}

impl ExitCode for failure::Error {
    fn exit_code(&self) -> i32 {
        1
    }
}

impl<T, E> ExitCode for Result<T, E>
where
    T: ExitCode,
    E: ExitCode,
{
    fn exit_code(&self) -> i32 {
        match self {
            Ok(ref r) => r.exit_code(),
            Err(e) => e.exit_code(),
        }
    }
}
