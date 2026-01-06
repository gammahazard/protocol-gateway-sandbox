# Security Analysis

## Attack Surface

### What We Protect Against

| Attack Vector | Traditional Gateway | WASM Gateway |
|--------------|---------------------|--------------|
| **Buffer Overflow** | Process crash or RCE | Sandbox trap, 8ms restart |
| **Integer Overflow** | Undefined behavior | Rust panics, sandbox trap |
| **Format String** | Memory disclosure | Not possible (no printf) |
| **Heap Corruption** | Arbitrary write | Linear memory isolated |
| **Path Traversal** | File access | No filesystem capability |
| **Network Pivot** | Lateral movement | No network capability |

### The WASM Security Boundary

```
┌─────────────────────────────────────────────────────────────────┐
│                         UNTRUSTED                               │
│                                                                 │
│   ┌───────────────────────────────────────────────────────┐    │
│   │               WASM LINEAR MEMORY                       │    │
│   │                                                        │    │
│   │   • 32-bit address space (max 4GB)                     │    │
│   │   • No access to host memory                           │    │
│   │   • No syscalls                                        │    │
│   │   • No file handles                                    │    │
│   │   • No network sockets                                 │    │
│   │   • Can only call imported functions                   │    │
│   └───────────────────────────────────────────────────────┘    │
│                                                                 │
└────────────────────────────────────────┬────────────────────────┘
                                         │ WIT Interface
                                         │ (capability boundary)
┌────────────────────────────────────────▼────────────────────────┐
│                         TRUSTED                                 │
│                                                                 │
│   Host Runtime:                                                 │
│   • Provides modbus-source (read-only)                          │
│   • Provides mqtt-sink (publish only)                           │
│   • Controls what capabilities are granted                      │
│   • Survives guest crashes                                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Targeted Attack Vectors (Fuzz Testing)

We test against specific Modbus parser vulnerabilities:

### 1. Buffer Overflow Attack
```
Length header: 255 bytes
Actual payload: 2 bytes

Naive parser reads past buffer → crash
nom parser returns Incomplete → handled
```

### 2. Off-by-One Attack
```
Start address: 65535 (max u16)
Quantity: 125

Address + Quantity overflows u16 → undefined behavior
Rust checked arithmetic → panic → trap
```

### 3. Illegal Function Code
```
Function code: 0xFF (undefined)

Switch statement fallthrough → UB
Explicit rejection → error returned
```

### 4. Truncated Header
```
MBAP header: 7 bytes required
Actual: 3 bytes

struct.unpack crashes → process dies
nom returns Incomplete → error handled
```

### 5. Wrong Protocol ID
```
Protocol ID: 0xDEAD (should be 0x0000)

Unchecked → processes non-Modbus data
Validated → rejected at parse
```

## IEC 62443 Alignment

### Principle: Least Privilege

The WASM component has **zero** capabilities by default:
- ❌ Filesystem access
- ❌ Network access
- ❌ Process spawning
- ❌ Environment variables

It only has what the host explicitly provides:
- ✅ Read Modbus frames (via import)
- ✅ Publish MQTT payloads (via import)

### Principle: Defense in Depth

```
Layer 1: Rust type safety (compile time)
Layer 2: nom parser (never panics on input)
Layer 3: WASM sandbox (memory isolation)
Layer 4: Host crash recovery (availability)
```

### Principle: Attack Surface Minimization

Only 2 of 40+ Modbus function codes implemented:
- 0x03 Read Holding Registers
- 0x04 Read Input Registers

All write operations rejected. This is a **read-only data conduit**.

## Crash Recovery Metrics

| Metric | Target | Achieved |
|--------|--------|----------|
| Recovery time | <10ms | ~8ms |
| Host crash rate | 0% | 0% |
| Memory leak on crash | 0 bytes | 0 bytes (linear memory wiped) |
| State preserved | N/A | Fresh instance each time |

## Future Considerations

### What This Doesn't Protect Against

1. **Logic bugs in the parser** - If the parser returns wrong data, WASM won't catch that
2. **Side-channel attacks** - Timing attacks still possible
3. **Host vulnerabilities** - If the Node.js host has bugs, those are not sandboxed

### Recommended Hardening

1. **Cryptographic signing** - Verify WASM component before loading
2. **Resource limits** - Cap memory and CPU per instance
3. **Audit logging** - Log all crashes for forensic analysis
4. **Rate limiting** - Detect and block rapid crash attempts
