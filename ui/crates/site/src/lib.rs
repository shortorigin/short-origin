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
/// Mounts [`SiteApp`] into the dedicated application root for client-side rendering.
///
/// This is the browser entrypoint used by the `site_app` binary when built for `wasm32` with the
/// `csr` feature enabled.
pub fn mount() {
    console_error_panic_hook::set_once();
    let environment = telemetry::EnvironmentProfile::from_optional(
        option_env!("ORIGIN_ENVIRONMENT"),
        telemetry::EnvironmentProfile::Development,
    );
    let tracing_config =
        telemetry::TracingBootstrapConfig::browser("ui/crates/site", environment.clone());
    if let Err(error) = telemetry::bootstrap_browser_tracing(&tracing_config) {
        web_sys::console::error_1(&format!("failed to bootstrap browser tracing: {error}").into());
    } else {
        tracing::info!(
            event = "ui_bootstrap",
            operation = "mount",
            component = tracing_config.component.as_str(),
            runtime_target = tracing_config.runtime_target.as_str(),
            environment = tracing_config.environment.as_str(),
            "browser tracing bootstrap initialized"
        );
    }
    use wasm_bindgen::JsCast;

    let document = web_sys::window()
        .and_then(|window| window.document())
        .expect("window document should be available for CSR mount");
    let app_root = document
        .get_element_by_id("app")
        .expect("site app root element should exist")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("site app root should be an HtmlElement");
    leptos::mount::mount_to(app_root, || leptos::view! { <SiteApp /> }).forget();
}
