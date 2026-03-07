#[cfg(target_arch = "wasm32")]
pub fn try_set_pointer_capture(ev: &web_sys::PointerEvent) {
    use wasm_bindgen::JsCast;

    if let Some(target) = ev.current_target() {
        if let Ok(element) = target.dyn_into::<web_sys::Element>() {
            let _ = element.set_pointer_capture(ev.pointer_id());
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn try_set_pointer_capture(_: &web_sys::PointerEvent) {}
