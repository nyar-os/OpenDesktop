//! Operation names and audit modes shared by every capability entry point.

/// Audit mode recorded for one operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditMode {
    /// The operation only reads host state.
    DryRun,
    /// The operation changes host state.
    Execute,
}

/// Operation exposed by the CLI and the GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Report the application version.
    Version,
    /// Report the capability snapshot.
    Capabilities,
    /// Query the shared audit trail.
    AuditQuery,
    /// Replace audit attribution.
    AuditConfigure,
    /// Open the desktop window.
    GuiOpen,
    /// Pump pending window messages.
    GuiPump,
    /// Close the desktop window.
    GuiClose,
    /// Query window state.
    GuiStatus,
}

impl Operation {
    /// Stable capability identifier written into the audit trail.
    #[must_use]
    pub const fn capability(self) -> &'static str {
        match self {
            Self::Version => "version",
            Self::Capabilities => "capabilities",
            Self::AuditQuery => "audit-query",
            Self::AuditConfigure => "audit-configure",
            Self::GuiOpen => "gui-open",
            Self::GuiPump => "gui-pump",
            Self::GuiClose => "gui-close",
            Self::GuiStatus => "gui-status",
        }
    }

    /// Audit mode of the operation.
    #[must_use]
    pub const fn mode(self) -> AuditMode {
        match self {
            Self::GuiOpen | Self::GuiClose => AuditMode::Execute,
            Self::Version | Self::Capabilities | Self::AuditQuery | Self::AuditConfigure | Self::GuiPump | Self::GuiStatus => AuditMode::DryRun,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPERATIONS: [Operation; 8] = [
        Operation::Version,
        Operation::Capabilities,
        Operation::AuditQuery,
        Operation::AuditConfigure,
        Operation::GuiOpen,
        Operation::GuiPump,
        Operation::GuiClose,
        Operation::GuiStatus,
    ];

    #[test]
    fn capability_ids_are_stable_and_unique() {
        let mut ids: Vec<&str> = OPERATIONS.iter().map(|operation| operation.capability()).collect();
        ids.sort_unstable();
        ids.dedup();

        assert_eq!(ids.len(), OPERATIONS.len());
        assert_eq!(Operation::Version.capability(), "version");
        assert_eq!(Operation::GuiOpen.capability(), "gui-open");
    }

    #[test]
    fn only_window_open_and_close_are_executed() {
        for operation in OPERATIONS {
            let expected = matches!(operation, Operation::GuiOpen | Operation::GuiClose);
            assert_eq!(operation.mode() == AuditMode::Execute, expected, "unexpected mode for {}", operation.capability());
        }
    }
}
