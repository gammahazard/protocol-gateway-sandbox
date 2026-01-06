// guest/src/lib.rs
// main entry point for the protocol gateway wasm component.
// this file wires together the modbus parser, mqtt payload builder,
// and metrics tracking. it implements the wit-exported run() function.

// generate bindings from wit interface definitions
wit_bindgen::generate!({
    world: "protocol-gateway",
    path: "../wit",
});

mod modbus;
mod mqtt;
mod metrics;

use modbus::{frame::MbapHeader, function::{FunctionCode, ReadResponse}};
use mqtt::payload::{TelemetryPayload, Register};
use metrics::Metrics;

// import the wit-generated bindings for host capabilities
use crate::bindings::gateway::protocols::{modbus_source, mqtt_sink};

/// implements the wit-exported run function
/// this is called by the host runtime in a loop
struct GatewayComponent;

impl Guest for GatewayComponent {
    fn run() {
        // receive frame from host (mock or real modbus source)
        let frame = match modbus_source::receive_frame() {
            Ok(data) => data,
            Err(e) => {
                Metrics::record_error(format!("receive error: {}", e.message));
                return;
            }
        };
        
        let frame_size = frame.len() as u64;
        
        // parse mbap header using nom (fuzz-proof)
        let (remaining, header) = match MbapHeader::parse(&frame) {
            Ok(result) => result,
            Err(_) => {
                Metrics::record_error("malformed mbap header".to_string());
                return;
            }
        };
        
        // validate header fields
        if let Err(msg) = header.validate() {
            Metrics::record_error(msg.to_string());
            return;
        }
        
        // parse function code and response data
        let response = match ReadResponse::parse(remaining) {
            Ok((_, resp)) => resp,
            Err(_) => {
                Metrics::record_error("malformed pdu".to_string());
                return;
            }
        };
        
        // build mqtt payload
        let payload = TelemetryPayload {
            source: format!("modbus://plc:502"),
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
            timestamp: "2026-01-05T00:00:00Z".to_string(), // placeholder
        };
        
        let json = payload.to_json();
        let json_size = json.len() as u64;
        
        // publish to mqtt sink
        let topic = format!("ics/telemetry/unit_{}", header.unit_id);
        if let Err(e) = mqtt_sink::publish(&topic, &json, 0) {
            Metrics::record_error(format!("mqtt publish error: {}", e.message));
            return;
        }
        
        // record successful processing
        Metrics::record_frame(frame_size);
        Metrics::record_outbound(json_size);
    }
}

// export the metrics interface implementation
impl crate::bindings::exports::gateway::protocols::metrics::Guest for GatewayComponent {
    fn get_stats() -> crate::bindings::exports::gateway::protocols::metrics::GatewayStats {
        Metrics::get_snapshot()
    }
}

export!(GatewayComponent);
