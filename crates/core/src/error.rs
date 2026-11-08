//! Structured capability reasons and the error surfaced when a capability is unusable.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Reason a capability cannot be used on the current host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum CapabilityReason {
    /// The platform does not provide this capability.
    UnsupportedPlatform {
        /// Platform tag such as `linux-x64`.
        platform: String,
        /// Message reported by the upstream provider, when it has one.
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    /// The capability is planned but not delivered yet.
    NotImplemented { scope: String },
    /// Delivery is blocked on an external dependency or on its review.
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

impl fmt::Display for CapabilityReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform { platform, detail: Some(detail) } => {
                write!(f, "unsupported on platform {platform}: {detail}")
            }
            Self::UnsupportedPlatform { platform, detail: None } => write!(f, "unsupported on platform {platform}"),
            Self::NotImplemented { scope } => write!(f, "{scope} is not implemented yet"),
            Self::DependencyBlocked { dependency, detail } => write!(f, "blocked on {dependency}: {detail}"),
        }
    }
}

impl std::error::Error for CapabilityReason {}

/// Error returned when an operation cannot use a capability.
#[derive(Debug, Error)]
pub enum PlatformError {
    /// The capability exists in the contract but is not usable.
    #[error("{capability} unavailable: {reason}")]
    Unavailable {
        /// Capability identifier, for example `protocol:rdp`.
        capability: String,
        /// Structured reason, safe to surface to the CLI and the UI.
        #[source]
        reason: CapabilityReason,
    },
    /// The capability is usable but the provider failed at runtime.
    #[error("{capability} failed: {message}")]
    Failed {
        /// Capability identifier, for example `gui`.
        capability: String,
        /// Provider message, localized further down the stack.
        message: String,
    },
}

impl PlatformError {
    /// Capability identifier carried by either variant.
    #[must_use]
    pub fn capability(&self) -> &str {
        match self {
            Self::Unavailable { capability, .. } | Self::Failed { capability, .. } => capability,
        }
    }

    /// Structured reason, present only when the capability itself is unavailable.
    #[must_use]
    pub fn reason(&self) -> Option<&CapabilityReason> {
        match self {
            Self::Unavailable { reason, .. } => Some(reason),
            Self::Failed { .. } => None,
        }
    }

    /// Stable machine-readable code, safe for audit records and CLI exits.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unavailable { reason, .. } => reason.code(),
            Self::Failed { .. } => "failed",
        }
    }
}

/// Result of an operation that can refuse through a capability.
pub type Result<T> = std::result::Result<T, PlatformError>;
