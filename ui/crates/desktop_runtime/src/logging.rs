//! Shared observability helpers for the UI runtime crates.

use telemetry::{EnvironmentProfile, RuntimeTarget};

/// Returns the active runtime target for the compiled binary.
#[must_use]
pub const fn runtime_target() -> RuntimeTarget {
    #[cfg(target_arch = "wasm32")]
    {
        RuntimeTarget::Wasm
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        RuntimeTarget::Desktop
    }
}

/// Returns the deterministic diagnostics profile for the current build.
#[must_use]
pub const fn environment_profile() -> EnvironmentProfile {
    if cfg!(debug_assertions) {
        EnvironmentProfile::Development
    } else {
        EnvironmentProfile::Production
    }
}

/// Emits a schema-stable tracing event with the shared UI runtime fields.
#[macro_export]
macro_rules! ui_event {
    ($level:ident, $event:expr, $operation:expr, $component:expr $(, $name:ident = $value:expr )* $(,)?) => {
        tracing::$level!(
            event = %$event,
            operation = %$operation,
            component = %$component,
            runtime_target = %$crate::logging::runtime_target().as_str(),
            environment = %$crate::logging::environment_profile().as_str(),
            $($name = %$value,)*
        );
    };
}
