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

## 2oo3 Triple Modular Redundancy (TMR)

### The Question

> "Does WASM's fast restart (~8ms) eliminate the need for redundancy?"

**Answer:** No — but we go beyond simple redundancy with **voting**.

### 2oo3 Architecture

We apply SIL 3 safety patterns (IEC 61508) at the software layer:

```
┌────────────────────────────────────────────────────────────────────────┐
│                         2oo3 INSTANCE POOL                             │
│   ┌───────────┐     ┌───────────┐     ┌───────────┐                   │
│   │ INSTANCE 0│     │ INSTANCE 1│     │ INSTANCE 2│                   │
│   │    ✓      │     │    ✓      │     │    ✗      │                   │
│   └─────┬─────┘     └─────┬─────┘     └─────┬─────┘                   │
│         │                 │                 │                          │
│         └────────┬────────┴────────┬────────┘                          │
│                  │     VOTER       │                                   │
│                  │  2/3 agree ✓    │                                   │
│                  └────────┬────────┘                                   │
│                           ▼                                            │
│                     Result: OK                                         │
│                     Faulty: Instance 2 (rebuild async)                 │
└────────────────────────────────────────────────────────────────────────┘
```

### Why 2oo3 Over 1oo2

| Pattern | Instances | Fault Detection | Use Case |
|---------|-----------|-----------------|----------|
| **1oo2 (Hot-Standby)** | 2 | Crash only | General availability |
| **2oo3 (Voting/TMR)** | 3 | Crash + wrong result | Safety-critical |

**Key advantage:** 2oo3 can detect *which* instance produced a wrong result, not just that one crashed. This matters for:
- Byzantine faults (instance returns wrong data instead of crashing)
- Transient errors (radiation, memory bit-flips in industrial environments)

### Industrial Redundancy Patterns

| SIL Level | Pattern | Used For | Our Implementation |
|-----------|---------|----------|-------------------|
| SIL 1 | 1oo1 | Monitoring | — |
| SIL 2 | 1oo2D | Process control | — |
| **SIL 3** | **2oo3** | **Emergency shutdown** | **WASM 2oo3 voting** |
| SIL 4 | 2oo4 | Nuclear, aviation | — |

### Comparison with Traditional Solutions

| Solution | Fault Detection | Rebuild Time | Memory |
|----------|----------------|--------------|--------|
| **Python multiprocessing** | Crash only | ~500ms | ~30-50 MB/worker |
| **Container restart** | Crash only | ~2-5s | Container overhead |
| **PLC 2oo3 (Triconex)** | Voting | Hardware | Expensive |
| **WASM 2oo3** | **Voting** | **~8ms** | **~2 MB/instance** |

### Voting Outcomes

| Agreement | Action | Example |
|-----------|--------|---------|
| 3/3 (unanimous) | Use result, all healthy | Normal operation |
| 2/3 (majority) | Use majority, rebuild faulty | Transient fault detected |
| 0/3 (split) | Reject, critical error | All instances corrupted |

### IEC 61508 Alignment

| IEC 61508 Principle | Our Implementation |
|---------------------|-------------------|
| Triple redundancy | 3 WASM instances |
| Majority voting | 2/3 must agree |
| Fault isolation | Per-instance sandboxing |
| Async recovery | Faulty instance rebuilds in background |


## Why WASM Over Traditional Industrial Solutions

### The Trade-off Matrix

| Solution | Fault Type | Switchover | Memory | Cost |
|----------|-----------|------------|--------|------|
| **PLC Hardware Redundancy** | Hardware | ~10-50μs | N/A | 💰💰💰 (2x hardware) |
| **PRP/HSR (IEC 62439-3)** | Network | ~50μs | Network duplication | 💰💰 |
| **Python Multiprocessing** | Software | ~5ms | 30-50MB/worker | 💰 |
| **Docker Restart** | Software | ~500ms-2s | Container overhead | 💰 |
| **WASM Hot-Standby** | Software | **~100μs** | **~2MB/instance** | **💰** |

### Why WASM Wins for Software Faults

1. **Same-Process Isolation**
   - Both WASM instances live in the same Node.js process
   - No IPC overhead, no serialization, no context switching
   - Switchover is literally changing an index variable

2. **Memory Efficiency**
   - WASM linear memory: ~1-2MB per instance
   - Python runtime: ~30-50MB per process
   - For a 2-instance pool: 4MB vs 100MB

3. **True Sandboxing Without OS Overhead**
   - Containers isolate at OS level (slow)
   - WASM isolates at language level (fast)
   - Trap handling is part of the WASM spec, not OS signals

4. **Deterministic Recovery**
   - `WebAssembly.instantiate()` is predictable (~8ms)
   - Process restart depends on OS, init system, etc.

### What WASM Doesn't Replace

WASM hot-standby is for **software fault tolerance**, not:
- ❌ Network path redundancy (still need PRP/HSR for that)
- ❌ Hardware failure (still need redundant PLCs)
- ❌ Power failure (still need UPS)

It's a **complementary layer** in defense-in-depth, not a replacement.
