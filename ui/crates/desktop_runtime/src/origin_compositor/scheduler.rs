#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
pub fn request_animation_frame(callback: &wasm_bindgen::closure::Closure<dyn FnMut()>) {
    if let Some(window) = web_sys::window() {
        let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn request_animation_frame(_: &()) {}
