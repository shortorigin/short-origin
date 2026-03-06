//! Binary entrypoint for the browser-hosted `site` application.

#[cfg(all(target_arch = "wasm32", feature = "csr"))]
fn main() {
    site::mount();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!(
        "This binary is intended for the browser/WASM workflow. Use `cargo ui-dev` for preview builds or build `site_app` for wasm32 with the `csr` feature."
    );
}
