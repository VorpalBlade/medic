//! Basic types for the medic crate

use anstyle::AnsiColor;
use strum::IntoStaticStr;

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
    const fn style(&self) -> anstyle::Style {
        match self {
            Self::Ok => AnsiColor::Green.on_default(),
            Self::Info => AnsiColor::Green.on_default(),
            Self::Warning => AnsiColor::Yellow.on_default(),
            Self::Error => AnsiColor::Red.on_default(),
            Self::Fatal => AnsiColor::Red.on_default(),
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
    pub(crate) name: &'static str,
    pub(crate) func: CheckFn,
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
