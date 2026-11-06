//! Capability reporting shared by the CLI and the GUI.

use serde::{Deserialize, Serialize};

use crate::{CapabilityReason, PlatformError, Protocol};

/// Availability of one capability on the current host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    /// Whether the capability can be used right now.
    pub available: bool,
    /// Structured reason, present only when unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<CapabilityReason>,
}

impl Capability {
    /// Capability that is usable.
    #[must_use]
    pub const fn available() -> Self {
        Self { available: true, reason: None }
    }

    /// Capability that is not usable, with the reason it cannot be used.
    #[must_use]
    pub fn unavailable(reason: CapabilityReason) -> Self {
        Self { available: false, reason: Some(reason) }
    }

    /// Whether the capability is usable and carries no reason.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.available && self.reason.is_none()
    }
}

/// Capability of one protocol backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolCapability {
    /// Protocol this entry describes.
    pub protocol: Protocol,
    /// Availability fields of the protocol backend.
    #[serde(flatten)]
    pub capability: Capability,
}

/// Capability snapshot of a host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// Platform tag such as `win32-x64`.
    pub platform: String,
    /// GUI host availability.
    pub gui: Capability,
    /// One entry per protocol the session model knows about.
    pub protocols: Vec<ProtocolCapability>,
    /// Platform credential store availability.
    pub credentials: Capability,
}

impl Capabilities {
    /// Detect capabilities for the host this binary was compiled for.
    #[must_use]
    pub fn detect() -> Self {
        Self::for_platform(&platform_tag())
    }

    /// Build the capability snapshot for a platform tag.
    ///
    /// Tests and forward-looking hosts pass explicit tags instead of relying on the build target.
    #[must_use]
    pub fn for_platform(platform: &str) -> Self {
        let gui = if supports_gui(platform) { Capability::available() } else { Capability::unavailable(unsupported_platform(platform)) };

        let credentials = if platform.starts_with("win32") {
            Capability::unavailable(not_implemented("windows credential store binding"))
        }
        else {
            Capability::unavailable(unsupported_platform(platform))
        };

        let protocols = Protocol::ALL
            .iter()
            .map(|protocol| ProtocolCapability {
                protocol: *protocol,
                capability: match *protocol {
                    Protocol::Ssh => Capability::unavailable(CapabilityReason::DependencyBlocked {
                        dependency: String::from("ssh transport crate"),
                        detail: String::from("candidate selection and license review pending"),
                    }),
                    Protocol::Rdp => Capability::unavailable(not_implemented("rdp backend")),
                    Protocol::Vnc => Capability::unavailable(not_implemented("vnc backend")),
                },
            })
            .collect();

        Self { platform: platform.to_owned(), gui, protocols, credentials }
    }

    /// Find the entry of one protocol.
    #[must_use]
    pub fn protocol(&self, protocol: Protocol) -> Option<&ProtocolCapability> {
        self.protocols.iter().find(|entry| entry.protocol == protocol)
    }

    /// Whether one protocol can be used right now.
    #[must_use]
    pub fn supports(&self, protocol: Protocol) -> bool {
        self.protocol(protocol).is_some_and(|entry| entry.capability.is_available())
    }

    /// Gate an operation on the requested protocol only.
    ///
    /// A request for one protocol is never satisfied by another protocol's readiness.
    pub fn require(&self, protocol: Protocol) -> Result<(), PlatformError> {
        let reason = match self.protocol(protocol) {
            Some(entry) if entry.capability.is_available() => return Ok(()),
            Some(entry) => {
                entry.capability.reason.clone().unwrap_or_else(|| CapabilityReason::NotImplemented { scope: format!("{protocol} backend") })
            }
            None => CapabilityReason::NotImplemented { scope: format!("{protocol} backend") },
        };

        Err(PlatformError::Unavailable { capability: format!("protocol:{protocol}"), reason })
    }
}

fn supports_gui(platform: &str) -> bool {
    // The GUI host ships as the win32-x64 native package only. Reporting it as
    // available elsewhere would promise a window that no artifact can open.
    platform.starts_with("win32")
}

fn not_implemented(scope: &str) -> CapabilityReason {
    CapabilityReason::NotImplemented { scope: scope.to_owned() }
}

fn unsupported_platform(platform: &str) -> CapabilityReason {
    CapabilityReason::UnsupportedPlatform { platform: platform.to_owned(), detail: None }
}

/// Platform tag in npm form, for example `win32-x64`.
#[must_use]
pub fn platform_tag() -> String {
    let os = match std::env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" => "ia32",
        other => other,
    };

    format!("{os}-{arch}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_protocol_has_one_entry() {
        let capabilities = Capabilities::for_platform("win32-x64");

        for protocol in Protocol::ALL {
            assert!(capabilities.protocol(protocol).is_some(), "missing entry for {protocol}");
        }
        assert_eq!(capabilities.protocols.len(), Protocol::ALL.len());
    }

    #[test]
    fn ssh_transport_is_blocked_on_the_dependency_decision() {
        let capabilities = Capabilities::for_platform("win32-x64");
        let reason =
            capabilities.protocol(Protocol::Ssh).and_then(|entry| entry.capability.reason.clone()).expect("ssh transport must carry a reason");

        assert_eq!(reason.code(), "dependency_blocked");
        assert!(!capabilities.supports(Protocol::Ssh));
    }

    #[test]
    fn rdp_stays_reserved_and_never_falls_back_to_ssh() {
        let capabilities = Capabilities::for_platform("win32-x64");
        let error = capabilities.require(Protocol::Rdp).expect_err("rdp must not be usable yet");

        assert_eq!(error.capability(), "protocol:rdp");
        assert_eq!(error.reason().map(CapabilityReason::code), Some("not_implemented"));
        assert_eq!(capabilities.protocol(Protocol::Rdp).map(|entry| entry.protocol), Some(Protocol::Rdp));
    }

    #[test]
    fn gui_is_available_only_where_the_host_build_ships() {
        assert!(Capabilities::for_platform("win32-x64").gui.is_available());

        for platform in ["darwin-arm64", "linux-x64"] {
            let capabilities = Capabilities::for_platform(platform);
            let gui = capabilities.gui.reason.as_ref().expect("gui must carry a reason");

            assert_eq!(gui.code(), "unsupported_platform", "gui must not claim availability on {platform}");
        }
    }

    #[test]
    fn gui_reports_platform_unsupported_elsewhere() {
        let capabilities = Capabilities::for_platform("freebsd-x64");
        let gui = capabilities.gui.reason.as_ref().expect("gui must carry a reason");

        assert_eq!(gui.code(), "unsupported_platform");
    }

    #[test]
    fn credentials_depend_on_the_windows_store() {
        let capabilities = Capabilities::for_platform("linux-x64");
        let credentials = capabilities.credentials.reason.as_ref().expect("credentials must carry a reason");

        assert_eq!(credentials.code(), "unsupported_platform");
        assert!(!capabilities.supports(Protocol::Ssh));
    }

    #[test]
    fn snapshot_serializes_stable_codes() {
        let capabilities = Capabilities::for_platform("win32-x64");
        let json = serde_json::to_value(&capabilities).expect("snapshot must serialize");
        let entries = json["protocols"].as_array().expect("protocols must be an array");
        let rdp = entries.iter().find(|entry| entry["protocol"] == "rdp").expect("rdp entry");

        assert_eq!(rdp["available"], false);
        assert_eq!(rdp["reason"]["code"], "not_implemented");
        assert_eq!(json["gui"]["available"], true);
    }
}
