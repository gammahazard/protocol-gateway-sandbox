// guest/src/lib.rs
// main entry point for the protocol gateway wasm component.
// simplified version for initial build verification.

wit_bindgen::generate!({
    world: "protocol-gateway",
    path: "../wit",
});

mod modbus;
mod mqtt;

use modbus::{frame::MbapHeader, function::{FunctionCode, ReadResponse}};
use mqtt::payload::{TelemetryPayload, Register};

use std::cell::{Cell, RefCell};

// metrics storage
thread_local! {
    static FRAMES_PROCESSED: Cell<u64> = Cell::new(0);
    static FRAMES_INVALID: Cell<u64> = Cell::new(0);
    static BYTES_IN: Cell<u64> = Cell::new(0);
    static BYTES_OUT: Cell<u64> = Cell::new(0);
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

fn record_frame(size: u64) {
    FRAMES_PROCESSED.with(|f| f.set(f.get() + 1));
    BYTES_IN.with(|b| b.set(b.get() + size));
}

fn record_error(msg: String) {
    FRAMES_INVALID.with(|f| f.set(f.get() + 1));
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
}

fn record_outbound(size: u64) {
    BYTES_OUT.with(|b| b.set(b.get() + size));
}

struct Component;

export!(Component);

impl Guest for Component {
    fn run() {
        // receive frame from host
        let frame = match gateway::protocols::modbus_source::receive_frame() {
            Ok(data) => data,
            Err(e) => {
                record_error(format!("receive error: {}", e.message));
                return;
            }
        };
        
        let frame_size = frame.len() as u64;
        
        // parse mbap header
        let (remaining, header) = match MbapHeader::parse(&frame) {
            Ok(result) => result,
            Err(_) => {
                record_error("malformed mbap header".to_string());
                return;
            }
        };
        
        // validate header
        if let Err(msg) = header.validate() {
            record_error(msg.to_string());
            return;
        }
        
        // parse response
        let response = match ReadResponse::parse(remaining) {
            Ok((_, resp)) => resp,
            Err(_) => {
                record_error("malformed pdu".to_string());
                return;
            }
        };
        
        // build mqtt payload
        let payload = TelemetryPayload {
            source: "modbus://plc:502".to_string(),
            unit_id: header.unit_id,
            function: match response.function {
                FunctionCode::ReadHoldingRegisters => "read_holding_registers".to_string(),
                FunctionCode::ReadInputRegisters => "read_input_registers".to_string(),
            },
            registers: response.registers.iter().enumerate().map(|(i, &value)| {
                Register {
                    address: i as u16,
                    value,
                    label: None,
                }
            }).collect(),
            timestamp: "2026-01-05T00:00:00Z".to_string(),
        };
        
        let json = payload.to_json();
        let json_size = json.len() as u64;
        
        // publish
        let topic = format!("ics/telemetry/unit_{}", header.unit_id);
        if let Err(e) = gateway::protocols::mqtt_sink::publish(&topic, &json, 0) {
            record_error(format!("mqtt publish error: {}", e.message));
            return;
        }
        
        record_frame(frame_size);
        record_outbound(json_size);
    }
}

impl exports::gateway::protocols::metrics::Guest for Component {
    fn get_stats() -> exports::gateway::protocols::metrics::GatewayStats {
        exports::gateway::protocols::metrics::GatewayStats {
            frames_processed: FRAMES_PROCESSED.with(|f| f.get()),
            frames_invalid: FRAMES_INVALID.with(|f| f.get()),
            bytes_in: BYTES_IN.with(|b| b.get()),
            bytes_out: BYTES_OUT.with(|b| b.get()),
            last_error: LAST_ERROR.with(|e| e.borrow().clone()),
        }
    }
}
