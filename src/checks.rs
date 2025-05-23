//! Standard checks to include for most programs

use crate::types::Check;
use crate::types::CheckResult;

/// Provide info on the rust version used to compile the code
pub const CHECK_RUSTC_VERSION: Check = Check {
    name: "rustc-version",
    func: || {
        Ok((
            CheckResult::Ok,
            format!("{}", rustc_version_runtime::version()),
        ))
    },
};

/// Create a version check (for information only) for the crate the macro is
/// called from.
#[doc(hidden)]
#[macro_export]
macro_rules! crate_version_check {
    () => {
        Check::new("version", || {
            Ok((CheckResult::Ok, env!("CARGO_PKG_VERSION").to_string()))
        })
    };
}

#[doc(inline)]
pub use crate_version_check;

/// Provide info on the running host system and architecture
pub const CHECK_HOST: Check = Check {
    name: "host",
    func: || {
        let info = os_info::get();
        Ok((
            CheckResult::Ok,
            format!(
                "os={}, arch={}, info={}",
                std::env::consts::OS,
                std::env::consts::ARCH,
                info
            ),
        ))
    },
};
