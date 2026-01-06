// guest/src/mqtt/payload.rs
// transforms parsed modbus data into json payloads for mqtt publishing.
// uses serde for serialization - the output format is designed for
// consumption by scada historians and cloud analytics platforms.

use serde::Serialize;

/// telemetry payload published to mqtt
/// this is the json structure that downstream systems will receive
#[derive(Serialize, Debug)]
pub struct TelemetryPayload {
    pub source: String,           // e.g., "modbus://10.0.0.50:502"
    pub unit_id: u8,              // modbus slave address
    pub function: String,         // "read_holding_registers" or "read_input_registers"
    pub registers: Vec<Register>, // parsed register values
    pub timestamp: String,        // iso 8601 format
}

/// individual register value with optional human-readable label
#[derive(Serialize, Debug)]
pub struct Register {
    pub address: u16,             // register address (0-65535)
    pub value: u16,               // raw 16-bit value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,    // human-readable name if configured
}

impl TelemetryPayload {
    /// serialize to json string for mqtt publishing
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_payload_serialization() {
        let payload = TelemetryPayload {
            source: "modbus://10.0.0.50:502".to_string(),
            unit_id: 1,
            function: "read_holding_registers".to_string(),
            registers: vec![
                Register { address: 0, value: 1000, label: Some("temperature".to_string()) },
                Register { address: 1, value: 2000, label: None },
            ],
            timestamp: "2026-01-05T00:00:00Z".to_string(),
        };
        
        let json = payload.to_json();
        assert!(json.contains("modbus://10.0.0.50:502"));
        assert!(json.contains("temperature"));
        assert!(json.contains("1000"));
    }
}
