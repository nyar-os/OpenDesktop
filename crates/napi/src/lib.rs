//! Node-API surface for `@wae/open-desktop`.
//!
//! The WAE facade and the first exported functions land with the domain exports.

/// Native library name produced by this crate.
pub const CDYLIB_NAME: &str = "open_desktop_napi";

/// Core crate version exposed through the native surface.
#[must_use]
pub fn core_version() -> &'static str {
    open_desktop_core::version()
}
