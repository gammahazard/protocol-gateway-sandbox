// host/shim/modbus-source.js
// mock modbus tcp source that simulates plc responses.
// provides the modbus-source interface to the wasm guest.
// can inject chaos packets for fuzz testing via setChaosMode().

// queue of frames to be returned by receive-frame
let frameQueue = [];
let chaosMode = false;

// import chaos attack vectors for testing
import { CHAOS_ATTACKS } from './chaos-attacks.js';

/**
 * add a valid modbus frame to the queue
 * @param {Uint8Array} frame - raw modbus tcp frame (mbap + pdu)
 */
export function queueFrame(frame) {
    frameQueue.push(frame);
}

/**
 * enable or disable chaos mode
 * when enabled, injects random malformed packets
 */
export function setChaosMode(enabled) {
    chaosMode = enabled;
}

/**
 * build a valid modbus read holding registers response
 * @param {number} unitId - slave address
 * @param {number} transactionId - transaction id for matching
 * @param {number[]} registers - array of 16-bit register values
 */
export function buildReadResponse(unitId, transactionId, registers) {
    const byteCount = registers.length * 2;
    const length = 3 + byteCount; // unit_id + function + byte_count + data
    
    const frame = new Uint8Array(7 + 2 + byteCount);
    const view = new DataView(frame.buffer);
    
    // mbap header
    view.setUint16(0, transactionId);  // transaction id
    view.setUint16(2, 0x0000);         // protocol id (modbus)
    view.setUint16(4, length);         // length
    frame[6] = unitId;                 // unit id
    
    // pdu - read holding registers response
    frame[7] = 0x03;                   // function code
    frame[8] = byteCount;              // byte count
    
    // register values (big-endian)
    for (let i = 0; i < registers.length; i++) {
        view.setUint16(9 + i * 2, registers[i]);
    }
    
    return frame;
}

/**
 * wit interface implementation: receive-frame
 * returns the next frame from the queue, or a chaos packet if enabled
 */
export function receiveFrame() {
    // if chaos mode and random chance, inject malformed packet
    if (chaosMode && Math.random() < 0.3) {
        const attackNames = Object.keys(CHAOS_ATTACKS);
        const randomAttack = attackNames[Math.floor(Math.random() * attackNames.length)];
        const malformedFrame = CHAOS_ATTACKS[randomAttack]();
        console.log(`[MODBUS-SOURCE] Injecting chaos: ${randomAttack}`);
        return { tag: 'ok', val: Array.from(malformedFrame) };
    }
    
    // return queued frame or generate a sample one
    if (frameQueue.length > 0) {
        const frame = frameQueue.shift();
        return { tag: 'ok', val: Array.from(frame) };
    }
    
    // no frames queued - generate sample data
    const sampleFrame = buildReadResponse(1, 1, [1000, 2000, 3000, 4000, 5000]);
    return { tag: 'ok', val: Array.from(sampleFrame) };
}
