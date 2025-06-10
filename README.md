# Medic - helper library for self diagnostic output in CLIs

[ [lib.rs] ] [ [crates.io] ]

This is a helper for command line programs to output information on the
environment. The intended use is to mention in your bug reporting template
that you want the output of `--medic`, `--doctor` or similar. This library
will help generate that output, supplemented by your own checks specific
to your program.

Your checks might be able to catch common mistakes so that the user doesn't
even need to report a bug in the first place. But even if not, now you have
some basic info (such as platform, rust version etc) to go on.

Example output (from [chezmoi_modify_manager]):

```text
RESULT   CHECK             MESSAGE
Info     version           3.1.2
Warning  build             Github CI build (not official release)
Info     rustc-version     1.76.0
Info     host              os=linux, arch=x86_64, info=Arch Linux Rolling Release [64-bit]
Ok       has-chezmoi       Chezmoi found. Version: chezmoi version v2.46.1, built at 2024-02-12T09:19:56Z
Ok       chezmoi-override  CHEZMOI_MODIFY_MANAGER_ASSUME_CHEZMOI_VERSION is not set
Ok       in-path           chezmoi_modify_manager is in PATH at /home/user/bin/chezmoi_modify_manager
Ok       has-ignore        Ignore of **/*.src.ini found
Ok       no-hook-script    No legacy hook script found

Warning: Warning(s) found, consider investigating (especially if you have issues)
```

The actual output is uses ANSI colour codes as well.

## MSRV

Current minimum supported Rust version is 1.85.0. This may be updated as
needed. MSRV bump is not considered a semver breaking change.

## Credits

The idea was inspired by `chezmoi doctor` and implemented in [chezmoi_modify_manager],
then extracted into a separate crate to allow for reuse later on.

[chezmoi_modify_manager]: https://github.com/VorpalBlade/chezmoi_modify_manager
[crates.io]: https://crates.io/crates/medic
[lib.rs]: https://lib.rs/crates/medic
