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

use anstyle::AnsiColor;
use anstyle::Effects;
use anstyle::Reset;
use std::cmp::max;
use std::io::Write;
use thiserror::Error;

pub mod checks;
#[cfg(test)]
mod tests;
mod types;

pub use types::Check;
pub use types::CheckFn;
pub use types::CheckResult;

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
                results.push((CheckResult::Fatal, *name, format!("{err}")));
                worst_issues_found = CheckResult::Fatal;
            }
        }
    }
    let mut status_width = "RESULT".len();
    let mut name_width = "CHECK".len();
    for (status, name, _) in &results {
        status_width = max(
            status_width,
            <&CheckResult as Into<&str>>::into(status).len(),
        );
        name_width = max(name_width, name.len());
    }

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
