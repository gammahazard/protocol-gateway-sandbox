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
  <img src="https://img.shields.io/badge/tests-13%20passing-brightgreen" alt="Tests"/>
  <a href="https://protocol-gateway-sandbox.vercel.app"><img src="https://img.shields.io/badge/demo-live-blue" alt="Demo"/></a>
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
│   ├── runtime.js          # **2oo3 TMR voting + crash recovery**
│   ├── shim/
│   │   ├── modbus-source.js
│   │   ├── mqtt-sink.js
│   │   └── chaos-attacks.js
│   └── test/
│       └── fuzz.test.js    # Security invariant tests
├── cli/                    # Node.js CLI demo
│   └── run.mjs             # **Real benchmarks outside browser**
├── legacy/                 # Python "villain" comparison
│   └── vulnerable_gateway.py
├── dashboard/              # Leptos web UI
│   ├── src/lib.rs          # **Real WASM measurements + 2oo3 visualization**
│   └── styles.css          # Security console dark theme
└── docs/
    ├── [ARCHITECTURE.md](docs/ARCHITECTURE.md)     # 2oo3 TMR pattern
    └── [SECURITY.md](docs/SECURITY.md)             # IEC 62443 + SIL 3 alignment
```

## 🖥️ Dashboard Demo

The dashboard shows **two live terminals side-by-side** with **real WASM measurements**:

| Python (Multiprocessing) | WASM (2oo3 Voting) |
|--------------------------|--------------------|
| 3 workers: 1 active, 2 idle | 3 instances: all voting |
| Crash detection only | **Fault detection via voting** |
| ~500ms worker spawn (simulated) | **~4ms instantiate (real)** |
| No fault isolation | Faulty instance identified |

### Attack Demo: Buffer Overflow

<p align="center">
  <img src="docs/assets/attack_buffer_overflow.png" alt="Buffer Overflow Attack Demo" width="800"/>
</p>

**Key observations:**
- **Python:** 500ms downtime, frames lost during crash
- **WASM:** 0ms downtime, 2/3 voting continues, instance rebuilt in ~7ms (real)

### Real vs Simulated Metrics

| Metric | Source |
|--------|--------|
| WASM compile time | ✅ Real `WebAssembly.compile()` |
| WASM instantiate time | ✅ Real `WebAssembly.instantiate()` |
| WASM rebuild time | ✅ Real (re-instantiate during fault recovery) |
| WASM memory | ✅ Real measurement |
| Python spawn time | 🔶 Simulated (~500ms based on benchmarks) |

Run locally:
```bash
# Dashboard (browser demo)
cd dashboard && trunk serve
# Open http://localhost:8080

# CLI benchmark (Node.js - proves edge portability)
node cli/run.mjs
# Shows real compile/instantiate times
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

| Metric | Python | WASM (Cold) | WASM (2oo3 TMR) |
|--------|--------|-------------|-----------------|
| **Crash behavior** | Process dies | Sandbox traps | Sandbox traps |
| **Recovery time** | Manual (~60s) | Auto (~8ms) | **Instant (voting)** |
| **Fault detection** | Crash only | Crash only | **Wrong result detected** |
| **Packets lost** | All in-flight | 1-2 | **0** |

### 2oo3 Triple Modular Redundancy (TMR)

We apply SIL 3 safety patterns (IEC 61508) at the software layer:

```
┌───────────┐     ┌───────────┐     ┌───────────┐
│ INSTANCE 0│     │ INSTANCE 1│     │ INSTANCE 2│
│    ✓      │     │    ✓      │     │    ✗      │
└─────┬─────┘     └─────┬─────┘     └─────┬─────┘
      │                 │                 │
      └────────┬────────┴────────┬────────┘
               │     VOTER       │
               │  2/3 agree ✓    │
               └────────┬────────┘
                        ▼
                  Result: OK
                  Faulty: Instance 2 (rebuild async)
```

**Why 2oo3 over 1oo2?** 
- 1oo2 detects crashes only
- 2oo3 detects crashes **AND wrong results** (Byzantine faults)
- SIL 3 safety systems (Triconex, HIMA) use 2oo3 for emergency shutdown

### Why WASM 2oo3 Beats Traditional Industrial Solutions

| Solution | Fault Detection | Rebuild Time | Memory |
|----------|----------------|--------------|--------|
| **PLC 2oo3 (Triconex)** | Voting | Hardware | Expensive |
| **Python Multiprocessing** | Crash only | ~500ms | 30-50MB/worker |
| **Docker Restart** | Crash only | ~2-5s | Container overhead |
| **WASM 2oo3** | **Voting** | **~8ms** | **~2MB/instance** |

**Key Advantages of WASM 2oo3:**

1. **Voting Fault Detection**: Identifies *which* instance is faulty, not just that one crashed
2. **Same-Process Isolation**: All 3 instances share the same Node.js process — no IPC overhead
3. **Memory Efficiency**: WASM linear memory is ~2MB per instance vs Python's ~30-50MB
4. **Async Rebuild**: Faulty instance rebuilds in background (~8ms) without blocking voting

## ⚠️ What This Doesn't Solve

WASM + WASI + Rust solve **software security** — not everything:

| ✅ We Solve | ❌ Still Need |
|-------------|--------------|
| Memory safety (Rust) | Network encryption (TLS) |
| Sandbox isolation (WASM) | Authentication (OAuth, certs) |
| Capability control (WASI) | Network redundancy (PRP/HSR) |
| Software fault recovery | Hardware/power redundancy |

See [**Security Analysis**](docs/SECURITY.md#what-each-technology-solves-and-doesnt) for the full breakdown.

## 🔧 Deployment Targets

The same WASM component runs anywhere there's a runtime:

| Platform | Runtime | Use Case |
|----------|---------|----------|
| **Browser** | Built-in (V8) | Dashboard demo (this repo) |
| **Node.js** | V8 / JCO | Development, testing |
| **Edge Devices** | Wasmtime, WasmEdge | Industrial gateways |
| **Embedded** | WAMR | Microcontrollers, PLCs |
| **Cloud** | Fastly, Cloudflare Workers | Serverless edge |

### Example Hardware

| Device | Specs | Notes |
|--------|-------|-------|
| Raspberry Pi 4 | 4GB RAM, ARM64 | Runs Wasmtime natively |
| Industrial PC (Advantech, Moxa) | x64, 2-8GB | Production-ready |
| ESP32 | 520KB RAM | WAMR interpreter mode |

**Key insight:** Write once, deploy to browser (demo), server (test), and edge device (production) with zero code changes.

### 📡 Size & Bandwidth Comparison

For remote deployments with limited connectivity (offshore rigs, remote substations):

| Package | Size | Transfer @ 1 Mbps |
|---------|------|-------------------|
| **WASM Component** | ~68 KB | **<1 second** |
| Docker (Python) | ~500 MB | ~67 minutes |
| Docker (ML stack) | ~2 GB | ~4.5 hours |

*This is why WASM matters for remote ICS environments.*

## 📚 Documentation

- [**Architecture Deep Dive**](docs/ARCHITECTURE.md): 2oo3 TMR voting, "Compile-Once, Instantiate-Many"
- [**Security Analysis**](docs/SECURITY.md): What each technology solves, SIL 3 alignment, limitations

## 📜 License

MIT © 2026
