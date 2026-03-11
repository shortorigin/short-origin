//! Desktop runtime state model, reducer, persistence hooks, and shell components.
//!
//! `desktop_runtime` is the main API surface for the managed desktop shell. It exposes:
//!
//! - data types in [`model`]
//! - state transitions in [`reduce_desktop`]
//! - persistence helpers in [`persistence`]
//! - host-side effect execution helpers in [`host`]
//! - app registration metadata and placeholder utility surfaces in [`apps`]
//! - shared UI/icon primitives re-exported from [`system_ui`]
//! - runtime provider/context wiring in the internal `runtime_context` module
//! - Leptos UI primitives in [`components`]
//! - app integration bridge types re-exported from [`desktop_app_contract`]
//!
//! The crate is intentionally layered so reducer logic stays pure, host effects stay explicit,
//! and built-in apps consume the runtime through typed contracts rather than direct host-adapter
//! imports.
//!
//! # Example
//!
//! ```rust
//! use desktop_runtime::{
//!     reduce_desktop, ApplicationId, DesktopAction, DesktopState, InteractionState,
//!     OpenWindowRequest,
//! };
//!
//! let mut state = DesktopState::default();
//! let mut interaction = InteractionState::default();
//!
//! let effects = reduce_desktop(
//!     &mut state,
//!     &mut interaction,
//!     DesktopAction::OpenWindow(OpenWindowRequest::new(
//!         ApplicationId::trusted("system.settings"),
//!     )),
//! )
//! .expect("reducer should open a window");
//!
//! assert_eq!(state.windows.len(), 1);
//! assert!(effects.iter().any(|effect| matches!(effect, desktop_runtime::RuntimeEffect::PersistLayout)));
//! ```

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

mod app_runtime;
/// Application registry metadata and app view renderers.
pub mod apps;
/// Desktop shell UI components and re-exported runtime provider/context entrypoints.
pub mod components;
/// Browser-only deterministic E2E scene configuration and query parsing helpers.
pub mod e2e;
mod effect_executor;
/// Host-side effect execution and viewport helpers used by the shell runtime.
pub mod host;
/// Core runtime state model and serializable snapshot types.
pub mod model;
/// Browser/local persistence helpers for desktop runtime state.
pub mod persistence;
/// Reducer actions and effect generation for desktop state transitions.
pub mod reducer;
mod runtime_context;
mod shell;
mod window_manager;

/// Re-exported runtime provider and shell UI entrypoints.
pub use components::{DesktopProvider, DesktopRuntimeContext, DesktopShell, use_desktop_runtime};
/// Re-exported app-runtime contract types for managed app integrations.
pub use desktop_app_contract::{
    AppCapability, AppCommand, AppEvent, AppLifecycleEvent, AppModule, AppMountContext,
    AppRegistration, AppServices, ApplicationId, CapabilitySet, IpcEnvelope, SuspendPolicy,
};
/// Re-exported browser E2E scene types used by the site entrypoint and shell.
pub use e2e::{BrowserE2eConfig, BrowserE2eScene, current_browser_e2e_config};
/// Re-exported host-side effect execution context.
pub use host::DesktopHostContext;
/// Re-exported runtime state model types.
pub use model::*;
/// Re-exported persistence entrypoints used by the shell runtime.
pub use persistence::{
    DurableDesktopSnapshot, load_boot_snapshot, load_durable_boot_snapshot,
    load_durable_boot_snapshot_record, load_theme, persist_layout_snapshot,
    persist_terminal_history, persist_theme,
};
/// Re-exported reducer entrypoint and core action/effect enums.
pub use reducer::{DesktopAction, HydrationMode, RuntimeEffect, SyncDomain, reduce_desktop};
/// Re-exported shared UI primitives for runtime-owned shell surfaces.
pub use system_ui::prelude::{Icon, IconName, IconSize};
