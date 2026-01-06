// guest/src/modbus/mod.rs
// modbus protocol parsing module.
// contains frame parsing (mbap header) and function code handlers.
// uses nom for fuzz-proof parsing - malformed input returns errors, never panics.

pub mod frame;
pub mod function;
