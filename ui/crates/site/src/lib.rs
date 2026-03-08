//! Web entrypoints for the Origin OS shell site.
//!
//! This crate exposes the root Leptos components and the browser mount helper used by the
//! `site_app` binary.
//!
//! In normal local development, start the browser shell with `cargo ui-dev`; that workflow
//! delegates to the same Trunk/WASM pipeline this crate expects at runtime.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

mod browser_navigation;
mod pwa;
mod web_app;

pub use web_app::{DesktopEntry, SiteApp};

#[cfg(all(feature = "csr", target_arch = "wasm32"))]
fn init_site_tracing() {
    use std::sync::Once;

    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        tracing_wasm::set_as_global_default();
    });

    tracing::info!(
        event = "site.mount.bootstrap",
        operation = "site.mount",
        component = "site",
        runtime_target = %telemetry::RuntimeTarget::Wasm.as_str(),
        environment = %if cfg!(debug_assertions) {
            telemetry::EnvironmentProfile::Development.as_str()
        } else {
            telemetry::EnvironmentProfile::Production.as_str()
        }
    );
}

#[cfg(all(feature = "csr", target_arch = "wasm32"))]
/// Mounts [`SiteApp`] into the dedicated application root for client-side rendering.
///
/// This is the browser entrypoint used by the `site_app` binary when built for `wasm32` with the
/// `csr` feature enabled.
pub fn mount() {
    init_site_tracing();
    console_error_panic_hook::set_once();
    use wasm_bindgen::JsCast;

    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        tracing::error!(
            event = "site.mount.failed",
            operation = "site.mount",
            component = "site",
            runtime_target = %telemetry::RuntimeTarget::Wasm.as_str(),
            environment = %if cfg!(debug_assertions) {
                telemetry::EnvironmentProfile::Development.as_str()
            } else {
                telemetry::EnvironmentProfile::Production.as_str()
            },
            reason = "window_document_unavailable"
        );
        return;
    };
    let Some(app_root) = document.get_element_by_id("app") else {
        tracing::error!(
            event = "site.mount.failed",
            operation = "site.mount",
            component = "site",
            runtime_target = %telemetry::RuntimeTarget::Wasm.as_str(),
            environment = %if cfg!(debug_assertions) {
                telemetry::EnvironmentProfile::Development.as_str()
            } else {
                telemetry::EnvironmentProfile::Production.as_str()
            },
            reason = "app_root_missing"
        );
        return;
    };
    let Ok(app_root) = app_root.dyn_into::<web_sys::HtmlElement>() else {
        tracing::error!(
            event = "site.mount.failed",
            operation = "site.mount",
            component = "site",
            runtime_target = %telemetry::RuntimeTarget::Wasm.as_str(),
            environment = %if cfg!(debug_assertions) {
                telemetry::EnvironmentProfile::Development.as_str()
            } else {
                telemetry::EnvironmentProfile::Production.as_str()
            },
            reason = "app_root_not_html_element"
        );
        return;
    };
    leptos::mount_to(app_root, || leptos::view! { <SiteApp /> })
}
