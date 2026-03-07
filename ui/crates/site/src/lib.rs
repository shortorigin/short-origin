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
    use wasm_bindgen::JsCast;

    let document = web_sys::window()
        .and_then(|window| window.document())
        .expect("window document should be available for CSR mount");
    let app_root = document
        .get_element_by_id("app")
        .expect("site app root element should exist")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("site app root should be an HtmlElement");
    leptos::mount_to(app_root, || leptos::view! { <SiteApp /> })
}
