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

| Metric | Cold Restart | 2oo3 TMR |
|--------|--------------|----------|
| Recovery time | ~8ms | **~0ms (2/3 still voting)** |
| Host crash rate | 0% | 0% |
| Memory leak on crash | 0 bytes | 0 bytes |
| Packets lost (1000/sec) | 1-2 | **0** |

### 2oo3 Triple Modular Redundancy

The host runtime maintains three WASM instances with voting. On fault:

1. **Zero switchover delay** - 2/3 instances still voting, result is valid
2. **Async rebuild** - faulty instance rebuilds in background (~7ms)
3. **Zero packet loss** - majority voting continues during rebuild

This implements SIL 3 voting patterns (IEC 61508) at the software layer.

## What Each Technology Solves (And Doesn't)

Understanding the **boundaries** of each technology is critical for defense-in-depth.

### Rust

| ✅ Solves | ❌ Doesn't Solve |
|-----------|-----------------|
| Buffer overflows | Logic bugs |
| Use-after-free | Algorithm errors |
| Data races | Business logic mistakes |
| Null pointer dereference | Incorrect calculations |

*Example: Rust prevents accessing invalid memory. It doesn't prevent returning the wrong register value.*

### WASM

| ✅ Solves | ❌ Doesn't Solve |
|-----------|-----------------|
| Memory isolation (sandbox) | Logic bugs |
| Trap instead of crash | Network-level security |
| No ambient syscall access | Side-channel attacks |
| Deterministic execution | Authentication |

*Example: WASM catches buffer overflows at runtime. It doesn't catch a parser that returns wrong data.*

### WASI

| ✅ Solves | ❌ Doesn't Solve |
|-----------|-----------------|
| Capability-based security | Network encryption |
| Deny-by-default permissions | User authentication |
| Explicit host control | Access control policies |
| No ambient authority | Audit logging |

*Example: WASI prevents the parser from opening `/etc/passwd`. It doesn't encrypt the Modbus traffic.*

### 2oo3 TMR Redundancy

| ✅ Solves | ❌ Doesn't Solve |
|-----------|-----------------|
| Software fault recovery (~0ms) | Network path failure |
| Parser crash containment | Hardware failure |
| Zero packet loss on trap | Power failure |

*Example: 2oo3 voting continues with 2/3 instances. If the NIC dies, redundancy can't help.*

## Complementary Technologies Still Needed

| Concern | WASM/WASI/Rust? | What You Need |
|---------|-----------------|---------------|
| Memory safety | ✅ | — |
| Sandbox isolation | ✅ | — |
| Capability control | ✅ | — |
| Software fault recovery | ✅ | — |
| Network encryption | ❌ | TLS, mTLS |
| Authentication | ❌ | OAuth, certificates |
| Network redundancy | ❌ | PRP/HSR (IEC 62439-3), dual NICs |
| Hardware redundancy | ❌ | Redundant servers, failover |
| Power redundancy | ❌ | UPS, generators |
| Logic correctness | ❌ | Unit tests, fuzzing, formal verification |

## Recommended Hardening

Beyond WASM sandboxing, consider:

1. **Cryptographic signing** — Verify WASM component before loading
2. **Resource limits** — Cap memory and CPU per instance
3. **Audit logging** — Log all crashes for forensic analysis
4. **Rate limiting** — Detect and block rapid crash attempts
5. **TLS everywhere** — Encrypt Modbus-over-TCP and MQTT
6. **mTLS** — Mutual authentication between gateway and broker
