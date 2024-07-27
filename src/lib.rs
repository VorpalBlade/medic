//! # Sanity checking of environment
//! This is a helper for command line programs to output information on the
//! environment. The intended use is to mention in your bug reporting template
//! that you want the output of `--medic`, `--doctor` or similar. This library
//! will help generate that output, supplemented by your own checks specific
//! to your program.
//!
//! Your checks might be able to catch common mistakes so that the user doesn't
//! even need to report a bug in the first place. But even if not, now you have
//! some basic info (such as platform, rust version etc) to go on.
//!
//! Example output (from [chezmoi_modify_manager](https://github.com/VorpalBlade/chezmoi_modify_manager)):
//!
//! ```text
//! RESULT   CHECK             MESSAGE
//! Info     version           3.1.2
//! Warning  build             Github CI build (not official release)
//! Info     rustc-version     1.76.0
//! Info     host              os=linux, arch=x86_64, info=Arch Linux Rolling Release [64-bit]
//! Ok       has-chezmoi       Chezmoi found. Version: chezmoi version v2.46.1, built at 2024-02-12T09:19:56Z
//! Ok       chezmoi-override  CHEZMOI_MODIFY_MANAGER_ASSUME_CHEZMOI_VERSION is not set
//! Ok       in-path           chezmoi_modify_manager is in PATH at /home/user/bin/chezmoi_modify_manager
//! Ok       has-ignore        Ignore of **/*.src.ini found
//! Ok       no-hook-script    No legacy hook script found
//!
//! Warning: Warning(s) found, consider investigating (especially if you have issues)
//! ```
//!
//! The actual output is uses ANSI colour codes as well.

use std::cmp::max;
use std::io::Write;

use anstyle::AnsiColor;
use anstyle::Effects;
use anstyle::Reset;
use strum::IntoStaticStr;
use thiserror::Error;

pub mod checks;
#[cfg(test)]
mod tests;

/// Error from medic
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MedicError {
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("Error from check")]
    CheckError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Perform environment sanity check
///
/// Returns the worst level found (which can be passed to [`summary`])
pub fn medic<'iter>(
    output: &mut impl Write,
    checks: impl Iterator<Item = &'iter Check>,
) -> Result<CheckResult, MedicError> {
    let mut worst_issues_found = CheckResult::Ok;
    // Buffer output messages so that we can format them in a nice table
    let mut results = vec![];

    for Check { name, func } in checks {
        match func() {
            Ok((result, text)) => {
                results.push((result, *name, text));
                if result >= worst_issues_found {
                    worst_issues_found = result;
                }
            }
            Err(err) => {
                results.push((CheckResult::Fatal, *name, format!("{}", err)));
                worst_issues_found = CheckResult::Fatal;
            }
        }
    }
    let mut status_width = "RESULT".len();
    let mut name_width = "CHECK".len();
    results.iter().for_each(|(status, name, _)| {
        status_width = max(
            status_width,
            <&CheckResult as Into<&str>>::into(status).len(),
        );
        name_width = max(name_width, name.len());
    });

    let text_alignment = status_width + name_width + 4;

    writeln!(
        output,
        "{}{: <status_width$}  {: <name_width$}  MESSAGE{}",
        Effects::BOLD.render(),
        "RESULT",
        "CHECK",
        Reset.render()
    )?;
    for (status, name, text) in results {
        let text = text.replace(
            '\n',
            &("\n".to_owned() + " ".repeat(text_alignment).as_str()),
        );
        writeln!(
            output,
            "{status: <status_width$}  {name: <name_width$}  {text}"
        )?;
    }

    Ok(worst_issues_found)
}

/// Print summary line at the end
pub fn summary(output: &mut impl Write, worst_issues_found: CheckResult) -> Result<(), MedicError> {
    if worst_issues_found >= CheckResult::Error {
        writeln!(
            output,
            "\n{}Error{}: Error(s) found, you should rectify these for proper operation",
            AnsiColor::Red.render_fg(),
            Reset.render()
        )?;
    } else if worst_issues_found >= CheckResult::Warning {
        writeln!(
            output,
            "\n{}Warning{}: Warning(s) found, consider investigating (especially if you have \
             issues)",
            AnsiColor::Yellow.render_fg(),
            Reset.render()
        )?;
    }
    Ok(())
}

/// Result of a check (the level of severity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntoStaticStr)]
pub enum CheckResult {
    /// This notes a value that is within expected parameters
    Ok,
    /// Information, not a problem in itself (but might be interesting for
    /// troubleshooting if some optional dependency is missing for example).
    Info,
    /// Warning, something that might be a problem
    Warning,
    /// Error, definitely a problem
    Error,
    /// Fatal error, the check itself couldn't complete (returned an [`Err`])
    Fatal,
}

impl CheckResult {
    /// Get style for this severity level
    fn style(&self) -> anstyle::Style {
        match self {
            CheckResult::Ok => anstyle::AnsiColor::Green.on_default(),
            CheckResult::Info => anstyle::AnsiColor::Green.on_default(),
            CheckResult::Warning => anstyle::AnsiColor::Yellow.on_default(),
            CheckResult::Error => anstyle::AnsiColor::Red.on_default(),
            CheckResult::Fatal => anstyle::AnsiColor::Red.on_default(),
        }
    }
}

/// Coloured formatting of check result
impl std::fmt::Display for CheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let style = self.style();
        let rendered = style.render();
        let reset = style.render_reset();
        let stringified: &'static str = self.into();

        // This may seem strange, but ensures that the formatting settings of f
        // (in particular field width) gets passed on to `stringified`, but not
        // to the format string.
        // See also https://github.com/rust-cli/anstyle/issues/167
        write!(f, "{rendered}")?;
        stringified.fmt(f)?;
        write!(f, "{reset}")?;
        Ok(())
    }
}

/// Type of function that performs a check
///
/// This should return the severity level and a message describing the situation
///
/// Multi-line messages are supported, the framework handles alignment.
pub type CheckFn = fn() -> Result<(CheckResult, String), Box<dyn std::error::Error + Send + Sync>>;

/// A check with a name
#[derive(Debug)]
pub struct Check {
    name: &'static str,
    func: CheckFn,
}

impl Check {
    /// Create a new check
    ///
    /// * `name`: Name of check (for display)
    /// * `func`: Function to perform the check
    pub const fn new(name: &'static str, func: CheckFn) -> Self {
        Self { name, func }
    }
}
