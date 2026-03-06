use crate::{
    app_runtime::{
        deliver_window_event, publish_topic_event, set_window_lifecycle, subscribe_window_topic,
        unsubscribe_window_topic,
    },
    components::DesktopRuntimeContext,
};

pub(super) fn dispatch_lifecycle(
    runtime: DesktopRuntimeContext,
    window_id: crate::model::WindowId,
    event: desktop_app_contract::AppLifecycleEvent,
) {
    set_window_lifecycle(runtime.app_runtime, window_id, event);
}

pub(super) fn deliver_app_event(
    runtime: DesktopRuntimeContext,
    window_id: crate::model::WindowId,
    event: desktop_app_contract::AppEvent,
) {
    deliver_window_event(runtime.app_runtime, window_id, event);
}

pub(super) fn subscribe_topic(
    runtime: DesktopRuntimeContext,
    window_id: crate::model::WindowId,
    topic: String,
) {
    subscribe_window_topic(runtime.app_runtime, window_id, &topic);
}

pub(super) fn unsubscribe_topic(
    runtime: DesktopRuntimeContext,
    window_id: crate::model::WindowId,
    topic: String,
) {
    unsubscribe_window_topic(runtime.app_runtime, window_id, &topic);
}

pub(super) fn publish_event(
    runtime: DesktopRuntimeContext,
    source_window_id: crate::model::WindowId,
    topic: String,
    payload: serde_json::Value,
    correlation_id: Option<String>,
    reply_to: Option<String>,
) {
    publish_topic_event(
        runtime.app_runtime,
        source_window_id,
        &topic,
        payload,
        correlation_id,
        reply_to,
    );
}
