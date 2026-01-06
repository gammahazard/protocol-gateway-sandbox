// guest/src/modbus/frame.rs
// parses the modbus application protocol (mbap) header from raw tcp bytes.
// uses nom combinators for fuzz-proof parsing - returns Err on malformed input,
// never panics, which is critical for wasm sandbox crash containment.

use nom::{
    number::complete::{be_u16, be_u8},
    IResult,
};

/// mbap header - 7 bytes that wrap every modbus tcp message
/// this is the tcp-specific wrapper, not part of the serial modbus protocol
#[derive(Debug, Clone, PartialEq)]
pub struct MbapHeader {
    pub transaction_id: u16,  // echoed by server for request/response matching
    pub protocol_id: u16,     // must be 0x0000 for modbus
    pub length: u16,          // remaining bytes (unit_id + pdu)
    pub unit_id: u8,          // slave address (usually 0x01 or 0xFF)
}

impl MbapHeader {
    /// parse mbap header from raw bytes using nom combinators.
    /// returns (remaining_bytes, header) on success, or nom error on failure.
    /// this design means malformed input never panics - it returns an error
    /// that the wasm guest can handle gracefully.
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, transaction_id) = be_u16(input)?;
        let (input, protocol_id) = be_u16(input)?;
        let (input, length) = be_u16(input)?;
        let (input, unit_id) = be_u8(input)?;
        
        Ok((input, Self {
            transaction_id,
            protocol_id,
            length,
            unit_id,
        }))
    }
    
    /// validate the header after parsing.
    /// checks protocol id and length field are within modbus spec bounds.
    pub fn validate(&self) -> Result<(), &'static str> {
        // protocol id must be 0x0000 for modbus
        if self.protocol_id != 0x0000 {
            return Err("invalid protocol id - must be 0x0000 for modbus");
        }
        // length must be at least 2 (unit_id + function code) and at most 253
        if self.length < 2 || self.length > 253 {
            return Err("invalid length field - must be 2-253");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_header() {
        // transaction: 0x0001, protocol: 0x0000, length: 0x0006, unit: 0x01
        let data = [0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0x00, 0x00];
        let (remaining, header) = MbapHeader::parse(&data).unwrap();
        
        assert_eq!(header.transaction_id, 1);
        assert_eq!(header.protocol_id, 0x0000);
        assert_eq!(header.length, 6);
        assert_eq!(header.unit_id, 1);
        assert_eq!(remaining, &[0x03, 0x00, 0x00]); // pdu follows
    }
    
    #[test]
    fn test_parse_truncated_header() {
        // only 3 bytes - not enough for 7-byte header
        let data = [0x00, 0x01, 0x00];
        assert!(MbapHeader::parse(&data).is_err());
    }
    
    #[test]
    fn test_validate_wrong_protocol() {
        let header = MbapHeader {
            transaction_id: 1,
            protocol_id: 0xDEAD, // not modbus!
            length: 6,
            unit_id: 1,
        };
        assert!(header.validate().is_err());
    }
}
