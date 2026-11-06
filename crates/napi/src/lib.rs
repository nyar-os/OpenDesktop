//! Node-API surface for `@wae/open-desktop`.
//!
//! Every platform difference belongs to the `wae` facade: this crate has no
//! `cfg` branches and no platform-specific messages. Capability facts come from
//! `open_desktop_core::capabilities()` only.

use open_desktop_core::{CapabilityReason, PlatformError, capabilities, platform_tag};
use wae::webview::{Capability, IpcHandler, OpenOptions, Platform, WebViewError, WebViewHost};

/// Native library name produced by this crate.
pub const CDYLIB_NAME: &str = "open_desktop_napi";

const GUI_CAPABILITY: &str = "gui";

/// Open the GUI host through the platform-selected `wae` backend.
///
/// The handler is supplied by the Node-API layer once IPC wiring lands.
pub fn open_gui(options: OpenOptions, handler: Box<dyn IpcHandler>) -> Result<WebViewHost, PlatformError> {
    ensure_gui()?;
    WebViewHost::open(options, handler).map_err(map_gui_error)
}

/// Pump pending window messages.
pub fn pump_events(host: &mut WebViewHost) -> bool {
    host.pump()
}

/// Ask the GUI host to close.
pub fn close_gui(host: &mut WebViewHost) -> Result<(), PlatformError> {
    host.close().map_err(map_gui_error)
}

/// Whether the GUI window is still open.
pub fn is_gui_open(host: &WebViewHost) -> bool {
    host.is_alive()
}

fn ensure_gui() -> Result<(), PlatformError> {
    let capability = capabilities().gui;
    if capability.is_available() {
        return Ok(());
    }

    let reason = capability.reason.unwrap_or(CapabilityReason::NotImplemented { scope: String::from(GUI_CAPABILITY) });
    Err(PlatformError::Unavailable { capability: String::from(GUI_CAPABILITY), reason })
}

fn map_gui_error(error: WebViewError) -> PlatformError {
    match error {
        WebViewError::Unsupported { capability, platform } => PlatformError::Unavailable {
            capability: String::from(GUI_CAPABILITY),
            reason: CapabilityReason::UnsupportedPlatform {
                platform: platform_tag(),
                detail: Some(format!("upstream webview capability {capability:?} is unavailable on {platform:?}")),
            },
        },
        WebViewError::Host(failure) => PlatformError::Failed {
            capability: String::from(GUI_CAPABILITY),
            message: format!("{} (stage {}): {}", failure.code, failure.stage, failure.message),
        },
        WebViewError::Message(message) => PlatformError::Failed { capability: String::from(GUI_CAPABILITY), message },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_unsupported_maps_to_a_structured_platform_reason() {
        let error = map_gui_error(WebViewError::unsupported(Capability::Window, Platform::Windows));

        assert_eq!(error.capability(), GUI_CAPABILITY);
        assert_eq!(error.reason().map(CapabilityReason::code), Some("unsupported_platform"));
    }

    #[test]
    fn upstream_host_failure_keeps_the_upstream_code_and_stage() {
        let error = map_gui_error(WebViewError::host("webview2_navigation_failed", "navigate", "page did not load"));

        assert_eq!(error.capability(), GUI_CAPABILITY);
        assert!(error.reason().is_none());
        assert!(format!("{error:?}").contains("webview2_navigation_failed"));
        assert!(format!("{error:?}").contains("navigate"));
    }

    #[test]
    fn upstream_runtime_message_maps_to_failed() {
        let error = map_gui_error(WebViewError::Message(String::from("window creation failed")));

        assert_eq!(error.capability(), GUI_CAPABILITY);
        assert!(error.reason().is_none());
    }

    #[test]
    fn ensure_gui_matches_the_capability_snapshot() {
        let capability = capabilities().gui;

        match ensure_gui() {
            Ok(()) => assert!(capability.is_available()),
            Err(error) => assert_eq!(error.capability(), GUI_CAPABILITY),
        }
    }
}
