// host/shim/modbus-source.js
// mock modbus tcp source that simulates plc responses.
// provides the modbus-source interface to the wasm guest.

import crypto from 'crypto';

// queue of frames to be returned 
let frameQueue = [];
let chaosMode = false;

// chaos attack vectors
const CHAOS_ATTACKS = {
    bufferOverflow: () => new Uint8Array([
        0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0x01, 0x03,
    ]),
    offByOne: () => new Uint8Array([
        0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01,
        0x03, 0xFF, 0xFF, 0x00, 0x7D,
    ]),
    illegalFunction: () => new Uint8Array([
        0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x01, 0xFF,
    ]),
    truncatedHeader: () => new Uint8Array([0x00, 0x01, 0x00]),
    wrongProtocol: () => new Uint8Array([
        0x00, 0x01, 0xDE, 0xAD, 0x00, 0x06, 0x01, 0x03, 0x00, 0x00, 0x00, 0x0A,
    ]),
    randomGarbage: () => new Uint8Array(crypto.randomBytes(Math.floor(Math.random() * 300))),
};

export function queueFrame(frame) {
    frameQueue.push(frame);
}

export function setChaosMode(enabled) {
    chaosMode = enabled;
}

/**
 * build a valid modbus read holding registers response
 */
export function buildReadResponse(unitId, transactionId, registers) {
    const byteCount = registers.length * 2;
    const length = 1 + 1 + 1 + byteCount;

    const totalLength = 6 + length;
    const frame = new Uint8Array(totalLength);

    // MBAP header
    frame[0] = (transactionId >> 8) & 0xFF;
    frame[1] = transactionId & 0xFF;
    frame[2] = 0x00;
    frame[3] = 0x00;
    frame[4] = (length >> 8) & 0xFF;
    frame[5] = length & 0xFF;
    frame[6] = unitId;

    // PDU
    frame[7] = 0x03;
    frame[8] = byteCount;

    for (let i = 0; i < registers.length; i++) {
        frame[9 + i * 2] = (registers[i] >> 8) & 0xFF;
        frame[9 + i * 2 + 1] = registers[i] & 0xFF;
    }

    return frame;
}

/**
 * jco expects this to return Uint8Array directly, not a Result wrapper
 * errors should be thrown as exceptions
 */
export function receiveFrame() {
    if (chaosMode && Math.random() < 0.3) {
        const attackNames = Object.keys(CHAOS_ATTACKS);
        const randomAttack = attackNames[Math.floor(Math.random() * attackNames.length)];
        const malformedFrame = CHAOS_ATTACKS[randomAttack]();
        console.log(`[MODBUS-SOURCE] Injecting chaos: ${randomAttack}`);
        return malformedFrame;
    }

    if (frameQueue.length > 0) {
        const frame = frameQueue.shift();
        return frame instanceof Uint8Array ? frame : new Uint8Array(frame);
    }

    // generate sample response
    return buildReadResponse(1, 1, [1000, 2000, 3000, 4000, 5000]);
}
