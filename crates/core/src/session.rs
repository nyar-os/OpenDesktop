//! Session profiles and the protocol enum shared by the CLI and the GUI.

use serde::{Deserialize, Serialize};

/// Connection protocol backing a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Ssh,
    Rdp,
    Vnc,
}

impl Protocol {
    /// Every protocol the session model knows about.
    pub const ALL: [Self; 3] = [Self::Ssh, Self::Rdp, Self::Vnc];

    /// Lowercase protocol name used in JSON and CLI output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ssh => "ssh",
            Self::Rdp => "rdp",
            Self::Vnc => "vnc",
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Kind of credential reference held by a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthKind {
    Password,
    PrivateKey,
    Agent,
}

/// Opaque handle into the platform credential store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthRef {
    /// Credential kind used by the session.
    pub kind: AuthKind,
    /// Credential-store handle. Never plaintext secret material.
    pub reference: String,
}

/// Non-secret session metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Free-form tags used for grouping.
    pub tags: Vec<String>,
    /// Optional operator notes.
    pub notes: Option<String>,
    /// Creation timestamp in Unix milliseconds.
    pub created_at_unix_ms: u64,
    /// Last successful use in Unix milliseconds.
    pub last_used_at_unix_ms: Option<u64>,
}

/// A saved connection profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionProfile {
    /// Stable profile identifier.
    pub id: String,
    /// Display name shown in the session list.
    pub name: String,
    /// Protocol this profile connects with.
    pub protocol: Protocol,
    /// Host name or address.
    pub host: String,
    /// TCP port for the protocol.
    pub port: u16,
    /// Credential reference, never plaintext.
    pub auth: AuthRef,
    /// Non-secret metadata.
    pub metadata: SessionMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_profile() -> SessionProfile {
        SessionProfile {
            id: "s1".to_owned(),
            name: "build box".to_owned(),
            protocol: Protocol::Ssh,
            host: "example.test".to_owned(),
            port: 22,
            auth: AuthRef { kind: AuthKind::PrivateKey, reference: "cred:build-box".to_owned() },
            metadata: SessionMetadata::default(),
        }
    }

    #[test]
    fn profile_serializes_protocol_and_auth_as_snake_case() {
        let json = serde_json::to_value(sample_profile()).expect("profile must serialize");
        assert_eq!(json["protocol"], "ssh");
        assert_eq!(json["auth"]["kind"], "private_key");
        assert_eq!(json["port"], 22);
    }

    #[test]
    fn protocol_names_stay_stable() {
        assert_eq!(Protocol::ALL.map(Protocol::as_str), ["ssh", "rdp", "vnc"]);
    }
}
