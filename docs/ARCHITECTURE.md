# Architecture

## The Problem

Industrial Control Systems (ICS) face a billion-dollar challenge: **legacy protocol translation**. 

PLCs from the 1990s speak Modbus TCP, a binary protocol with no built-in security. To connect them to modern IT systems (cloud, SCADA historians, AI/ML platforms), you need a gateway that:

1. Parses the dangerous binary protocol
2. Transforms it to a modern format (JSON over MQTT)
3. Does this **without exposing the control loop to attackers**

Traditional gateways use Python or C parsers that crash when fed malformed input. A single buffer overflow can:
- Crash the gateway → loss of visibility
- Execute arbitrary code → lateral movement into OT
- Reach the PLC → safety-critical failure

## The Solution: WASM Sandboxing

We move the parser into a WASM sandbox. The security model is simple:

```
┌─────────────────────────────────────────────────────────────────┐
│                        HOST RUNTIME (Node.js)                   │
│  • Compiles WASM module once at startup                         │
│  • Creates new instance on each crash (<1ms)                    │
│  • Provides mock Modbus source and MQTT sink                    │
├─────────────────────────────────────────────────────────────────┤
│                        WASM SANDBOX (Rust)                      │
│  • Parses Modbus TCP frames with nom (fuzz-proof)               │
│  • Transforms to JSON payload                                   │
│  • Has NO filesystem or network access                          │
│  • If it crashes, only the sandbox dies                         │
└─────────────────────────────────────────────────────────────────┘
```

### Memory Safety "Twice"

1. **Language Level:** Rust provides compile-time memory safety
2. **Runtime Level:** WASM provides linear memory isolation

Even if the Rust parser has a logic bug, the attacker is trapped in a 32-bit linear memory space with no syscall access.

### Compile-Once, Instantiate-Many

The key optimization for fast recovery:

```javascript
// EXPENSIVE (50-100ms) - done once at startup
const compiledModule = await WebAssembly.compile(wasmBuffer);

// CHEAP (<1ms) - done on each crash
instance = await WebAssembly.instantiate(compiledModule);
```

This achieves **sub-millisecond restart times** instead of seconds.

## Component Model & WIT

We use the WASI 0.2 Component Model with WIT interfaces:

```wit
world protocol-gateway {
    import modbus-source;  // Host provides Modbus frames
    import mqtt-sink;      // Host accepts MQTT payloads
    
    export metrics;        // Guest provides stats
    export run: func();    // Guest processing loop
}
```

The guest component:
- **Cannot** access the filesystem
- **Cannot** open network connections
- **Can only** call the functions the host provides

This is **capability-based security** in action.

## Attack Surface Minimization (IEC 62443)

Per IEC 62443 principles, we minimize the attack surface:

| Modbus Function | Code | Implemented | Reason |
|----------------|------|-------------|--------|
| Read Holding Registers | 0x03 | ✅ Yes | Data conduit |
| Read Input Registers | 0x04 | ✅ Yes | Data conduit |
| Write Single Register | 0x06 | ❌ No | Attack surface |
| Write Multiple Registers | 0x10 | ❌ No | Attack surface |
| All others | * | ❌ No | Attack surface |

If someone asks "why only two function codes?", the answer is:

> "Per IEC 62443, we minimize attack surface by only implementing the minimum required for the data conduit."

## Crash Recovery Flow

```
1. Malformed packet arrives
2. nom parser returns Err (doesn't panic)
3. If panic occurs, WASM traps
4. Host catches trap
5. Host creates new instance from cached module (<1ms)
6. Gateway continues processing
7. PLC never noticed
```

## Comparison: Python vs WASM

| Aspect | Python Gateway | WASM Gateway |
|--------|---------------|--------------|
| Parser | `struct.unpack()` | `nom` combinators |
| Bounds checking | Manual (often missing) | Built into parser |
| Crash behavior | Process exits | Sandbox traps |
| Recovery | Manual restart (60s) | Automatic (8ms) |
| Attack blast radius | Entire gateway | Single request |
| Memory corruption | Heap corruption possible | Linear memory isolated |
