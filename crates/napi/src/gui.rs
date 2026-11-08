//! Capability-gated window lifecycle and the IPC surface served to the UI.
//!
//! Platform selection belongs to the `wae` facade. This module only asks the
//! capability snapshot whether the GUI host is usable and forwards the window
//! result to the audit trail through [`crate::audit::run`].

use std::cell::RefCell;

use open_desktop_core::{CapabilityReason, Operation, PlatformError, Result as CoreResult, capabilities, platform_tag};
use serde_json::{Value, json};
use wae::webview::{IpcHandler, IpcOutcome, OpenOptions, WebViewError, WebViewHost, open};

use crate::audit::{self, Summary};

const GUI_CAPABILITY: &str = "gui";

thread_local! {
    static HOST: RefCell<Option<WebViewHost>> = const { RefCell::new(None) };
}

struct HostIpc;

impl IpcHandler for HostIpc {
    fn on_ipc(&mut self, message: &str) -> IpcOutcome {
        let Ok(request) = serde_json::from_str::<Value>(message)
        else {
            return IpcOutcome::ReplyToPage(reply(json!({ "type": "error", "message": "invalid json" })));
        };
        let request_id = request.get("requestId").cloned();
        let kind = request.get("type").and_then(Value::as_str).unwrap_or_default();
        match kind {
            "capabilities" => {
                let result = audit::run(Operation::Capabilities, Value::Null, || {
                    let report = capabilities();
                    let count = report.protocols.len() as u64;
                    Ok((report, Summary::new(count, "host capabilities")))
                });
                match result {
                    Ok(report) => {
                        IpcOutcome::ReplyToPage(reply(json!({ "type": "capabilitiesResult", "requestId": request_id, "report": report })))
                    }
                    Err(error) => IpcOutcome::ReplyToPage(reply(
                        json!({ "type": "capabilitiesResult", "requestId": request_id, "error": payload(&error) }),
                    )),
                }
            }
            "close" => {
                let _ = audit::run(Operation::GuiClose, Value::Null, || Ok(((), Summary::new(1, "gui window close"))));
                IpcOutcome::CloseWindow
            }
            other => IpcOutcome::ReplyToPage(reply(
                json!({ "type": "error", "requestId": request_id, "message": format!("unknown message: {other}") }),
            )),
        }
    }
}

fn reply(value: Value) -> String {
    serde_json::to_string(&value).unwrap_or_else(|_| String::from(r#"{"type":"error","message":"serialization failed"}"#))
}

fn payload(error: &PlatformError) -> Value {
    let mut payload = json!({ "code": error.code(), "capability": error.capability(), "message": error.to_string() });
    if let Some(reason) = error.reason() {
        payload["reason"] = serde_json::to_value(reason).unwrap_or(Value::Null);
    }
    payload
}

/// Open the window for a content directory and return the backend name.
pub fn open_window(content_dir: &str, title: &str) -> CoreResult<String> {
    ensure_gui()?;
    let options = OpenOptions::from_content_dir(content_dir, title);
    let host = open(options, Box::new(HostIpc)).map_err(map_gui_error)?;
    let backend = host.backend_name().to_string();
    HOST.with(|slot| {
        *slot.borrow_mut() = Some(host);
    });
    Ok(backend)
}

/// Pump pending window messages; returns whether the window is still open.
pub fn pump() -> bool {
    HOST.with(|slot| {
        let mut slot = slot.borrow_mut();
        let alive = match slot.as_mut() {
            Some(host) => host.pump(),
            None => return false,
        };
        if !alive {
            slot.take();
        }
        alive
    })
}

/// Ask the window to close; returns whether a window was closed.
pub fn close() -> CoreResult<bool> {
    HOST.with(|slot| {
        let mut slot = slot.borrow_mut();
        match slot.take() {
            Some(mut host) => match host.close() {
                Ok(()) => Ok(true),
                Err(error) => Err(map_gui_error(error)),
            },
            None => Ok(false),
        }
    })
}

/// Whether a window is currently open.
pub fn status() -> bool {
    HOST.with(|slot| slot.borrow().as_ref().map(WebViewHost::is_alive).unwrap_or(false))
}

fn ensure_gui() -> CoreResult<()> {
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
    use wae::webview::{Capability, Platform};

    use super::*;

    #[test]
    fn invalid_json_is_reported_to_the_page() {
        let outcome = HostIpc.on_ipc("not json");

        match outcome {
            IpcOutcome::ReplyToPage(text) => {
                let value: Value = serde_json::from_str(&text).expect("reply must be json");
                assert_eq!(value["type"], "error");
                assert_eq!(value["message"], "invalid json");
            }
            _ => panic!("invalid json must produce a page reply"),
        }
    }

    #[test]
    fn unknown_messages_are_reported_to_the_page() {
        let outcome = HostIpc.on_ipc(r#"{"type":"sessions"}"#);

        match outcome {
            IpcOutcome::ReplyToPage(text) => {
                let value: Value = serde_json::from_str(&text).expect("reply must be json");
                assert_eq!(value["type"], "error");
                assert!(value["message"].as_str().is_some_and(|message| message.contains("unknown message")));
            }
            _ => panic!("unknown messages must produce a page reply"),
        }
    }

    #[test]
    fn closing_without_a_window_is_a_no_op() {
        assert!(!close().expect("close must not fail without a window"));
    }

    #[test]
    fn status_without_a_window_is_false() {
        assert!(!status());
        assert!(!pump());
    }

    #[test]
    fn upstream_unsupported_maps_to_a_structured_platform_reason() {
        let error = map_gui_error(WebViewError::unsupported(Capability::Window, Platform::Windows));

        assert_eq!(error.capability(), GUI_CAPABILITY);
        assert_eq!(error.reason().map(CapabilityReason::code), Some("unsupported_platform"));
        assert_eq!(error.code(), "unsupported_platform");
    }

    #[test]
    fn upstream_host_failure_keeps_the_upstream_code_and_stage() {
        let error = map_gui_error(WebViewError::host("webview2_navigation_failed", "navigate", "page did not load"));

        assert_eq!(error.capability(), GUI_CAPABILITY);
        assert!(error.reason().is_none());
        assert_eq!(error.code(), "failed");
        assert!(format!("{error:?}").contains("webview2_navigation_failed"));
        assert!(format!("{error:?}").contains("navigate"));
    }

    #[test]
    fn upstream_runtime_message_maps_to_failed() {
        let error = map_gui_error(WebViewError::Message(String::from("window creation failed")));

        assert_eq!(error.capability(), GUI_CAPABILITY);
        assert!(error.reason().is_none());
        assert_eq!(error.code(), "failed");
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
