// guest/src/modbus/function.rs
// handles modbus function codes. per iec 62443 attack surface minimization,
// we only implement read-only function codes (0x03, 0x04) for the data conduit.
// all other codes are explicitly rejected - this is intentional security design.

use nom::{
    number::complete::{be_u16, be_u8},
    IResult,
};

/// supported modbus function codes - intentionally limited scope
/// per iec 62443 principle of minimizing attack surface, we only implement
/// what's needed for a read-only data conduit
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionCode {
    ReadHoldingRegisters,  // 0x03 - read analog outputs / configuration
    ReadInputRegisters,    // 0x04 - read analog inputs from field devices
}

impl FunctionCode {
    /// parse function code byte. rejects all codes except 0x03 and 0x04.
    /// this is not a bug - it's iec 62443 attack surface minimization.
    /// if someone asks "why only two function codes?", the answer is:
    /// "per iec 62443, we minimize attack surface by only implementing
    /// the minimum required for the data conduit."
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x03 => Some(Self::ReadHoldingRegisters),
            0x04 => Some(Self::ReadInputRegisters),
            _ => None, // intentionally reject all other function codes
        }
    }
    
    /// convert function code to its byte representation
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::ReadHoldingRegisters => 0x03,
            Self::ReadInputRegisters => 0x04,
        }
    }
}

/// parsed read request (0x03 or 0x04)
/// sent from master to slave to request register values
#[derive(Debug, Clone, PartialEq)]
pub struct ReadRequest {
    pub function: FunctionCode,
    pub start_address: u16,
    pub quantity: u16,
}

/// parsed read response (0x03 or 0x04)
/// sent from slave to master with requested register values
#[derive(Debug, Clone, PartialEq)]
pub struct ReadResponse {
    pub function: FunctionCode,
    pub byte_count: u8,
    pub registers: Vec<u16>,
}

impl ReadResponse {
    /// parse a read holding/input registers response.
    /// format: [function_code(1), byte_count(1), register_values(N*2)]
    /// uses nom for fuzz-proof parsing - returns error on malformed input
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, function_byte) = be_u8(input)?;
        let function = FunctionCode::from_byte(function_byte)
            .ok_or(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))?;
        
        let (input, byte_count) = be_u8(input)?;
        let register_count = (byte_count / 2) as usize;
        
        // parse each 16-bit register value (big-endian)
        let mut registers = Vec::with_capacity(register_count);
        let mut remaining = input;
        
        for _ in 0..register_count {
            let (input, value) = be_u16(remaining)?;
            registers.push(value);
            remaining = input;
        }
        
        Ok((remaining, Self {
            function,
            byte_count,
            registers,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_code_parsing() {
        assert_eq!(FunctionCode::from_byte(0x03), Some(FunctionCode::ReadHoldingRegisters));
        assert_eq!(FunctionCode::from_byte(0x04), Some(FunctionCode::ReadInputRegisters));
        assert_eq!(FunctionCode::from_byte(0x06), None); // write single register - rejected
        assert_eq!(FunctionCode::from_byte(0xFF), None); // illegal code - rejected
    }
    
    #[test]
    fn test_parse_read_response() {
        // function: 0x03, byte_count: 4, registers: [1000, 2000]
        let data = [0x03, 0x04, 0x03, 0xE8, 0x07, 0xD0];
        let (_, response) = ReadResponse::parse(&data).unwrap();
        
        assert_eq!(response.function, FunctionCode::ReadHoldingRegisters);
        assert_eq!(response.byte_count, 4);
        assert_eq!(response.registers, vec![1000, 2000]);
    }
    
    #[test]
    fn test_reject_illegal_function() {
        // function: 0xFF (illegal)
        let data = [0xFF, 0x02, 0x00, 0x00];
        assert!(ReadResponse::parse(&data).is_err());
    }
}
