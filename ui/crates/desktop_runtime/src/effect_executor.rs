//! Explicit runtime effect-queue executor for reducer-emitted side effects.

use leptos::*;

use crate::runtime_context::DesktopRuntimeContext;

/// Installs the effect executor that drains reducer-emitted runtime effects in order.
pub fn install(runtime: DesktopRuntimeContext) {
    // Clear the current queue before processing so nested dispatches enqueue a fresh batch instead
    // of being overwritten by the in-flight drain.
    create_effect(move |_| {
        let queued = runtime.effects.get();
        if queued.is_empty() {
            return;
        }

        runtime.effects.set(Vec::new());

        for effect in queued {
            runtime.host.get_value().run_runtime_effect(runtime, effect);
        }
    });
}
