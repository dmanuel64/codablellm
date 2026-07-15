use clap::error::ErrorKind;

/// Cargo-style user-facing error: prints `error: <msg>` and exits via
/// the downcast in main.
pub fn user_error(msg: impl std::fmt::Display) -> clap::Error {
    clap::Error::raw(ErrorKind::InvalidValue, format!("{msg}\n"))
}
