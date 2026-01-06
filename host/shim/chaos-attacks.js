// host/shim/chaos-attacks.js
// shared library of targeted attack vectors for fuzz testing.
// used by both the test suite and the dashboard chaos button.
// each attack exploits a specific modbus parser vulnerability.

import crypto from 'crypto';

/**
 * targeted attack vectors that exploit common modbus parser vulnerabilities.
 * each function returns a malformed frame designed to crash naive parsers.
 */
export const CHAOS_ATTACKS = {
    /**
     * buffer overflow attack
     * length header claims 255 bytes, but only 2 bytes follow.
     * naive parsers using struct.unpack without bounds checking will crash.
     */
    bufferOverflow: () => Buffer.from([
        0x00, 0x01,  // transaction id
        0x00, 0x00,  // protocol id
        0x00, 0xFF,  // length: 255 (LIE!)
        0x01, 0x03,  // only 2 bytes of payload
    ]),

    /**
     * off-by-one attack
     * requests register 65535 (max u16) with quantity that would overflow.
     * tests for integer overflow vulnerabilities in address calculations.
     */
    offByOne: () => Buffer.from([
        0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01,
        0x03,        // read holding registers
        0xFF, 0xFF,  // start address: 65535
        0x00, 0x7D,  // quantity: 125 (would overflow past 65535)
    ]),

    /**
     * illegal function code attack
     * sends function code 0xFF which is undefined in modbus spec.
     * tests that parser properly rejects unknown function codes.
     */
    illegalFunction: () => Buffer.from([
        0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x01,
        0xFF,        // illegal function code!
    ]),

    /**
     * truncated header attack
     * sends only 3 bytes when 7 are expected for mbap header.
     * tests bounds checking on header parsing.
     */
    truncatedHeader: () => Buffer.from([0x00, 0x01, 0x00]),

    /**
     * wrong protocol id attack
     * sends protocol id 0xDEAD instead of 0x0000.
     * tests that parser validates protocol field.
     */
    wrongProtocol: () => Buffer.from([
        0x00, 0x01,
        0xDE, 0xAD,  // protocol id: not 0x0000!
        0x00, 0x06, 0x01, 0x03, 0x00, 0x00, 0x00, 0x0A,
    ]),

    /**
     * zero length attack
     * sends length field of 0, which is invalid (min is 2).
     * tests length validation.
     */
    zeroLength: () => Buffer.from([
        0x00, 0x01, 0x00, 0x00,
        0x00, 0x00,  // length: 0 (invalid!)
        0x01,
    ]),

    /**
     * massive length attack  
     * claims to have max possible length (65535 bytes).
     * tests for memory allocation attacks.
     */
    massiveLength: () => Buffer.from([
        0x00, 0x01, 0x00, 0x00,
        0xFF, 0xFF,  // length: 65535
        0x01, 0x03,
    ]),

    /**
     * random garbage attack
     * generates completely random bytes of random length.
     * the ultimate chaos test.
     */
    randomGarbage: () => crypto.randomBytes(Math.floor(Math.random() * 300)),
};

/**
 * get a random attack from the collection
 * @returns {{name: string, frame: Buffer}} attack name and frame
 */
export function getRandomAttack() {
    const names = Object.keys(CHAOS_ATTACKS);
    const name = names[Math.floor(Math.random() * names.length)];
    return { name, frame: CHAOS_ATTACKS[name]() };
}

/**
 * get all attack names
 * @returns {string[]} array of attack names
 */
export function getAttackNames() {
    return Object.keys(CHAOS_ATTACKS);
}
