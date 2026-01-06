<p align="center">
  <img src="https://img.shields.io/badge/WASI-0.2%20Preview%202-blueviolet?style=for-the-badge&logo=webassembly" alt="WASI 0.2"/>
  <img src="https://img.shields.io/badge/Rust-1.75+-orange?style=for-the-badge&logo=rust" alt="Rust"/>
  <img src="https://img.shields.io/badge/Modbus-TCP-blue?style=for-the-badge" alt="Modbus TCP"/>
  <img src="https://img.shields.io/badge/Arch-Cross--Platform_(ARM64%2Fx64)-lightgrey?style=for-the-badge&logo=cpu" alt="Cross Platform"/>
</p>

<h1 align="center">🔒 Protocol Gateway Sandbox</h1>

<p align="center">
  <strong>Safe Legacy Protocol Translation via WASM Sandboxing</strong><br/>
  <em>"How do I connect my 1990s PLC to the cloud without letting hackers into the control loop?"</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-completed-brightgreen" alt="Status"/>
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License"/>
</p>

---

## 🎯 The Security Thesis

**Without WASM:** A buffer overflow in the Modbus parser crashes/owns the gateway, potentially reaching the PLC.

**With WASM:** A buffer overflow in the Modbus parser crashes the WASM instance. The host restarts it in **<10ms**. The PLC never sees the attack.

```
┌────────────────────────────────────────────────────────────────────────────────┐
│                         PROTOCOL GATEWAY SANDBOX                                │
├────────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│   ┌─────────────────┐     ┌──────────────────────────┐     ┌────────────────┐ │
│   │  LEGACY OT      │     │    WASM SANDBOX          │     │   MODERN IT    │ │
│   │  (Modbus TCP)   │     │    (The Parser)          │     │   (MQTT)       │ │
│   │                 │     │                          │     │                │ │
│   │  PLC/RTU        │────▶│  Binary Parser (Rust)    │────▶│  MQTT Broker   │ │
│   │  10.0.0.50:502  │     │  • Decode Modbus PDU     │     │  Cloud/SCADA   │ │
│   │                 │     │  • Validate registers    │     │                │ │
│   │  Function codes:│     │  • Transform to JSON     │     │  Topics:       │ │
│   │  0x03, 0x04     │     │  • Encode to MQTT        │     │  ics/telemetry │ │
│   └─────────────────┘     └──────────────────────────┘     └────────────────┘ │
│                                       │                                        │
│                                       │ ☠️ ATTACK SURFACE                      │
│                                       │ Malformed Modbus = crash WASM, not PLC │
│                                       │                                        │
└────────────────────────────────────────────────────────────────────────────────┘
```

## 🏗️ Architecture

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Modbus Parser** | Rust → WASM | Memory-safe parsing of dangerous binary protocol |
| **Host Runtime** | JavaScript (Node.js) | WASM loader with 2oo3 TMR voting (SIL 3 pattern) |
| **Mock Sources** | JS Shims | Simulated PLC and MQTT broker |
| **Dashboard** | Leptos → WASM | Real-time security console with real WASM measurements |

### IEC 62443 Alignment

Per IEC 62443 attack surface minimization, we implement only:
- `0x03` Read Holding Registers
- `0x04` Read Input Registers

All other function codes are **explicitly rejected**. This is intentional.

### Attack Vectors Tested

| Attack | Description |
|--------|-------------|
| **Buffer Overflow** | "Length Lie" - header claims more bytes than sent |
| **Truncated Header** | Incomplete MBAP header (< 7 bytes) |
| **Illegal Function** | Unsupported codes like `0xFF` |
| **Random Garbage** | Non-Modbus binary noise |

## 📁 Project Structure

```
protocol-gateway-sandbox/
├── wit/                    # WIT interface definitions
│   └── world.wit           # modbus-source, mqtt-sink, metrics
├── guest/                  # Rust WASM component
│   └── src/
│       ├── lib.rs          # Main entry (run function)
│       ├── modbus/         # Protocol parser
│       │   ├── frame.rs    # MBAP header parsing (nom)
│       │   └── function.rs # Function code handlers
│       ├── mqtt/           # Payload builder
│       │   └── payload.rs  # JSON serialization
│       └── metrics_impl.rs # Gateway stats
├── host/                   # JavaScript runtime
│   ├── runtime.js          # **Hot-standby pool + crash recovery**
│   ├── shim/
│   │   ├── modbus-source.js
│   │   ├── mqtt-sink.js
│   │   └── chaos-attacks.js
│   └── test/
│       └── fuzz.test.js    # Security invariant tests
├── legacy/                 # Python "villain" comparison
│   └── vulnerable_gateway.py
├── dashboard/              # Leptos web UI
│   ├── src/lib.rs          # **Redundancy visualization**
│   └── styles.css          # Security console dark theme
└── docs/
    ├── [ARCHITECTURE.md](docs/ARCHITECTURE.md)     # Hot-standby pattern
    └── [SECURITY.md](docs/SECURITY.md)             # IEC 62443 alignment
```

## 🖥️ Dashboard Demo

The dashboard shows **two live terminals side-by-side**:

| Python Terminal | WASM Terminal |
|-----------------|---------------|
| Shows startup, then 💥 CRASH | Shows startup, ⚠️ warning, ✅ continues |
| 60-second restart countdown | Recovers in ~5ms, keeps processing |
| Connection to PLC lost | No impact on operations |

Run locally:
```bash
cd dashboard && trunk serve
# Open http://localhost:8080
```

## 🚀 Quick Start

### Prerequisites

```bash
cargo install cargo-component
npm install -g @bytecodealliance/jco
```

### Build & Run

```bash
# Build the WASM component
cd guest && cargo component build --release

# Transpile for Node.js
cd ../host && npx jco transpile ../guest/target/wasm32-wasi/release/*.wasm -o .

# Run the demo
npm run demo

# Run fuzz tests
npm test
```

## 🧪 The "Villain" Comparison

See [`legacy/vulnerable_gateway.py`](legacy/vulnerable_gateway.py) - a realistic Python gateway using `struct.unpack` without bounds checking.

Run both side-by-side to see the difference:

**Terminal 1 (Python - crashes):**
```bash
cd legacy && python vulnerable_gateway.py
# Sends malformed packet → 💥 PROCESS DIES
```

**Terminal 2 (WASM - survives):**
```bash
cd host && node runtime.js
# Sends malformed packet → ⚡ WASM traps → 🟢 Restarts in 8ms
```

## 📊 Key Metrics

| Metric | Python | WASM (Cold) | WASM (Hot-Standby) |
|--------|--------|-------------|--------------------|
| **Crash behavior** | Process dies | Sandbox traps | Sandbox traps |
| **Recovery time** | Manual (~60s) | Auto (~8ms) | **Instant (~100μs)** |
| **Packets lost** | All in-flight | 1-2 | **0** |
| **PLC impact** | Connection lost | None | None |

### Hot-Standby Redundancy

We apply industrial redundancy patterns (IEC 62439-3) at the software layer:

```
┌─────────────────┐     ┌─────────────────┐
│   INSTANCE 0    │ ←→  │   INSTANCE 1    │
│   (PRIMARY)     │     │   (STANDBY)     │
└─────────────────┘     └─────────────────┘

On crash: activeIndex swaps instantly (~100μs)
Failed instance rebuilds async (8ms, non-blocking)
```

**Why not just use fast restart?** Even 8ms loses packets. Hot-standby = zero loss.

### Why WASM Hot-Standby Beats Traditional Industrial Solutions

| Solution | Switchover Time | Memory Overhead | IPC Overhead | Complexity |
|----------|----------------|-----------------|--------------|------------|
| **PLC Hardware Redundancy** | ~10-50μs | 2x hardware cost | N/A | High |
| **PRP/HSR (IEC 62439-3)** | ~50μs | Network duplication | None | Medium |
| **Python Multiprocessing** | ~5ms | 30-50MB per worker | IPC penalty | Medium |
| **Docker Container Restart** | ~500ms-2s | Container overhead | Process isolation | Low |
| **WASM Hot-Standby** | **~100μs** | **~2MB per instance** | **None (same process)** | **Low** |

**Key Advantages of WASM:**

1. **Same-Process Isolation**: Both instances share the same Node.js process — no IPC overhead
2. **Memory Efficiency**: WASM linear memory is ~1-2MB vs Python's ~30-50MB runtime
3. **True Sandboxing**: Unlike containers, WASM provides language-level isolation
4. **Instant Instantiation**: Compiled module is cached; new instance is just memory allocation

## ⚠️ What This Doesn't Solve

WASM + WASI + Rust solve **software security** — not everything:

| ✅ We Solve | ❌ Still Need |
|-------------|--------------|
| Memory safety (Rust) | Network encryption (TLS) |
| Sandbox isolation (WASM) | Authentication (OAuth, certs) |
| Capability control (WASI) | Network redundancy (PRP/HSR) |
| Software fault recovery | Hardware/power redundancy |

See [**Security Analysis**](docs/SECURITY.md#what-each-technology-solves-and-doesnt) for the full breakdown.

## 📚 Documentation

- [**Architecture Deep Dive**](docs/ARCHITECTURE.md): Hot-standby pattern, "Compile-Once, Instantiate-Many"
- [**Security Analysis**](docs/SECURITY.md): What each technology solves, IEC 62443 alignment, limitations

## 📜 License

MIT © 2026
