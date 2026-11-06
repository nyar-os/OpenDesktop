//! Transport interface reserved for protocol backends.

use crate::{PlatformError, Protocol, SessionProfile};

/// Backend that owns one protocol's connection lifecycle.
///
/// No implementation is registered yet. SSH library candidates and their
/// licenses are still under review, so [`Capabilities`](crate::Capabilities)
/// reports the transport as dependency-blocked. A backend serves exactly one
/// protocol and must never stand in for another.
pub trait SessionTransport {
    /// Protocol served by this backend.
    fn protocol(&self) -> Protocol;

    /// Open the connection described by the profile.
    fn open(&mut self, profile: &SessionProfile) -> Result<(), PlatformError>;

    /// Close the connection if it is open.
    fn close(&mut self) -> Result<(), PlatformError>;
}
