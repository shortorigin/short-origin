//! Time helpers shared across host contracts and adapters.

use std::cell::Cell;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

thread_local! {
    static LAST_ENVELOPE_TIMESTAMP_MS: Cell<u64> = const { Cell::new(0) };
}

/// Returns the current unix timestamp in milliseconds.
pub fn unix_time_ms_now() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now().max(0.0) as u64
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// Returns a monotonic unix millisecond timestamp for envelope updates.
///
/// Values are monotonic within the current process even when the system clock does not advance.
pub fn next_monotonic_timestamp_ms() -> u64 {
    let now = unix_time_ms_now();
    LAST_ENVELOPE_TIMESTAMP_MS.with(|last| {
        let next = now.max(last.get().saturating_add(1));
        last.set(next);
        next
    })
}
