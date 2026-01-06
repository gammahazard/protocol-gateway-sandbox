// dashboard/src/lib.rs
// protocol gateway sandbox - leptos security console dashboard.
// dual terminal view: python crashes and restarts, wasm never crashes.

#![allow(unused)]

use leptos::*;
use wasm_bindgen::prelude::*;

// main app component
#[component]
pub fn App() -> impl IntoView {
    // gateway state
    let (frames_processed, set_frames_processed) = create_signal(0u64);
    let (frames_invalid, set_frames_invalid) = create_signal(0u64);
    let (wasm_recovery_time, set_wasm_recovery_time) = create_signal(0.0f64);
    
    // terminal logs - separate for each gateway
    let (python_logs, set_python_logs) = create_signal(Vec::<LogEntry>::new());
    let (wasm_logs, set_wasm_logs) = create_signal(Vec::<LogEntry>::new());
    
    // python restart state
    let (python_crashed, set_python_crashed) = create_signal(false);
    let (python_restart_countdown, set_python_restart_countdown) = create_signal(0i32);
    let (is_running, set_is_running) = create_signal(false);
    let (selected_attack, set_selected_attack) = create_signal("bufferOverflow".to_string());
    
    // helper to add python log
    let add_python_log = move |level: &str, message: &str| {
        set_python_logs.update(|logs| {
            logs.push(LogEntry { level: level.to_string(), message: message.to_string() });
            if logs.len() > 50 { logs.remove(0); }
        });
    };
    
    // helper to add wasm log
    let add_wasm_log = move |level: &str, message: &str| {
        set_wasm_logs.update(|logs| {
            logs.push(LogEntry { level: level.to_string(), message: message.to_string() });
            if logs.len() > 50 { logs.remove(0); }
        });
    };
    
    // trigger chaos attack
    let trigger_chaos = move |_| {
        set_is_running.set(true);
        let attack = selected_attack.get();
        
        // both gateways receive malformed packet
        add_python_log("info", &format!("$ ./gateway.py --port 502"));
        add_python_log("info", "Modbus Gateway started on port 502");
        add_python_log("info", "Waiting for connections...");
        add_python_log("info", &format!("Received frame: {} attack", attack));
        
        add_wasm_log("info", "$ ./protocol-gateway --port 502");
        add_wasm_log("success", "WASM sandbox initialized");
        add_wasm_log("info", "Modbus Gateway started on port 502");
        add_wasm_log("info", "Waiting for connections...");
        add_wasm_log("info", &format!("Received frame: {} attack", attack));
        
        // simulate async processing
        set_timeout(
            move || {
                // python crashes immediately
                add_python_log("error", "struct.error: unpack requires a buffer of 7 bytes");
                add_python_log("error", "Traceback (most recent call last):");
                add_python_log("error", "  File \"gateway.py\", line 42");
                add_python_log("error", "    header = struct.unpack('>HHHB', data[:7])");
                add_python_log("error", "💥 FATAL: Process terminated with exit code 1");
                set_python_crashed.set(true);
                
                // start python restart countdown (60 seconds simulated as 6 ticks)
                set_python_restart_countdown.set(60);
                start_python_restart(set_python_restart_countdown, set_python_crashed, add_python_log);
                
                // wasm handles gracefully and continues
                add_wasm_log("warn", "nom::Err::Incomplete(Size(7))");
                add_wasm_log("info", "Frame validation failed: truncated header");
                set_frames_invalid.update(|n| *n += 1);
                
                let recovery = 2.0 + (js_sys::Math::random() * 4.0);
                set_wasm_recovery_time.set(recovery);
                add_wasm_log("success", &format!("Sandbox recovered in {:.2}ms", recovery));
                add_wasm_log("success", "✅ Gateway operational - continuing to accept frames");
                
                // simulate wasm continuing to process normal frames
                set_timeout(move || {
                    add_wasm_log("info", "Received frame: normal read request");
                    add_wasm_log("success", "Frame processed → published to MQTT");
                    set_frames_processed.update(|n| *n += 1);
                }, std::time::Duration::from_millis(1000));
                
                set_timeout(move || {
                    add_wasm_log("info", "Received frame: normal read request");
                    add_wasm_log("success", "Frame processed → published to MQTT");
                    set_frames_processed.update(|n| *n += 1);
                }, std::time::Duration::from_millis(2000));
                
                set_is_running.set(false);
            },
            std::time::Duration::from_millis(800),
        );
    };
    
    // reset demo
    let reset_demo = move |_| {
        set_python_crashed.set(false);
        set_python_restart_countdown.set(0);
        set_python_logs.set(Vec::new());
        set_wasm_logs.set(Vec::new());
        set_frames_processed.set(0);
        set_frames_invalid.set(0);
    };
    
    view! {
        <div class="dashboard">
            <Header/>
            
            <ScenarioContext/>
            
            <div class="terminals-container">
                <TerminalPanel
                    title="🐍 Python Gateway"
                    subtitle="Traditional struct.unpack parser"
                    logs=python_logs
                    crashed=python_crashed
                    restart_countdown=python_restart_countdown
                    terminal_class="python-terminal"
                />
                
                <TerminalPanel
                    title="🦀 WASM Gateway"
                    subtitle="Sandboxed nom parser"
                    logs=wasm_logs
                    crashed=Signal::derive(move || false)
                    restart_countdown=Signal::derive(move || 0)
                    terminal_class="wasm-terminal"
                />
            </div>
            
            <div class="controls-row">
                <StatsPanel
                    frames_processed=frames_processed
                    frames_invalid=frames_invalid
                    recovery_time=wasm_recovery_time
                />
                
                <ChaosPanel
                    selected_attack=selected_attack
                    set_selected_attack=set_selected_attack
                    is_running=is_running
                    on_trigger=trigger_chaos
                    on_reset=reset_demo
                />
            </div>
            
            <Footer/>
        </div>
    }
}

// start python restart countdown
fn start_python_restart(
    set_countdown: WriteSignal<i32>,
    set_crashed: WriteSignal<bool>,
    add_log: impl Fn(&str, &str) + Copy + 'static,
) {
    // tick every second (simulated - we'll do faster for demo)
    fn tick(remaining: i32, set_countdown: WriteSignal<i32>, set_crashed: WriteSignal<bool>, add_log: impl Fn(&str, &str) + Copy + 'static) {
        if remaining <= 0 {
            add_log("info", "Restarting gateway process...");
            add_log("info", "$ ./gateway.py --port 502");
            add_log("success", "Modbus Gateway started on port 502");
            add_log("info", "⚠️  Lost 60 seconds of telemetry data");
            set_crashed.set(false);
            set_countdown.set(0);
            return;
        }
        
        set_countdown.set(remaining);
        if remaining == 60 {
            add_log("warn", &format!("Restarting in {}s...", remaining));
        } else if remaining % 15 == 0 {
            add_log("warn", &format!("Restarting in {}s...", remaining));
        }
        
        set_timeout(
            move || tick(remaining - 5, set_countdown, set_crashed, add_log),
            std::time::Duration::from_millis(500), // 500ms = simulated 5 seconds
        );
    }
    
    tick(60, set_countdown, set_crashed, add_log);
}

// header component
#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <div class="logo">
                <span class="logo-icon">"🔒"</span>
                <h1>"Protocol Gateway Sandbox"</h1>
            </div>
            <p class="subtitle">"WASM Crash Containment Demo - Python vs Rust"</p>
        </header>
    }
}

// scenario context
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
            </div>
            <p class="scenario-description">
                "Watch what happens when both gateways receive the same malicious Modbus frame."
            </p>
        </section>
    }
}

// terminal panel component
#[component]
fn TerminalPanel(
    title: &'static str,
    subtitle: &'static str,
    logs: ReadSignal<Vec<LogEntry>>,
    crashed: Signal<bool>,
    restart_countdown: Signal<i32>,
    terminal_class: &'static str,
) -> impl IntoView {
    view! {
        <div class=format!("terminal-panel {}", terminal_class)>
            <div class="terminal-header">
                <div class="terminal-title">{title}</div>
                <div class="terminal-subtitle">{subtitle}</div>
                <div class="terminal-status" class:crashed=crashed class:online=move || !crashed.get()>
                    {move || if crashed.get() {
                        let countdown = restart_countdown.get();
                        if countdown > 0 {
                            format!("⏳ RESTARTING ({countdown}s)")
                        } else {
                            "🔴 CRASHED".to_string()
                        }
                    } else {
                        "🟢 ONLINE".to_string()
                    }}
                </div>
            </div>
            <div class="terminal-output">
                {move || {
                    let entries = logs.get();
                    if entries.is_empty() {
                        view! {
                            <div class="terminal-placeholder">
                                "$ waiting for attack simulation..."
                            </div>
                        }.into_view()
                    } else {
                        entries.into_iter().map(|entry| {
                            let class = format!("log-line log-{}", entry.level);
                            view! {
                                <div class=class>{entry.message.clone()}</div>
                            }
                        }).collect_view()
                    }
                }}
            </div>
        </div>
    }
}

// stats panel
#[component]
fn StatsPanel(
    frames_processed: ReadSignal<u64>,
    frames_invalid: ReadSignal<u64>,
    recovery_time: ReadSignal<f64>,
) -> impl IntoView {
    view! {
        <section class="panel stats-panel">
            <h2>"📊 WASM Gateway Stats"</h2>
            <div class="stats-row">
                <div class="stat-item">
                    <span class="stat-value">{frames_processed}</span>
                    <span class="stat-label">"Processed"</span>
                </div>
                <div class="stat-item">
                    <span class="stat-value error">{frames_invalid}</span>
                    <span class="stat-label">"Rejected"</span>
                </div>
                <div class="stat-item">
                    <span class="stat-value">{move || format!("{:.1}ms", recovery_time.get())}</span>
                    <span class="stat-label">"Recovery"</span>
                </div>
            </div>
        </section>
    }
}

// chaos panel
#[component]
fn ChaosPanel(
    selected_attack: ReadSignal<String>,
    set_selected_attack: WriteSignal<String>,
    is_running: ReadSignal<bool>,
    on_trigger: impl Fn(()) + 'static,
    on_reset: impl Fn(()) + 'static,
) -> impl IntoView {
    let attacks = vec![
        ("bufferOverflow", "Buffer Overflow"),
        ("truncatedHeader", "Truncated Header"),
        ("illegalFunction", "Illegal Function Code"),
        ("randomGarbage", "Random Garbage"),
    ];
    
    view! {
        <section class="panel chaos-panel">
            <h2>"☠️ Attack Vector"</h2>
            <select class="attack-select" on:change=move |ev| {
                set_selected_attack.set(event_target_value(&ev));
            }>
                {attacks.into_iter().map(|(id, name)| {
                    view! { <option value=id selected=move || selected_attack.get() == id>{name}</option> }
                }).collect_view()}
            </select>
            
            <div class="button-row">
                <button
                    class="chaos-button"
                    disabled=is_running
                    on:click=move |_| on_trigger(())
                >
                    {move || if is_running.get() { "⏳ Attacking..." } else { "🎯 Launch Attack" }}
                </button>
                
                <button class="reset-button" on:click=move |_| on_reset(())>
                    "🔄 Reset"
                </button>
            </div>
        </section>
    }
}

// footer
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

// log entry
#[derive(Clone)]
struct LogEntry {
    level: String,
    message: String,
}

// timeout helper
fn set_timeout<F>(callback: F, duration: std::time::Duration)
where
    F: FnOnce() + 'static,
{
    use wasm_bindgen::closure::Closure;
    let window = web_sys::window().unwrap();
    let cb = Closure::once(callback);
    window.set_timeout_with_callback_and_timeout_and_arguments_0(
        cb.as_ref().unchecked_ref(),
        duration.as_millis() as i32,
    ).unwrap();
    cb.forget();
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
