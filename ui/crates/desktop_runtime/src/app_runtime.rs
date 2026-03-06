//! Runtime app-session and pub/sub bus state owned by the desktop shell.

use std::collections::{BTreeSet, HashMap};

use desktop_app_contract::{AppEvent, AppLifecycleEvent};
use leptos::*;
use platform_host::unix_time_ms_now;

use crate::model::{WindowId, WindowRecord};
const MAX_INBOX_EVENTS: usize = 256;

#[derive(Clone, Copy)]
/// Reactive per-window app session signals.
pub struct WindowAppSession {
    /// Latest lifecycle signal value for the window.
    pub lifecycle: RwSignal<AppLifecycleEvent>,
    /// App-bus inbox for the window.
    pub inbox: RwSignal<Vec<AppEvent>>,
}

#[derive(Default)]
/// Runtime-owned app session and topic subscription state.
pub struct AppRuntimeState {
    sessions: HashMap<WindowId, WindowAppSession>,
    topic_subscribers: HashMap<String, BTreeSet<WindowId>>,
}

impl AppRuntimeState {
    fn ensure_session(&mut self, window_id: WindowId) -> WindowAppSession {
        if let Some(session) = self.sessions.get(&window_id).copied() {
            return session;
        }

        let session = WindowAppSession {
            lifecycle: create_rw_signal(AppLifecycleEvent::Mounted),
            inbox: create_rw_signal(Vec::new()),
        };
        self.sessions.insert(window_id, session);
        session
    }

    fn remove_session(&mut self, window_id: WindowId) {
        self.sessions.remove(&window_id);
        for subscribers in self.topic_subscribers.values_mut() {
            subscribers.remove(&window_id);
        }
        self.topic_subscribers
            .retain(|_, subscribers| !subscribers.is_empty());
    }

    fn set_lifecycle(&mut self, window_id: WindowId, event: AppLifecycleEvent) {
        let session = self.ensure_session(window_id);
        session.lifecycle.set(event);
    }

    fn deliver_event(&mut self, window_id: WindowId, event: AppEvent) {
        let session = self.ensure_session(window_id);
        session.inbox.update(|inbox| {
            inbox.push(event);
            if inbox.len() > MAX_INBOX_EVENTS {
                let overflow = inbox.len() - MAX_INBOX_EVENTS;
                inbox.drain(0..overflow);
            }
        });
    }

    fn subscribe(&mut self, window_id: WindowId, topic: &str) {
        self.ensure_session(window_id);
        self.topic_subscribers
            .entry(topic.to_string())
            .or_default()
            .insert(window_id);
    }

    fn unsubscribe(&mut self, window_id: WindowId, topic: &str) {
        if let Some(subscribers) = self.topic_subscribers.get_mut(topic) {
            subscribers.remove(&window_id);
            if subscribers.is_empty() {
                self.topic_subscribers.remove(topic);
            }
        }
    }

    fn publish(
        &mut self,
        source_window_id: WindowId,
        topic: &str,
        payload: serde_json::Value,
        correlation_id: Option<String>,
        reply_to: Option<String>,
    ) {
        let Some(subscribers) = self.topic_subscribers.get(topic).cloned() else {
            return;
        };
        let mut stale_subscribers = Vec::new();

        for target in subscribers {
            if self.sessions.contains_key(&target) {
                let mut event = AppEvent::new(topic, payload.clone(), Some(source_window_id.0));
                event.correlation_id = correlation_id.clone();
                event.reply_to = reply_to.clone();
                event.timestamp_unix_ms = Some(unix_time_ms_now());
                self.deliver_event(target, event);
            } else {
                stale_subscribers.push(target);
            }
        }

        if !stale_subscribers.is_empty() {
            if let Some(topic_subscribers) = self.topic_subscribers.get_mut(topic) {
                for stale in stale_subscribers {
                    topic_subscribers.remove(&stale);
                }
                if topic_subscribers.is_empty() {
                    self.topic_subscribers.remove(topic);
                }
            }
        }
    }

    fn sync_windows(&mut self, windows: &[WindowRecord]) {
        let active: BTreeSet<WindowId> = windows.iter().map(|win| win.id).collect();

        for window_id in &active {
            self.ensure_session(*window_id);
        }

        let stale: Vec<WindowId> = self
            .sessions
            .keys()
            .copied()
            .filter(|window_id| !active.contains(window_id))
            .collect();

        for window_id in stale {
            self.remove_session(window_id);
        }
    }
}

/// Ensures and returns a per-window runtime app session.
pub fn ensure_window_session(
    runtime_state: RwSignal<AppRuntimeState>,
    window_id: WindowId,
) -> WindowAppSession {
    if let Some(session) =
        runtime_state.with_untracked(|state| state.sessions.get(&window_id).copied())
    {
        return session;
    }

    let mut session = None;
    runtime_state.update(|state| {
        session = Some(state.ensure_session(window_id));
    });
    session.expect("window app session ensured")
}

/// Syncs app runtime session state with currently open windows.
pub fn sync_runtime_sessions(runtime_state: RwSignal<AppRuntimeState>, windows: &[WindowRecord]) {
    runtime_state.update(|state| state.sync_windows(windows));
}

/// Applies an app lifecycle event to a window session.
pub fn set_window_lifecycle(
    runtime_state: RwSignal<AppRuntimeState>,
    window_id: WindowId,
    event: AppLifecycleEvent,
) {
    runtime_state.update(|state| state.set_lifecycle(window_id, event));
}

/// Delivers an app event directly to a specific window inbox.
pub fn deliver_window_event(
    runtime_state: RwSignal<AppRuntimeState>,
    window_id: WindowId,
    event: AppEvent,
) {
    runtime_state.update(|state| state.deliver_event(window_id, event));
}

/// Adds a topic subscription for a window.
pub fn subscribe_window_topic(
    runtime_state: RwSignal<AppRuntimeState>,
    window_id: WindowId,
    topic: &str,
) {
    runtime_state.update(|state| state.subscribe(window_id, topic));
}

/// Removes a topic subscription for a window.
pub fn unsubscribe_window_topic(
    runtime_state: RwSignal<AppRuntimeState>,
    window_id: WindowId,
    topic: &str,
) {
    runtime_state.update(|state| state.unsubscribe(window_id, topic));
}

/// Publishes an event to all subscribers of `topic`.
pub fn publish_topic_event(
    runtime_state: RwSignal<AppRuntimeState>,
    source_window_id: WindowId,
    topic: &str,
    payload: serde_json::Value,
    correlation_id: Option<String>,
    reply_to: Option<String>,
) {
    runtime_state
        .update(|state| state.publish(source_window_id, topic, payload, correlation_id, reply_to));
}
