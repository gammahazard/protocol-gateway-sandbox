// dashboard/src/lib.rs
// protocol gateway sandbox - leptos security console dashboard.
// visualizes the wasm vs python gateway comparison and allows
// triggering chaos attacks to demonstrate crash containment.

use leptos::*;
use wasm_bindgen::prelude::*;

// main app component
#[component]
pub fn App() -> impl IntoView {
    // reactive state
    let (frames_processed, set_frames_processed) = create_signal(0u64);
    let (frames_invalid, set_frames_invalid) = create_signal(0u64);
    let (bytes_in, set_bytes_in) = create_signal(0u64);
    let (bytes_out, set_bytes_out) = create_signal(0u64);
    let (recovery_time, set_recovery_time) = create_signal(0.0f64);
    let (logs, set_logs) = create_signal(Vec::<LogEntry>::new());
    let (is_running, set_is_running) = create_signal(false);
    let (selected_attack, set_selected_attack) = create_signal("bufferOverflow".to_string());
    
    // python comparison state (simulated)
    let (python_status, set_python_status) = create_signal("idle".to_string());
    let (python_crashed, set_python_crashed) = create_signal(false);
    
    // add log entry helper
    let add_log = move |level: &str, prefix: &str, message: &str| {
        set_logs.update(|logs| {
            logs.push(LogEntry {
                level: level.to_string(),
                prefix: prefix.to_string(),
                message: message.to_string(),
            });
            // keep last 100 entries
            if logs.len() > 100 {
                logs.remove(0);
            }
        });
    };
    
    // run single frame (simulated)
    let run_frame = move |_| {
        set_frames_processed.update(|n| *n += 1);
        set_bytes_in.update(|n| *n += 19);
        set_bytes_out.update(|n| *n += 128);
        add_log("success", "[WASM]", "Frame processed successfully");
    };
    
    // trigger chaos attack
    let trigger_chaos = move |_| {
        set_is_running.set(true);
        let attack = selected_attack.get();
        
        add_log("warn", "[CHAOS]", &format!("Injecting attack: {}", attack));
        
        // simulate python crash
        set_python_status.set("processing".to_string());
        
        // use timeout to simulate async behavior
        set_timeout(
            move || {
                // python crashes
                set_python_crashed.set(true);
                set_python_status.set("CRASHED".to_string());
                add_log("error", "[PYTHON]", "💥 PROCESS DIED - struct.unpack failed");
                add_log("error", "[PYTHON]", "Gateway offline. Manual restart required.");
                
                // wasm survives
                set_frames_invalid.update(|n| *n += 1);
                add_log("warn", "[WASM]", "Malformed frame detected");
                add_log("wasm", "[WASM]", "Parser returned Err(Incomplete) - handled gracefully");
                
                // simulate recovery time measurement
                let recovery = 2.3 + (js_sys::Math::random() * 5.0);
                set_recovery_time.set(recovery);
                add_log("success", "[WASM]", &format!("Recovery time: {:.2}ms", recovery));
                add_log("success", "[WASM]", "✅ Gateway operational - sandbox contained the attack");
                
                set_is_running.set(false);
            },
            std::time::Duration::from_millis(500),
        );
    };
    
    // reset demo
    let reset_demo = move |_| {
        set_python_crashed.set(false);
        set_python_status.set("idle".to_string());
        set_logs.set(Vec::new());
        add_log("info", "[SYSTEM]", "Demo reset. Ready for new attack.");
    };
    
    view! {
        <div class="dashboard">
            <Header/>
            
            <ScenarioContext/>
            
            <div class="main-content">
                <div class="left-panel">
                    <StatsPanel
                        frames_processed=frames_processed
                        frames_invalid=frames_invalid
                        bytes_in=bytes_in
                        bytes_out=bytes_out
                        recovery_time=recovery_time
                    />
                    
                    <ComparisonPanel
                        python_status=python_status
                        python_crashed=python_crashed
                    />
                    
                    <ChaosPanel
                        selected_attack=selected_attack
                        set_selected_attack=set_selected_attack
                        is_running=is_running
                        on_trigger=trigger_chaos
                        on_reset=reset_demo
                    />
                </div>
                
                <div class="right-panel">
                    <ConsolePanel logs=logs/>
                </div>
            </div>
            
            <Footer/>
        </div>
    }
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
            <p class="subtitle">"Safe Legacy Protocol Translation via WASM Sandboxing"</p>
        </header>
    }
}

// scenario context with diagram
#[component]
fn ScenarioContext() -> impl IntoView {
    view! {
        <section class="scenario-context">
            <div class="scenario-header">
                <span class="scenario-icon">"🏭"</span>
                <div>
                    <h2>"Modbus TCP → MQTT Gateway"</h2>
                    <p class="scenario-subtitle">"Translating legacy PLC protocols to modern cloud systems"</p>
                </div>
            </div>
            
            <div class="scenario-diagram">
                <div class="diagram-node modbus">
                    <span class="node-icon">"📟"</span>
                    <span class="node-label">"Modbus PLC"</span>
                    <span class="node-status">"10.0.0.50:502"</span>
                </div>
                
                <span class="diagram-arrow">"→"</span>
                
                <div class="diagram-node wasm">
                    <span class="node-icon">"🦀"</span>
                    <span class="node-label">"WASM Parser"</span>
                    <span class="node-status safe">"Sandboxed"</span>
                </div>
                
                <span class="diagram-arrow">"→"</span>
                
                <div class="diagram-node mqtt">
                    <span class="node-icon">"☁️"</span>
                    <span class="node-label">"MQTT Broker"</span>
                    <span class="node-status">"Cloud/SCADA"</span>
                </div>
            </div>
            
            <p class="scenario-description">
                "Malicious Modbus frames crash the "<strong>"WASM sandbox"</strong>", not the host process. "
                "The gateway recovers in "<strong>"<10ms"</strong>" while a traditional Python parser would crash entirely."
            </p>
        </section>
    }
}

// stats panel
#[component]
fn StatsPanel(
    frames_processed: ReadSignal<u64>,
    frames_invalid: ReadSignal<u64>,
    bytes_in: ReadSignal<u64>,
    bytes_out: ReadSignal<u64>,
    recovery_time: ReadSignal<f64>,
) -> impl IntoView {
    view! {
        <section class="panel stats-panel">
            <h2>"📊 Gateway Metrics"</h2>
            <div class="stats-grid">
                <div class="stat-item">
                    <span class="stat-label">"Frames Processed"</span>
                    <span class="stat-value">{frames_processed}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">"Frames Invalid"</span>
                    <span class="stat-value error">{frames_invalid}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">"Bytes In"</span>
                    <span class="stat-value">{bytes_in}</span>
                    <span class="stat-unit">"bytes"</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">"Bytes Out"</span>
                    <span class="stat-value">{bytes_out}</span>
                    <span class="stat-unit">"bytes"</span>
                </div>
                <div class="stat-item" style="grid-column: span 2;">
                    <span class="stat-label">"Last Recovery Time"</span>
                    <span class="stat-value">
                        {move || format!("{:.2}", recovery_time.get())}
                    </span>
                    <span class="stat-unit">"ms (target: <10ms)"</span>
                </div>
            </div>
        </section>
    }
}

// comparison panel
#[component]
fn ComparisonPanel(
    python_status: ReadSignal<String>,
    python_crashed: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <section class="panel comparison-panel">
            <h2>"⚔️ Python vs WASM Comparison"</h2>
            <div class="comparison-grid">
                <div class="comparison-column python">
                    <div class="comparison-header">
                        <span class="comparison-icon">"🐍"</span>
                        <span class="comparison-title">"Python Gateway"</span>
                    </div>
                    <div class="comparison-metrics">
                        <div class="metric-row">
                            <span class="metric-label">"Status"</span>
                            <span class="metric-value bad" class:bad=python_crashed>
                                {python_status}
                            </span>
                        </div>
                        <div class="metric-row">
                            <span class="metric-label">"On Malformed Input"</span>
                            <span class="metric-value bad">"Process Crash"</span>
                        </div>
                        <div class="metric-row">
                            <span class="metric-label">"Recovery Time"</span>
                            <span class="metric-value bad">"~60 seconds"</span>
                        </div>
                        <div class="metric-row">
                            <span class="metric-label">"PLC Impact"</span>
                            <span class="metric-value bad">"Connection Lost"</span>
                        </div>
                    </div>
                </div>
                
                <div class="comparison-column wasm">
                    <div class="comparison-header">
                        <span class="comparison-icon">"🦀"</span>
                        <span class="comparison-title">"WASM Gateway"</span>
                    </div>
                    <div class="comparison-metrics">
                        <div class="metric-row">
                            <span class="metric-label">"Status"</span>
                            <span class="metric-value good">"Operational"</span>
                        </div>
                        <div class="metric-row">
                            <span class="metric-label">"On Malformed Input"</span>
                            <span class="metric-value good">"Sandbox Traps"</span>
                        </div>
                        <div class="metric-row">
                            <span class="metric-label">"Recovery Time"</span>
                            <span class="metric-value good">"<10ms"</span>
                        </div>
                        <div class="metric-row">
                            <span class="metric-label">"PLC Impact"</span>
                            <span class="metric-value good">"None"</span>
                        </div>
                    </div>
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
        ("bufferOverflow", "Buffer Overflow", "Length claims 255 bytes, only 2 provided"),
        ("truncatedHeader", "Truncated Header", "Only 3 bytes when 7 required"),
        ("illegalFunction", "Illegal Function", "Function code 0xFF (undefined)"),
        ("wrongProtocol", "Wrong Protocol ID", "0xDEAD instead of 0x0000"),
        ("randomGarbage", "Random Garbage", "Random bytes of random length"),
    ];
    
    view! {
        <section class="panel chaos-panel">
            <h2>"☠️ Chaos Testing"</h2>
            
            <div class="attack-selector">
                {attacks.into_iter().map(|(id, name, desc)| {
                    let id_clone = id.to_string();
                    let is_selected = move || selected_attack.get() == id;
                    
                    view! {
                        <label
                            class="attack-option"
                            class:selected=is_selected
                            on:click=move |_| set_selected_attack.set(id_clone.clone())
                        >
                            <input type="radio" name="attack" value=id checked=is_selected/>
                            <div>
                                <div class="attack-name">{name}</div>
                                <div class="attack-desc">{desc}</div>
                            </div>
                        </label>
                    }
                }).collect_view()}
            </div>
            
            <button
                class="chaos-button"
                disabled=is_running
                on:click=move |_| on_trigger(())
            >
                {move || if is_running.get() { "⏳ Attacking..." } else { "🎯 Launch Attack" }}
            </button>
            
            <button
                class="chaos-button"
                style="background: linear-gradient(135deg, #555, #333);"
                on:click=move |_| on_reset(())
            >
                "🔄 Reset Demo"
            </button>
        </section>
    }
}

// console panel
#[component]
fn ConsolePanel(logs: ReadSignal<Vec<LogEntry>>) -> impl IntoView {
    view! {
        <section class="panel console-panel">
            <h2>"📜 Event Console"</h2>
            <div class="console-output">
                {move || {
                    let entries = logs.get();
                    if entries.is_empty() {
                        view! {
                            <div class="console-placeholder">
                                "Select an attack and click 'Launch Attack' to begin..."
                            </div>
                        }.into_view()
                    } else {
                        entries.into_iter().map(|entry| {
                            let class = format!("log-entry log-{}", entry.level);
                            view! {
                                <div class=class>
                                    <span class="log-prefix">{entry.prefix}</span>
                                    <span class="log-message">{entry.message}</span>
                                </div>
                            }
                        }).collect_view()
                    }
                }}
            </div>
        </section>
    }
}

// footer
#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <p>"Protocol Gateway Sandbox - WASM Security Demonstration"</p>
            <div class="tech-badges">
                <span class="badge rust">"Rust"</span>
                <span class="badge wasm">"WASI 0.2"</span>
                <span class="badge">"nom"</span>
                <span class="badge security">"IEC 62443"</span>
                <span class="badge">"Leptos"</span>
            </div>
        </footer>
    }
}

// log entry struct
#[derive(Clone)]
struct LogEntry {
    level: String,
    prefix: String,
    message: String,
}

// helper for set_timeout
fn set_timeout<F>(callback: F, duration: std::time::Duration)
where
    F: FnOnce() + 'static,
{
    use wasm_bindgen::closure::Closure;
    
    let window = web_sys::window().unwrap();
    let cb = Closure::once(callback);
    
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            duration.as_millis() as i32,
        )
        .unwrap();
    
    cb.forget();
}

// mount the app
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
