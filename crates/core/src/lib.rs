//! Session model and capability reporting for `@wae/open-desktop`.

mod capabilities;
mod error;
mod session;
mod transport;

pub use capabilities::{Capabilities, Capability, ProtocolCapability, platform_tag};
pub use error::{CapabilityReason, PlatformError};
pub use session::{AuthKind, AuthRef, Protocol, SessionMetadata, SessionProfile};
pub use transport::SessionTransport;

/// Capability snapshot of the current host.
#[must_use]
pub fn capabilities() -> Capabilities {
    Capabilities::detect()
}

/// Core crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
