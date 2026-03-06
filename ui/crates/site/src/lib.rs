//! Web entrypoints for the Short Origin OS shell site.
//!
//! This crate exposes the root Leptos components and the browser mount helper used by the
//! `site_app` binary.
//!
//! In normal local development, start the browser shell with `cargo ui-dev`; that workflow
//! delegates to the same Trunk/WASM pipeline this crate expects at runtime.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

mod web_app;

pub use web_app::{DesktopEntry, SiteApp};

#[cfg(all(feature = "csr", target_arch = "wasm32"))]
/// Mounts [`SiteApp`] into the document body for client-side rendering.
///
/// This is the browser entrypoint used by the `site_app` binary when built for `wasm32` with the
/// `csr` feature enabled.
pub fn mount() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| leptos::view! { <SiteApp /> })
}
