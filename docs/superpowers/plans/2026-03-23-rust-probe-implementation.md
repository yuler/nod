# Implementation Plan: Nod Rust Probe (Linux First)

**Date:** 2026-03-23  
**Target:** Linux (Primary), macOS (Future)  
**Goal:** Implement a lightweight background probe matching the `2026-03-23-nod-event-schema.md` spec.

## 1. Project Setup
- [ ] Initialize Cargo project (`cargo init`).
- [ ] Add dependencies for Linux:
    - `sysinfo`, `active-win-pos-rs`, `user-idle`, `battery`, `serde`, `serde_json`, `chrono`, `tokio`, `tracing`, `tracing-appender`.
- [ ] Setup Linux system dependencies (e.g., `libxcb`, `libdbus`).

## 2. Core Architecture
- [ ] **Data Models**: Define `Event` enums and structs with `serde` for JSONL output.
- [ ] **Collectors Trait**: Create a `MetricCollector` trait to abstract platform-specific logic.
    - `fn get_system_stats() -> TickData`
    - `fn get_active_window() -> Option<WindowData>`
    - `fn get_idle_time() -> u64`
- [ ] **Linux Implementation**: Implement the trait using Linux-specific sysfs/X11/Wayland calls.

## 3. Implementation Phases
### Phase 1: The Heartbeat (`tick`)
- Implement a 60s loop using `tokio::time`.
- Collect CPU, Mem, Disk, Uptime, and Power source.
- Validate JSON output matches spec.

### Phase 2: Activity Monitoring (`window` & `idle`)
- Implement a faster loop (e.g., 2s) to detect window changes.
- Implement idle time detection.

### Phase 3: Storage & Rotation
- Use `tracing-appender` to write to `data/<hostname>_YYYY-MM-DD.jsonl`.
- Ensure one JSON object per line (compact mode).

## 4. Test Strategy
### Unit Tests
- **Serialization Test**: Verify `Event` struct serializes to the exact keys (`cpu`, `mem`, `v`, etc.).
- **Format Test**: Verify CPU is `"12.5%"` and Mem is `"4.2GB/32GB"`.
- **Logic Test**: Verify `idle_secs` logic (e.g., if input is > threshold, state is idle).

### Integration Tests
- **File Rotation Test**: Mock the date and verify a new file is created.
- **Mock Collector Test**: Use a fake collector to ensure the event loop produces expected JSON sequences.

## 5. Linux Environment Setup (Command)
```bash
sudo apt install libxcb-ewmh-dev libxcb-randr0-dev libdbus-1-dev pkg-config
```
