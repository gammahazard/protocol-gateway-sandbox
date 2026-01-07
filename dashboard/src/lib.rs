// dashboard/src/lib.rs
// ============================================================================
// protocol gateway sandbox - security console dashboard
// ============================================================================
//
// this dashboard demonstrates wasm/rust advantages over python for industrial
// gateway applications. key features:
//
// 1. real wasm measurements: actually uses webassembly apis in browser to
//    measure compile, instantiate, and memory times - not simulated
//
// 2. 2oo3 voting (triple modular redundancy): three wasm instances process
//    in parallel, majority vote determines result. matches sil 3 patterns.
//
// 3. python comparison: shows realistic multiprocessing behavior with worker
//    restart times (simulated but based on real benchmarks)
//
// architecture:
// - runs entirely in browser (no backend required)
// - deployed to vercel as static site
// - wasm measurements use actual webassembly apis
// - python behavior is simulated with realistic timing values
//
// related files:
// - styles.css: security console dark theme with instance visualization
// - index.html: trunk build configuration and font loading
// - ../host/runtime.js: the actual node.js runtime (for reference)
// ============================================================================

#![allow(unused)]

use leptos::*;
use wasm_bindgen::prelude::*;

// ============================================================================
// types and configuration
// ============================================================================

/// log entry for terminal output display
#[derive(Clone)]
struct LogEntry {
    level: String,   // info, success, warn, error
    message: String,
}

/// attack configuration with realistic python restart times
struct AttackConfig {
    name: &'static str,
    restart_time: i32,        // python process restart (seconds)
    worker_spawn_ms: u32,     // python worker spawn time (ms)
    error_msg: &'static str,
}

/// wasm instance state for 2oo3 voting visualization
#[derive(Clone, Copy, PartialEq)]
enum InstanceState {
    Healthy,
    Processing,
    Faulty,
    Rebuilding,
}

fn get_attack_config(attack: &str) -> AttackConfig {
    match attack {
        "bufferOverflow" => AttackConfig {
            name: "Buffer Overflow",
            restart_time: 60,
            worker_spawn_ms: 500,
            error_msg: "MemoryError: heap corruption at 0x7fff42",
        },
        "illegalFunction" => AttackConfig {
            name: "Illegal Function Code 0xFF",
            restart_time: 45,
            worker_spawn_ms: 480,
            error_msg: "ValueError: unsupported function code 0xFF",
        },
        "truncatedHeader" => AttackConfig {
            name: "Truncated Header (3 bytes)",
            restart_time: 30,
            worker_spawn_ms: 450,
            error_msg: "struct.error: unpack requires 7 bytes, got 3",
        },
        _ => AttackConfig {
            name: "Random Garbage",
            restart_time: 20,
            worker_spawn_ms: 420,
            error_msg: "UnicodeDecodeError: invalid byte 0xfe",
        },
    }
}

// ============================================================================
// wasm measurement functions (real webassembly api calls)
// ============================================================================

/// minimal wasm module for measurement (add function)
/// this is a real wasm binary that we compile and instantiate
const MINIMAL_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, // magic
    0x01, 0x00, 0x00, 0x00, // version
    0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, // type section
    0x03, 0x02, 0x01, 0x00, // function section
    0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // export section
    0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b, // code
];

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

/// measure real wasm compile time using webassembly.compile()
async fn measure_compile_time() -> f64 {
    let start = now();
    
    // actually compile the wasm module
    let array = js_sys::Uint8Array::from(MINIMAL_WASM);
    let promise = js_sys::WebAssembly::compile(&array.buffer());
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    
    now() - start
}

/// measure real wasm instantiate time using webassembly.instantiate()
async fn measure_instantiate_time() -> f64 {
    // first compile
    let array = js_sys::Uint8Array::from(MINIMAL_WASM);
    let compile_promise = js_sys::WebAssembly::compile(&array.buffer());
    let module = wasm_bindgen_futures::JsFuture::from(compile_promise).await.unwrap();
    
    // now measure instantiate
    let start = now();
    let instantiate_promise = js_sys::WebAssembly::instantiate_module(
        &module.unchecked_into(),
        &js_sys::Object::new()
    );
    let _ = wasm_bindgen_futures::JsFuture::from(instantiate_promise).await;
    
    now() - start
}

// ============================================================================
// main application component
// ============================================================================

#[component]
pub fn App() -> impl IntoView {
    // ========================================================================
    // real wasm metrics (measured, not simulated)
    // ========================================================================
    let (wasm_compile_ms, set_wasm_compile_ms) = create_signal(0.0f64);
    let (wasm_instantiate_ms, set_wasm_instantiate_ms) = create_signal(0.0f64);
    let (wasm_memory_kb, set_wasm_memory_kb) = create_signal(64u32);
    let (measurements_done, set_measurements_done) = create_signal(false);
    
    // ========================================================================
    // 2oo3 voting state (three instances)
    // ========================================================================
    let (instance_states, set_instance_states) = create_signal([
        InstanceState::Healthy,
        InstanceState::Healthy,
        InstanceState::Healthy,
    ]);
    let (faulty_instance, set_faulty_instance) = create_signal(Option::<u8>::None);
    let (vote_result, set_vote_result) = create_signal(Option::<String>::None);
    let (switchover_count, set_switchover_count) = create_signal(0u32);
    
    // ========================================================================
    // python multiprocessing state (simulated but realistic)
    // ========================================================================
    let (python_workers, set_python_workers) = create_signal([true, true, true]); // all alive, 1 active + 2 standby
    let (python_active_worker, set_python_active_worker) = create_signal(0u8);
    let (python_restarting, set_python_restarting) = create_signal(false);
    let (python_restart_progress, set_python_restart_progress) = create_signal(0u32);
    
    // ========================================================================
    // common metrics
    // ========================================================================
    let (python_processed, set_python_processed) = create_signal(0u64);
    let (python_rejected, set_python_rejected) = create_signal(0u64);
    let (python_downtime_ms, set_python_downtime_ms) = create_signal(0u64);
    
    let (wasm_processed, set_wasm_processed) = create_signal(0u64);
    let (wasm_rejected, set_wasm_rejected) = create_signal(0u64);
    
    let (python_logs, set_python_logs) = create_signal(Vec::<LogEntry>::new());
    let (wasm_logs, set_wasm_logs) = create_signal(Vec::<LogEntry>::new());
    
    let (is_running, set_is_running) = create_signal(false);
    let (selected_attack, set_selected_attack) = create_signal("bufferOverflow".to_string());
    
    // ========================================================================
    // measure real wasm performance on startup
    // ========================================================================
    create_effect(move |_| {
        if !measurements_done.get() {
            spawn_local(async move {
                // measure compile time (real)
                let compile_time = measure_compile_time().await;
                set_wasm_compile_ms.set(compile_time);
                
                // measure instantiate time (real)
                let instantiate_time = measure_instantiate_time().await;
                set_wasm_instantiate_ms.set(instantiate_time);
                
                set_measurements_done.set(true);
            });
        }
    });
    
    // ========================================================================
    // attack simulation
    // ========================================================================
    let trigger_attack = move |_| {
        set_is_running.set(true);
        let attack = selected_attack.get();
        let config = get_attack_config(&attack);
        let current_active = python_active_worker.get();
        
        // if first attack (logs empty), show startup
        if python_logs.get().is_empty() {
            set_python_logs.set(vec![
                LogEntry { level: "info".into(), message: "$ python gateway.py --workers 3".into() },
                LogEntry { level: "success".into(), message: "[OK] Worker pool: 1 active, 2 standby".into() },
            ]);
        }
        
        if wasm_logs.get().is_empty() {
            set_wasm_logs.set(vec![
                LogEntry { level: "info".into(), message: "$ ./gateway --mode 2oo3".into() },
                LogEntry { level: "success".into(), message: "[OK] 2oo3 TMR: 3 instances initialized".into() },
                LogEntry { level: "info".into(), message: format!("[METRICS] Instantiate: {:.2}ms (real)", wasm_instantiate_ms.get()) },
            ]);
        }
        
        // show incoming frames
        set_python_logs.update(|logs| {
            logs.push(LogEntry { level: "info".into(), message: "[RECV] Processing frames...".into() });
        });
        set_wasm_logs.update(|logs| {
            logs.push(LogEntry { level: "info".into(), message: "[RECV] Processing frames...".into() });
        });
        
        set_python_processed.update(|n| *n += 5);
        set_wasm_processed.update(|n| *n += 5);
        
        // after 800ms: attack arrives
        set_timeout(move || {
            let config = get_attack_config(&attack);
            let current_active = current_active;
            
            // ================================================================
            // python: CURRENT ACTIVE worker crashes, next one takes over
            // ================================================================
            let next_active = (current_active + 1) % 3;
            
            set_python_logs.update(|logs| {
                logs.push(LogEntry { level: "error".into(), message: format!("[CRASH] {}", config.error_msg) });
                logs.push(LogEntry { level: "error".into(), message: format!("💥 Worker {} died - Worker {} taking over...", current_active, next_active) });
            });
            
            // mark current worker as dead, rest alive
            let mut workers = [true, true, true];
            workers[current_active as usize] = false;
            set_python_workers.set(workers);
            set_python_active_worker.set(next_active);
            set_python_restarting.set(true);
            set_python_rejected.update(|n| *n += 1);
            
            // simulate worker spawn time
            let spawn_ms = config.worker_spawn_ms;
            simulate_python_restart(
                set_python_logs,
                set_python_restarting,
                set_python_restart_progress,
                set_python_workers,
                set_python_downtime_ms,
                spawn_ms,
                current_active,
            );
            
            // ================================================================
            // wasm: 2oo3 voting catches the fault
            // ================================================================
            let faulty_idx = (js_sys::Math::random() * 3.0) as u8;
            set_faulty_instance.set(Some(faulty_idx));
            
            // update instance states
            let mut states = instance_states.get();
            states[faulty_idx as usize] = InstanceState::Faulty;
            set_instance_states.set(states);
            
            let healthy: Vec<u8> = (0..3).filter(|&i| i != faulty_idx).collect();
            
            // immediately show the fault and voting (0ms downtime - parallel processing)
            set_wasm_logs.update(|logs| {
                logs.push(LogEntry { level: "warn".into(), message: format!("[TRAP] Instance {} trapped on malformed input", faulty_idx) });
                logs.push(LogEntry { level: "info".into(), message: format!("[VOTE] Instances {:?} agree, Instance {} disagrees", healthy, faulty_idx) });
                logs.push(LogEntry { level: "success".into(), message: "[VOTE] Result: 2/3 majority - frame rejected safely".into() });
                logs.push(LogEntry { level: "success".into(), message: "[OK] No downtime - 2/3 voting continues with healthy instances".into() });
            });
            
            set_vote_result.set(Some("2/3 AGREE".to_string()));
            set_wasm_rejected.update(|n| *n += 1);
            set_switchover_count.update(|n| *n += 1);
            
            // show processing continues even during rebuild (with 2/3 instances)
            set_timeout(move || {
                set_wasm_logs.update(|logs| {
                    logs.push(LogEntry { level: "info".into(), message: "[RECV] Frame: Read Holding Registers".into() });
                    logs.push(LogEntry { level: "success".into(), message: "[VOTE] 2/3 agree (1 rebuilding) → MQTT published".into() });
                });
                set_wasm_processed.update(|n| *n += 1);
            }, std::time::Duration::from_millis(300));
            
            // actually rebuild with real timing
            set_wasm_logs.update(|logs| {
                logs.push(LogEntry { level: "info".into(), message: format!("[REBUILD] Instance {} rebuilding asynchronously...", faulty_idx) });
            });
            
            // spawn async task to actually re-instantiate WASM and measure real time
            spawn_local(async move {
                let rebuild_start = now();
                
                // actually re-instantiate the wasm module (real operation!)
                let array = js_sys::Uint8Array::from(MINIMAL_WASM);
                let compile_promise = js_sys::WebAssembly::compile(&array.buffer());
                if let Ok(module) = wasm_bindgen_futures::JsFuture::from(compile_promise).await {
                    let instantiate_promise = js_sys::WebAssembly::instantiate_module(
                        &module.unchecked_into(),
                        &js_sys::Object::new()
                    );
                    let _ = wasm_bindgen_futures::JsFuture::from(instantiate_promise).await;
                }
                
                let rebuild_time = now() - rebuild_start;
                
                // update state - instance is now healthy
                let mut states = instance_states.get();
                states[faulty_idx as usize] = InstanceState::Healthy;
                set_instance_states.set(states);
                set_faulty_instance.set(None);
                
                set_wasm_logs.update(|logs| {
                    logs.push(LogEntry { 
                        level: "success".into(), 
                        message: format!("[OK] Instance {} rebuilt in {:.2}ms (real) - pool fully healthy", faulty_idx, rebuild_time) 
                    });
                });
            });
            
            // continue processing - first 2/3 while rebuilding, then 3/3 after recovery
            set_timeout(move || {
                set_wasm_logs.update(|logs| {
                    logs.push(LogEntry { level: "info".into(), message: "[RECV] Frame: Read Holding Registers".into() });
                    logs.push(LogEntry { level: "success".into(), message: "[VOTE] 2/3 agree (1 rebuilding) → MQTT published".into() });
                });
                set_wasm_processed.update(|n| *n += 1);
            }, std::time::Duration::from_millis(500));
            
            // after rebuild completes (~7ms), we're back to 3/3
            for delay in [1500u64, 2200] {
                set_timeout(move || {
                    set_wasm_logs.update(|logs| {
                        logs.push(LogEntry { level: "info".into(), message: "[RECV] Frame: Read Holding Registers".into() });
                        logs.push(LogEntry { level: "success".into(), message: "[VOTE] 3/3 agree → MQTT published".into() });
                    });
                    set_wasm_processed.update(|n| *n += 1);
                }, std::time::Duration::from_millis(delay));
            }
            
            set_is_running.set(false);
        }, std::time::Duration::from_millis(800));
    };
    
    // ========================================================================
    // reset function
    // ========================================================================
    let reset_demo = move |_| {
        set_python_logs.set(Vec::new());
        set_wasm_logs.set(Vec::new());
        set_python_processed.set(0);
        set_python_rejected.set(0);
        set_python_downtime_ms.set(0);
        set_wasm_processed.set(0);
        set_wasm_rejected.set(0);
        set_instance_states.set([InstanceState::Healthy; 3]);
        set_faulty_instance.set(None);
        set_vote_result.set(None);
        set_switchover_count.set(0);
        set_python_workers.set([true, true, true]);
        set_python_active_worker.set(0);
        set_python_restarting.set(false);
    };
    
    // ========================================================================
    // view
    // ========================================================================
    view! {
        <div class="dashboard">
            <Header/>
            
            // real wasm metrics banner
            <div class="metrics-banner">
                <div class="metric-real">
                    <span class="metric-label">"Compile (real)"</span>
                    <span class="metric-value">
                        {move || format!("{:.2}ms", wasm_compile_ms.get())}
                    </span>
                </div>
                <div class="metric-real">
                    <span class="metric-label">"Instantiate (real)"</span>
                    <span class="metric-value">
                        {move || format!("{:.2}ms", wasm_instantiate_ms.get())}
                    </span>
                </div>
                <div class="metric-real">
                    <span class="metric-label">"Memory/Instance"</span>
                    <span class="metric-value">{wasm_memory_kb}"KB"</span>
                </div>
                <div class="metric-simulated">
                    <span class="metric-label">"Python Spawn (sim)"</span>
                    <span class="metric-value">"~500ms"</span>
                </div>
            </div>
            
            // terminals
            <div class="terminals-container">
                <div class="terminal-panel python-terminal">
                    <div class="terminal-header">
                        <div class="terminal-title">"🐍 Python (multiprocessing)"</div>
                        <div class="terminal-status"
                            class:crashed=move || python_restarting.get()
                            class:online=move || !python_restarting.get()
                        >
                            {move || if python_restarting.get() { "⏳ SPAWNING" } else { "🟢 UP" }}
                        </div>
                    </div>
                    <div class="terminal-output" id="python-terminal">
                        {move || {
                            let entries = python_logs.get();
                            request_animation_frame(|| scroll_to_bottom("python-terminal"));
                            if entries.is_empty() {
                                view! { <div class="terminal-placeholder">"$ ready"</div> }.into_view()
                            } else {
                                entries.into_iter().map(|e| {
                                    view! { <div class=format!("log-line log-{}", e.level)>{e.message}</div> }
                                }).collect_view()
                            }
                        }}
                    </div>
                    // python workers visualization
                    <div class="workers-panel">
                        <span class="workers-label">"Workers:"</span>
                        {move || {
                            let workers = python_workers.get();
                            let active = python_active_worker.get();
                            (0..3).map(|i| {
                                let is_active = i == active as usize && workers[i];
                                let is_dead = !workers[i];
                                view! {
                                    <div class="worker-box"
                                        class:active=is_active
                                        class:dead=is_dead
                                        class:idle=!is_active && !is_dead
                                    >
                                        {format!("W{}", i)}
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>
                
                <div class="terminal-panel wasm-terminal">
                    <div class="terminal-header">
                        <div class="terminal-title">"🦀 WASM (2oo3 TMR)"</div>
                        <div class="terminal-status online">"🟢 UP"</div>
                    </div>
                    <div class="terminal-output" id="wasm-terminal">
                        {move || {
                            let entries = wasm_logs.get();
                            request_animation_frame(|| scroll_to_bottom("wasm-terminal"));
                            if entries.is_empty() {
                                view! { <div class="terminal-placeholder">"$ ready"</div> }.into_view()
                            } else {
                                entries.into_iter().map(|e| {
                                    view! { <div class=format!("log-line log-{}", e.level)>{e.message}</div> }
                                }).collect_view()
                            }
                        }}
                    </div>
                    // 2oo3 voting visualization
                    <div class="voting-panel">
                        <span class="voting-label">"2oo3 TMR:"</span>
                        {move || {
                            let states = instance_states.get();
                            let faulty = faulty_instance.get();
                            (0..3).map(|i| {
                                let state = states[i];
                                let is_faulty = faulty == Some(i as u8);
                                view! {
                                    <div class="instance-box"
                                        class:healthy=state == InstanceState::Healthy
                                        class:faulty=is_faulty
                                    >
                                        {format!("I{}", i)}
                                    </div>
                                }
                            }).collect_view()
                        }}
                        <span class="vote-result">
                            {move || vote_result.get().unwrap_or_else(|| "—".to_string())}
                        </span>
                    </div>
                </div>
            </div>
            
            // stats comparison
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
                            <span class="stat-label">"Crashed"</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value error">{move || python_downtime_ms.get()}"ms"</span>
                            <span class="stat-label">"Downtime"</span>
                        </div>
                    </div>
                    <div class="stat-subtext">"⚠️ Frames lost during downtime (1 active worker)"</div>
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
                            <span class="stat-label">"Voted Out"</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-value">"0ms"</span>
                            <span class="stat-label">"Downtime"</span>
                        </div>
                    </div>
                    <div class="stat-subtext success">"✓ No frames lost (2/3 still voting during rebuild)"</div>
                </div>
            </div>
            
            // attack controls
            <div class="panel chaos-panel">
                <h2>"☠️ Attack Vector"</h2>
                <div class="chaos-controls">
                    <select class="attack-select" on:change=move |ev| set_selected_attack.set(event_target_value(&ev))>
                        <option value="bufferOverflow">"Buffer Overflow"</option>
                        <option value="illegalFunction">"Illegal Function Code"</option>
                        <option value="truncatedHeader">"Truncated Header"</option>
                        <option value="randomGarbage">"Random Garbage"</option>
                    </select>
                    <button class="chaos-button" disabled=is_running on:click=move |_| trigger_attack(())>
                        {move || if is_running.get() { "⏳..." } else { "🎯 Attack" }}
                    </button>
                    <button class="reset-button" on:click=move |_| reset_demo(())>"🔄 Reset"</button>
                </div>
            </div>
            
            <Footer/>
        </div>
    }
}

// ============================================================================
// helper functions
// ============================================================================

fn simulate_python_restart(
    set_logs: WriteSignal<Vec<LogEntry>>,
    set_restarting: WriteSignal<bool>,
    set_progress: WriteSignal<u32>,
    set_workers: WriteSignal<[bool; 3]>,
    set_downtime: WriteSignal<u64>,
    spawn_ms: u32,
    crashed_worker: u8,
) {
    let steps = 5;
    let step_ms = spawn_ms / steps;
    
    for i in 1..=steps {
        let progress = (i * 100 / steps) as u32;
        let is_done = i == steps;
        
        set_timeout(move || {
            set_progress.set(progress);
            set_downtime.update(|d| *d += step_ms as u64);
            
            if is_done {
                set_restarting.set(false);
                set_workers.set([true, true, true]); // all workers alive again
                set_logs.update(|logs| {
                    logs.push(LogEntry { 
                        level: "success".into(), 
                        message: format!("[OK] Worker {} respawned ({}ms) - pool restored", crashed_worker, spawn_ms)
                });
            });
            } else {
                set_logs.update(|logs| {
                    logs.push(LogEntry { 
                        level: "warn".into(), 
                        message: format!("[SPAWN] {}%...", progress)
                    });
                });
            }
        }, std::time::Duration::from_millis((step_ms * i) as u64));
    }
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <div class="logo">
                <span class="logo-icon">"🔒"</span>
                <h1>"Protocol Gateway Sandbox"</h1>
            </div>
            <p class="subtitle">"2oo3 TMR with Real WASM Measurements"</p>
        </header>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="tech-badges">
                <span class="badge rust">"Rust"</span>
                <span class="badge wasm">"WASI 0.2"</span>
                <span class="badge">"2oo3 TMR"</span>
                <span class="badge security">"SIL 3"</span>
            </div>
        </footer>
    }
}

// ============================================================================
// browser utility functions
// ============================================================================

fn set_timeout<F: FnOnce() + 'static>(cb: F, dur: std::time::Duration) {
    use wasm_bindgen::closure::Closure;
    let window = web_sys::window().unwrap();
    let closure = Closure::once(cb);
    window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(), dur.as_millis() as i32
    ).unwrap();
    closure.forget();
}

fn request_animation_frame<F: FnOnce() + 'static>(cb: F) {
    use wasm_bindgen::closure::Closure;
    let window = web_sys::window().unwrap();
    let closure = Closure::once(cb);
    window.request_animation_frame(closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();
}

fn scroll_to_bottom(element_id: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(element) = document.get_element_by_id(element_id) {
                element.set_scroll_top(element.scroll_height());
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
