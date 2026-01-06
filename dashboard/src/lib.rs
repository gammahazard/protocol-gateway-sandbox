// dashboard/src/lib.rs
// protocol gateway sandbox - security console dashboard
// dual terminal view: python crashes and restarts, wasm never crashes.

#![allow(unused)]

use leptos::*;
use wasm_bindgen::prelude::*;

#[derive(Clone)]
struct LogEntry {
    level: String,
    message: String,
}

// attack config
struct AttackConfig {
    name: &'static str,
    restart_time: i32,
    error_msg: &'static str,
    bytes_corrupted: u32,
}

fn get_attack_config(attack: &str) -> AttackConfig {
    match attack {
        "bufferOverflow" => AttackConfig {
            name: "Buffer Overflow",
            restart_time: 90,
            error_msg: "MemoryError: heap corruption at 0x7fff42",
            bytes_corrupted: 2048,
        },
        "illegalFunction" => AttackConfig {
            name: "Illegal Function Code 0xFF",
            restart_time: 75,
            error_msg: "ValueError: unsupported function code 0xFF",
            bytes_corrupted: 512,
        },
        "truncatedHeader" => AttackConfig {
            name: "Truncated Header (3 bytes)",
            restart_time: 60,
            error_msg: "struct.error: unpack requires 7 bytes, got 3",
            bytes_corrupted: 256,
        },
        _ => AttackConfig {
            name: "Random Garbage",
            restart_time: 45,
            error_msg: "UnicodeDecodeError: invalid byte 0xfe",
            bytes_corrupted: 128,
        },
    }
}

#[component]
pub fn App() -> impl IntoView {
    // consistent stats for both gateways
    let (python_processed, set_python_processed) = create_signal(0u64);
    let (python_rejected, set_python_rejected) = create_signal(0u64);
    let (python_downtime, set_python_downtime) = create_signal(0i32);
    
    let (wasm_processed, set_wasm_processed) = create_signal(0u64);
    let (wasm_rejected, set_wasm_rejected) = create_signal(0u64);
    let (wasm_downtime, set_wasm_downtime) = create_signal(0i32); // always 0
    
    // terminal logs
    let (python_logs, set_python_logs) = create_signal(Vec::<LogEntry>::new());
    let (wasm_logs, set_wasm_logs) = create_signal(Vec::<LogEntry>::new());
    
    // state
    let (python_crashed, set_python_crashed) = create_signal(false);
    let (python_countdown, set_python_countdown) = create_signal(0i32);
    let (is_running, set_is_running) = create_signal(false);
    let (selected_attack, set_selected_attack) = create_signal("bufferOverflow".to_string());
    let (total_bytes_processed, set_total_bytes_processed) = create_signal(0u64);
    
    let trigger_chaos = move |_| {
        set_is_running.set(true);
        let attack = selected_attack.get();
        let config = get_attack_config(&attack);
        
        // simulate some normal processing first
        let frames_before = 5;
        let bytes_per_frame = 19u64; // modbus frame size
        
        // python logs - detailed startup
        set_python_logs.set(vec![
            LogEntry { level: "info".into(), message: "$ python gateway.py --port 502".into() },
            LogEntry { level: "info".into(), message: "[INFO] Initializing struct-based parser...".into() },
            LogEntry { level: "success".into(), message: "[OK] Modbus Gateway listening on 0.0.0.0:502".into() },
            LogEntry { level: "info".into(), message: format!("[INFO] Processed {} frames ({} bytes)", frames_before, frames_before * bytes_per_frame) },
            LogEntry { level: "info".into(), message: format!("[RECV] Incoming: {} ({} bytes)", config.name, config.bytes_corrupted) },
        ]);
        set_python_processed.update(|n| *n += frames_before);
        
        // wasm logs - detailed startup
        set_wasm_logs.set(vec![
            LogEntry { level: "info".into(), message: "$ ./protocol-gateway --port 502".into() },
            LogEntry { level: "success".into(), message: "[OK] WASM sandbox initialized (mem: 64KB)".into() },
            LogEntry { level: "success".into(), message: "[OK] nom parser ready".into() },
            LogEntry { level: "info".into(), message: format!("[INFO] Processed {} frames ({} bytes)", frames_before, frames_before * bytes_per_frame) },
            LogEntry { level: "info".into(), message: format!("[RECV] Incoming: {} ({} bytes)", config.name, config.bytes_corrupted) },
        ]);
        set_wasm_processed.update(|n| *n += frames_before);
        set_total_bytes_processed.update(|n| *n += frames_before * bytes_per_frame * 2);
        
        set_timeout(
            move || {
                let config = get_attack_config(&attack);
                let telemetry_lost = config.restart_time as u64 * bytes_per_frame;
                
                // python crashes with detailed error
                set_python_logs.update(|logs| {
                    logs.push(LogEntry { level: "error".into(), message: format!("[ERROR] {}", config.error_msg) });
                    logs.push(LogEntry { level: "error".into(), message: "Traceback (most recent call last):".into() });
                    logs.push(LogEntry { level: "error".into(), message: "  File \"gateway.py\", line 42, in parse_mbap".into() });
                    logs.push(LogEntry { level: "error".into(), message: "    header = struct.unpack('>HHHB', data[:7])".into() });
                    logs.push(LogEntry { level: "error".into(), message: format!("💥 FATAL: exit(1) | Restart: {}s | Est. data loss: {} bytes", config.restart_time, telemetry_lost) });
                });
                set_python_crashed.set(true);
                set_python_rejected.update(|n| *n += 1);
                set_python_downtime.update(|n| *n += config.restart_time);
                set_python_countdown.set(config.restart_time);
                
                simulate_restart(set_python_logs, set_python_crashed, set_python_countdown, config.restart_time, telemetry_lost);
                
                // wasm handles gracefully - detailed
                let recovery_ms = 1.5 + (js_sys::Math::random() * 3.0);
                set_wasm_logs.update(|logs| {
                    logs.push(LogEntry { level: "warn".into(), message: format!("[WARN] nom::Err::Incomplete | Expected: 7 bytes, Got: {}", config.bytes_corrupted.min(6)) });
                    logs.push(LogEntry { level: "info".into(), message: "[INFO] Frame rejected - invalid MBAP header".into() });
                    logs.push(LogEntry { level: "success".into(), message: format!("[OK] Sandbox trap handled in {:.2}ms", recovery_ms) });
                    logs.push(LogEntry { level: "success".into(), message: "[OK] Parser state reset - ready for next frame".into() });
                });
                set_wasm_rejected.update(|n| *n += 1);
                // wasm_downtime stays at 0
                
                // wasm continues processing while python is down
                for delay in [1500, 3000, 4500] {
                    set_timeout(move || {
                        set_wasm_logs.update(|logs| {
                            logs.push(LogEntry { level: "info".into(), message: format!("[RECV] Frame: Read Holding Registers (19 bytes)") });
                            logs.push(LogEntry { level: "success".into(), message: "[OK] → MQTT: ics/telemetry/unit/1".into() });
                        });
                        set_wasm_processed.update(|n| *n += 1);
                        set_total_bytes_processed.update(|n| *n += bytes_per_frame);
                    }, std::time::Duration::from_millis(delay));
                }
                
                set_is_running.set(false);
            },
            std::time::Duration::from_millis(800),
        );
    };
    
    let reset_demo = move |_| {
        set_python_crashed.set(false);
        set_python_countdown.set(0);
        set_python_logs.set(Vec::new());
        set_wasm_logs.set(Vec::new());
        set_python_processed.set(0);
        set_python_rejected.set(0);
        set_python_downtime.set(0);
        set_wasm_processed.set(0);
        set_wasm_rejected.set(0);
        set_total_bytes_processed.set(0);
    };
    
    view! {
        <div class="dashboard">
            <Header/>
            <ScenarioContext/>
            
            <div class="terminals-container">
                <div class="terminal-panel python-terminal">
                    <div class="terminal-header">
                        <div class="terminal-title">"🐍 Python Gateway"</div>
                        <div class="terminal-subtitle">"struct.unpack"</div>
                        <div class="terminal-status"
                            class:crashed=move || python_crashed.get()
                            class:online=move || !python_crashed.get()
                        >
                            {move || {
                                if python_crashed.get() {
                                    let c = python_countdown.get();
                                    if c > 0 { format!("⏳ {}s", c) } else { "🔴 DOWN".into() }
                                } else { "🟢 UP".into() }
                            }}
                        </div>
                    </div>
                    <div class="terminal-output">
                        {move || {
                            let entries = python_logs.get();
                            if entries.is_empty() {
                                view! { <div class="terminal-placeholder">"$ ready"</div> }.into_view()
                            } else {
                                entries.into_iter().map(|e| {
                                    view! { <div class=format!("log-line log-{}", e.level)>{e.message}</div> }
                                }).collect_view()
                            }
                        }}
                    </div>
                </div>
                
                <div class="terminal-panel wasm-terminal">
                    <div class="terminal-header">
                        <div class="terminal-title">"🦀 WASM Gateway"</div>
                        <div class="terminal-subtitle">"nom parser"</div>
                        <div class="terminal-status online">"🟢 UP"</div>
                    </div>
                    <div class="terminal-output">
                        {move || {
                            let entries = wasm_logs.get();
                            if entries.is_empty() {
                                view! { <div class="terminal-placeholder">"$ ready"</div> }.into_view()
                            } else {
                                entries.into_iter().map(|e| {
                                    view! { <div class=format!("log-line log-{}", e.level)>{e.message}</div> }
                                }).collect_view()
                            }
                        }}
                    </div>
                </div>
            </div>
            
            <div class="stats-container">
                <div class="panel stats-panel python-stats">
                    <h2>"🐍 Python Stats"</h2>
                    <div class="stats-row">
                        <div class="stat-item">
                            <span class="stat-value">{python_processed}</span>
                            <span class="stat-label">"Processed"</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value error">{python_rejected}</span>
                            <span class="stat-label">"Rejected"</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value error">{python_downtime}"s"</span>
                            <span class="stat-label">"Downtime"</span>
                        </div>
                    </div>
                </div>
                
                <div class="panel stats-panel wasm-stats">
                    <h2>"🦀 WASM Stats"</h2>
                    <div class="stats-row">
                        <div class="stat-item">
                            <span class="stat-value">{wasm_processed}</span>
                            <span class="stat-label">"Processed"</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value warn">{wasm_rejected}</span>
                            <span class="stat-label">"Rejected"</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value">{wasm_downtime}"s"</span>
                            <span class="stat-label">"Downtime"</span>
                        </div>
                    </div>
                </div>
            </div>
            
            <div class="panel chaos-panel">
                <h2>"☠️ Attack Vector"</h2>
                <div class="chaos-controls">
                    <select class="attack-select" on:change=move |ev| set_selected_attack.set(event_target_value(&ev))>
                        <option value="bufferOverflow">"Buffer Overflow (90s restart)"</option>
                        <option value="illegalFunction">"Illegal Function (75s restart)"</option>
                        <option value="truncatedHeader">"Truncated Header (60s restart)"</option>
                        <option value="randomGarbage">"Random Garbage (45s restart)"</option>
                    </select>
                    <button class="chaos-button" disabled=is_running on:click=move |_| trigger_chaos(())>
                        {move || if is_running.get() { "⏳..." } else { "🎯 Attack" }}
                    </button>
                    <button class="reset-button" on:click=move |_| reset_demo(())>"🔄"</button>
                </div>
            </div>
            
            <Footer/>
        </div>
    }
}

fn simulate_restart(
    set_logs: WriteSignal<Vec<LogEntry>>,
    set_crashed: WriteSignal<bool>,
    set_countdown: WriteSignal<i32>,
    total: i32,
    bytes_lost: u64,
) {
    fn tick(remaining: i32, set_logs: WriteSignal<Vec<LogEntry>>, set_crashed: WriteSignal<bool>, set_countdown: WriteSignal<i32>, bytes_lost: u64) {
        if remaining <= 0 {
            set_logs.update(|logs| {
                logs.push(LogEntry { level: "info".into(), message: "[INFO] systemd: Restarting gateway.service...".into() });
                logs.push(LogEntry { level: "success".into(), message: "[OK] Gateway back online".into() });
                logs.push(LogEntry { level: "warn".into(), message: format!("[WARN] Total telemetry lost: {} bytes", bytes_lost) });
            });
            set_crashed.set(false);
            set_countdown.set(0);
            return;
        }
        set_countdown.set(remaining);
        if remaining % 20 == 0 || remaining <= 10 {
            set_logs.update(|logs| {
                logs.push(LogEntry { level: "warn".into(), message: format!("[WAIT] Restart in {}s...", remaining) });
            });
        }
        set_timeout(move || tick(remaining - 5, set_logs, set_crashed, set_countdown, bytes_lost), std::time::Duration::from_millis(350));
    }
    tick(total, set_logs, set_crashed, set_countdown, bytes_lost);
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <div class="logo">
                <span class="logo-icon">"🔒"</span>
                <h1>"Protocol Gateway Sandbox"</h1>
            </div>
            <p class="subtitle">"WASM Crash Containment"</p>
        </header>
    }
}

#[component]
fn ScenarioContext() -> impl IntoView {
    view! {
        <section class="scenario-context">
            <div class="scenario-diagram">
                <div class="diagram-node modbus">
                    <span class="node-icon">"📟"</span>
                    <span class="node-label">"Modbus PLC"</span>
                </div>
                <span class="diagram-arrow">"→"</span>
                <div class="diagram-node attack">
                    <span class="node-icon">"☠️"</span>
                    <span class="node-label">"Malformed Frame"</span>
                </div>
                <span class="diagram-arrow">"→"</span>
                <div class="diagram-node gateway">
                    <span class="node-icon">"🔄"</span>
                    <span class="node-label">"Gateway"</span>
                </div>
                <span class="diagram-arrow">"→"</span>
                <div class="diagram-node mqtt">
                    <span class="node-icon">"☁️"</span>
                    <span class="node-label">"MQTT"</span>
                </div>
            </div>
            <p class="scenario-description">"Same attack → Python crashes, WASM continues"</p>
        </section>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="tech-badges">
                <span class="badge rust">"Rust"</span>
                <span class="badge wasm">"WASI 0.2"</span>
                <span class="badge">"nom"</span>
                <span class="badge security">"IEC 62443"</span>
            </div>
        </footer>
    }
}

fn set_timeout<F: FnOnce() + 'static>(cb: F, dur: std::time::Duration) {
    use wasm_bindgen::closure::Closure;
    let window = web_sys::window().unwrap();
    let closure = Closure::once(cb);
    window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(), dur.as_millis() as i32
    ).unwrap();
    closure.forget();
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
