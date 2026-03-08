//! Standards-based same-origin cross-context synchronization helpers.

use serde::{Deserialize, Serialize};

/// Browser state domains that participate in shell synchronization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShellSyncKind {
    /// Theme or accessibility preferences changed.
    Theme,
    /// Wallpaper selection or wallpaper metadata changed.
    Wallpaper,
    /// Desktop layout state changed.
    Layout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Shell-sync event envelope exchanged across same-origin browser contexts.
pub struct ShellSyncEvent {
    /// Domain affected by the change.
    pub kind: ShellSyncKind,
    /// Stable sender identity for the current browser context.
    pub sender_id: String,
    /// Monotonic revision for stale-event suppression.
    pub revision: u64,
}

impl ShellSyncEvent {
    /// Creates a new event using the current browser context sender identity.
    pub fn new(kind: ShellSyncKind, revision: u64) -> Self {
        Self {
            kind,
            sender_id: shell_sync_sender_id(),
            revision,
        }
    }
}

/// Returns whether the current browser exposes the `BroadcastChannel` API.
pub fn broadcast_channel_supported() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| {
                js_sys::Reflect::has(window.as_ref(), &"BroadcastChannel".into()).ok()
            })
            .unwrap_or(false)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Returns the stable sender identity for this browser context.
pub fn shell_sync_sender_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        thread_local! {
            static SHELL_SYNC_SENDER_ID: String = format!(
                "origin-shell-{}",
                platform_host::next_monotonic_timestamp_ms()
            );
        }

        SHELL_SYNC_SENDER_ID.with(Clone::clone)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        "origin-shell-test".to_string()
    }
}

/// Returns whether an incoming sync event should be applied locally.
pub fn should_apply_shell_sync_event(
    incoming: &ShellSyncEvent,
    current_sender_id: &str,
    last_applied_revision: Option<u64>,
) -> bool {
    if incoming.sender_id == current_sender_id {
        return false;
    }

    last_applied_revision.is_none_or(|current| incoming.revision > current)
}

/// Encodes an event into a transport payload.
pub fn encode_shell_sync_event(event: &ShellSyncEvent) -> Result<String, String> {
    serde_json::to_string(event).map_err(|error| error.to_string())
}

/// Decodes a transport payload into a typed sync event.
pub fn decode_shell_sync_event(raw: &str) -> Option<ShellSyncEvent> {
    serde_json::from_str(raw).ok()
}

/// Publishes a shell-sync event to other same-origin browser contexts.
pub fn publish_shell_sync_event(event: &ShellSyncEvent) {
    #[cfg(target_arch = "wasm32")]
    {
        if !broadcast_channel_supported() {
            return;
        }

        if let Ok(channel) = web_sys::BroadcastChannel::new("origin-os-shell-sync") {
            if let Ok(message) = encode_shell_sync_event(event) {
                let _ = channel.post_message(&wasm_bindgen::JsValue::from_str(&message));
            }
            channel.close();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = event;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_self_originated_events() {
        let event = ShellSyncEvent {
            kind: ShellSyncKind::Layout,
            sender_id: "tab-a".to_string(),
            revision: 10,
        };
        assert!(!should_apply_shell_sync_event(&event, "tab-a", Some(9)));
    }

    #[test]
    fn ignores_stale_events() {
        let event = ShellSyncEvent {
            kind: ShellSyncKind::Theme,
            sender_id: "tab-b".to_string(),
            revision: 9,
        };
        assert!(!should_apply_shell_sync_event(&event, "tab-a", Some(9)));
        assert!(!should_apply_shell_sync_event(&event, "tab-a", Some(10)));
    }

    #[test]
    fn accepts_newer_cross_context_events() {
        let event = ShellSyncEvent {
            kind: ShellSyncKind::Wallpaper,
            sender_id: "tab-b".to_string(),
            revision: 11,
        };
        assert!(should_apply_shell_sync_event(&event, "tab-a", Some(10)));
    }

    #[test]
    fn event_round_trip_is_stable() {
        let event = ShellSyncEvent {
            kind: ShellSyncKind::Layout,
            sender_id: "tab-a".to_string(),
            revision: 42,
        };
        let encoded = encode_shell_sync_event(&event).expect("encode shell sync event");
        let decoded = decode_shell_sync_event(&encoded).expect("decode shell sync event");
        assert_eq!(decoded, event);
    }
}
