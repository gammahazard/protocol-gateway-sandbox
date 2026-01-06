// guest/src/metrics_impl.rs
// tracks gateway performance and security stats for the dashboard.
// implements the wit-exported metrics::get-stats function.
// uses cell-based counters since wasm component instances are single-threaded.
// renamed from metrics.rs to avoid collision with wit-generated metrics module.

use std::cell::{Cell, RefCell};

// module-level storage for metrics
// wasm component model instances are single-threaded, so we use Cell instead of atomics
thread_local! {
    static FRAMES_PROCESSED: Cell<u64> = Cell::new(0);
    static FRAMES_INVALID: Cell<u64> = Cell::new(0);
    static BYTES_IN: Cell<u64> = Cell::new(0);
    static BYTES_OUT: Cell<u64> = Cell::new(0);
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

/// metrics tracking for the gateway
/// all methods are static since we use thread-local storage
pub struct MetricsTracker;

impl MetricsTracker {
    /// record a successfully processed frame
    /// called after parsing and publishing succeed
    pub fn record_frame(size: u64) {
        FRAMES_PROCESSED.with(|f| f.set(f.get() + 1));
        BYTES_IN.with(|b| b.set(b.get() + size));
    }

    /// record a parse or publish error
    /// called when frame is malformed or mqtt publish fails
    pub fn record_error(msg: String) {
        FRAMES_INVALID.with(|f| f.set(f.get() + 1));
        LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
    }
    
    /// record outbound mqtt payload size
    /// called after successful mqtt publish
    pub fn record_outbound(size: u64) {
        BYTES_OUT.with(|b| b.set(b.get() + size));
    }

    /// get current stats snapshot
    /// connects to the wit export 'metrics::get-stats'
    /// the host calls this to display live stats on the dashboard
    pub fn get_snapshot() -> crate::exports::gateway::protocols::metrics::GatewayStats {
        crate::exports::gateway::protocols::metrics::GatewayStats {
            frames_processed: FRAMES_PROCESSED.with(|f| f.get()),
            frames_invalid: FRAMES_INVALID.with(|f| f.get()),
            bytes_in: BYTES_IN.with(|b| b.get()),
            bytes_out: BYTES_OUT.with(|b| b.get()),
            last_error: LAST_ERROR.with(|e| e.borrow().clone()),
        }
    }
}
