//! Session model and capability reporting for `@wae/open-desktop`.

/// Core crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
