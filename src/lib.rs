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
//! RESULT    CHECK                MESSAGE
//! Info      version              3.1.2
//! Warning   build                Github CI build (not official release)
//! Info      rustc-version        1.76.0
//! Info      host                 os=linux, arch=x86_64, info=Arch Linux Rolling Release [64-bit]
//! Ok        has-chezmoi          Chezmoi found. Version: chezmoi version v2.46.1, built at 2024-02-12T09:19:56Z
//! Ok        chezmoi-override     CHEZMOI_MODIFY_MANAGER_ASSUME_CHEZMOI_VERSION is not set
//! Ok        in-path              chezmoi_modify_manager is in PATH at /home/user/bin/chezmoi_modify_manager
//! Ok        has-ignore           Ignore of **/*.src.ini found
//! Ok        no-hook-script       No legacy hook script found
//! 
//! Warning: Warning(s) found, consider investigating (especially if you have issues)
//! ```
//! 
//! The actual output is uses ANSI colour codes as well.

pub mod checks;

use anstream::println;
use anstyle::{AnsiColor, Effects, Reset};
use strum::IntoStaticStr;

/// Perform environment sanity check
///
/// Returns the worst level found (which can be passed to [`summary`])
pub fn medic<'iter>(checks: impl Iterator<Item = &'iter Check>) -> anyhow::Result<CheckResult> {
    let mut worst_issues_found = CheckResult::Ok;
    // TODO: Figure out the maximum length check name and use that for formatting
    println!(
        "{}RESULT    CHECK                MESSAGE{}",
        Effects::BOLD.render(),
        Reset.render()
    );
    for Check { name, func } in checks {
        match func() {
            Ok((result, text)) => {
                let text = text.replace('\n', "\n                               ");
                println!("{result: <9} {name: <20} {text}");
                if result >= worst_issues_found {
                    worst_issues_found = result;
                }
            }
            Err(err) => {
                println!("{:<9} {name: <20} {err}", CheckResult::Fatal);
                worst_issues_found = CheckResult::Fatal;
            }
        }
    }

    Ok(worst_issues_found)
}

/// Print summary line at the end
pub fn summary(worst_issues_found: CheckResult) {
    if worst_issues_found >= CheckResult::Error {
        println!(
            "\n{}Error{}: Error(s) found, you should rectify these for proper operation",
            AnsiColor::Red.render_fg(),
            Reset.render()
        );
    } else if worst_issues_found >= CheckResult::Warning {
        println!(
            "\n{}Warning{}: Warning(s) found, consider investigating (especially if you have issues)",
            AnsiColor::Yellow.render_fg(),
            Reset.render()
        );
    }
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
pub type CheckFn = fn() -> anyhow::Result<(CheckResult, String)>;

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
