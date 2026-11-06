//! Structured capability reasons and the error surfaced when a capability is unusable.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Reason a capability cannot be used on the current host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[serde(tag = "code", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum CapabilityReason {
    /// The platform does not provide this capability at all.
    #[error("unsupported on platform {platform}")]
    UnsupportedPlatform { platform: String },
    /// The capability is planned but not delivered yet.
    #[error("{scope} is not implemented yet")]
    NotImplemented { scope: String },
    /// Delivery is blocked on an external dependency or on its review.
    #[error("blocked on {dependency}: {detail}")]
    DependencyBlocked { dependency: String, detail: String },
}

impl CapabilityReason {
    /// Stable machine-readable code, mirrored by the JSON tag.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedPlatform { .. } => "unsupported_platform",
            Self::NotImplemented { .. } => "not_implemented",
            Self::DependencyBlocked { .. } => "dependency_blocked",
        }
    }
}

/// Error returned when an operation needs a capability that is not usable.
#[derive(Debug, Error)]
#[error("{capability} unavailable: {reason}")]
pub struct PlatformError {
    /// Capability identifier, for example `protocol:rdp`.
    pub capability: String,
    /// Structured reason, safe to surface to the CLI and the UI.
    #[source]
    pub reason: CapabilityReason,
}
