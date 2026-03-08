//! Explicit runtime effect-queue executor for reducer-emitted side effects.

use leptos::*;

use crate::reducer::RuntimeEffect;
use crate::runtime_context::DesktopRuntimeContext;

fn take_effect_batch(mut queued: Vec<RuntimeEffect>) -> Option<Vec<RuntimeEffect>> {
    if queued.is_empty() {
        None
    } else {
        let batch = queued.clone();
        queued.clear();
        Some(batch)
    }
}

fn drain_effect_batch<F>(queued: Vec<RuntimeEffect>, mut run_effect: F)
where
    F: FnMut(RuntimeEffect),
{
    for effect in queued {
        run_effect(effect);
    }
}

/// Installs the effect executor that drains reducer-emitted runtime effects in order.
pub fn install(runtime: DesktopRuntimeContext) {
    // Clear the current queue before processing so nested dispatches enqueue a fresh batch instead
    // of being overwritten by the in-flight drain.
    create_effect(move |_| {
        let queued = runtime.effects.get();
        let Some(batch) = take_effect_batch(queued) else {
            return;
        };

        runtime.effects.set(Vec::new());
        drain_effect_batch(batch, |effect| {
            runtime.host.get_value().run_runtime_effect(runtime, effect);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::{drain_effect_batch, take_effect_batch};
    use crate::reducer::RuntimeEffect;

    #[test]
    fn take_effect_batch_returns_none_for_empty_queue() {
        assert_eq!(take_effect_batch(Vec::new()), None);
    }

    #[test]
    fn drain_effect_batch_preserves_order() {
        let mut drained = Vec::new();
        drain_effect_batch(
            vec![
                RuntimeEffect::PlaySound("first"),
                RuntimeEffect::OpenExternalUrl("https://example.com".to_string()),
            ],
            |effect| drained.push(effect),
        );

        assert_eq!(
            drained,
            vec![
                RuntimeEffect::PlaySound("first"),
                RuntimeEffect::OpenExternalUrl("https://example.com".to_string()),
            ]
        );
    }

    #[test]
    fn nested_effects_wait_for_next_batch() {
        let mut next_queue = Vec::new();
        let mut drained = Vec::new();
        let batch = take_effect_batch(vec![RuntimeEffect::PlaySound("outer")]).expect("batch");

        drain_effect_batch(batch, |effect| {
            drained.push(effect);
            next_queue.push(RuntimeEffect::PlaySound("nested"));
        });

        assert_eq!(drained, vec![RuntimeEffect::PlaySound("outer")]);
        assert_eq!(next_queue, vec![RuntimeEffect::PlaySound("nested")]);
    }
}
