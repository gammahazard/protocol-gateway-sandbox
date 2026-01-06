<p align="center">
  <img src="https://img.shields.io/badge/WASI-0.2%20Preview%202-blueviolet?style=for-the-badge&logo=webassembly" alt="WASI 0.2"/>
  <img src="https://img.shields.io/badge/Rust-1.75+-orange?style=for-the-badge&logo=rust" alt="Rust"/>
  <img src="https://img.shields.io/badge/Modbus-TCP-blue?style=for-the-badge" alt="Modbus TCP"/>
</p>

<h1 align="center">🔒 Protocol Gateway Sandbox</h1>

<p align="center">
  <strong>Safe Legacy Protocol Translation via WASM Sandboxing</strong><br/>
  <em>"How do I connect my 1990s PLC to the cloud without letting hackers into the control loop?"</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-in%20development-yellow" alt="Status"/>
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
| **Host Runtime** | JavaScript (Node.js) | WASM loader with crash recovery |
| **Mock Sources** | JS Shims | Simulated PLC and MQTT broker |
| **Dashboard** | Leptos → WASM | Real-time security console |

### IEC 62443 Alignment

Per IEC 62443 attack surface minimization, we implement only:
- `0x03` Read Holding Registers
- `0x04` Read Input Registers

All other function codes are **explicitly rejected**. This is intentional.

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
│       └── metrics.rs      # Gateway stats
├── host/                   # JavaScript runtime
│   ├── runtime.js          # WASM loader + crash recovery
│   ├── shim/
│   │   ├── modbus-source.js
│   │   ├── mqtt-sink.js
│   │   └── chaos-attacks.js
│   └── test/
│       └── fuzz.test.js    # The crown jewel
├── legacy/                 # Python "villain" comparison
│   └── vulnerable_gateway.py
├── dashboard/              # Leptos web UI
└── docs/
```

## 🚀 Quick Start

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

| Metric | Python | WASM |
|--------|--------|------|
| **Crash on malformed input** | Process dies | Sandbox traps |
| **Recovery time** | Manual restart (~60s) | Automatic (~8ms) |
| **Blast radius** | Entire gateway | Single request |
| **PLC impact** | Connection lost | None |

## 🔗 Portfolio Story

This project is the evolution of [Vanguard ICS Guardian](https://github.com/gammahazard/vanguard-ics-guardian):

| Project | Story | Skills Demonstrated |
|---------|-------|---------------------|
| **Vanguard ICS Guardian** | "I understand capability-based security" | WASI, IEC 62443, data diode |
| **Protocol Gateway Sandbox** | "I solved legacy protocol translation safely" | Binary parsing, fuzzing, crash containment |

Together they show: **Security depth + Engineering breadth**

## 📜 License

MIT © 2026
