//! The time source of the engine (`timer()` and the random seed).
//!
//! The core has no I/O; on native targets it reads `std::time`, which is not available on
//! `wasm32-unknown-unknown`. A host (the wasm crate) installs its own clock with
//! [`set_clock`].
use std::cell::Cell;

thread_local! {
    static CLOCK: Cell<Option<fn() -> f64>> = const { Cell::new(None) };
}

/// Installs a clock returning milliseconds since the Unix epoch.
pub fn set_clock(f: fn() -> f64) {
    CLOCK.with(|c| c.set(Some(f)));
}

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> f64 {
    if let Some(f) = CLOCK.with(|c| c.get()) {
        return f();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        0.0
    }
}
