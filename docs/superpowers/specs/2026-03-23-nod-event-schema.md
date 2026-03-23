# Nod Event Schema Specification (v1.0)

**Project:** Nod (Endpoint Activity Probe)  
**Status:** Approved  
**Last Updated:** 2026-03-23

## 1. Overview
This document specifies the event schema for **Nod**, a lightweight system activity monitor. Each event is recorded as a single JSON line in a `.jsonl` file.

## 2. Common Fields (Global Header)
All events MUST include these top-level fields:

| Field | Type | Description |
|-------|------|-------------|
| `event` | `string` | The event type name (see Section 3). |
| `hostname` | `string` | The system's unique identifier. |
| `timestamp` | `string` | ISO 8601 UTC timestamp (e.g., `2026-03-23T06:00:00Z`). |
| `v` | `number` | Schema version (current: `1`). |

---

## 3. Event Catalog

### `tick` (Heartbeat & Resource Metrics)
Sent periodically (default: 60s) to report system health and status.

| Field | Type | Description |
|-------|------|-------------|
| `network` | `enum` | Connection type: `["wired", "wifi", "none", "unknown"]`. |
| `cpu` | `string` | CPU utilization (e.g., `"12.5%"`). |
| `mem` | `string` | Memory usage (e.g., `"4.2GB/32GB"`). |
| `disk` | `string` | Disk usage of the primary partition (e.g., `"45GB/500GB"`). |
| `idle_secs` | `number` | User idle time in seconds. |
| `uptime` | `string` | System uptime (e.g., `"2 days, 04:12"`). |
| `power` | `string` | Current power source (e.g., `"AC"`, `"Battery"`). |

**Example:**
```json
{
  "event": "tick",
  "hostname": "yule-aorus",
  "timestamp": "2026-03-23T14:00:05Z",
  "v": 1,
  "network": "wifi",
  "cpu": "12.5%",
  "mem": "4.5GB/32GB",
  "disk": "120GB/1TB",
  "idle_secs": 300,
  "uptime": "2 days, 04:12:05",
  "power": "AC"
}
```

### `network` (Connection Change)
Triggered when the primary interface type switches.

| Field | Type | Description |
|-------|------|-------------|
| `prev_network` | `string` | Previous network state. |
| `new_network` | `string` | Current network state. |

### `screen` (State Transition)
Captures lock, unlock, sleep, and wake transitions.

| Field | Type | Description |
|-------|------|-------------|
| `state` | `enum` | `["lock", "unlock", "sleep", "wake"]`. |
| `window` | `string` | (Optional) The active application name at transition. |

### `window` (Focus Change)
Sent when the user switches between applications or active windows.

| Field | Type | Description |
|-------|------|-------------|
| `window` | `string` | The binary/process name (e.g., `Visual Studio Code`). |
| `title` | `string` | (Optional) The title of the focused window. |

### `lifecycle` (Host & Probe Lifecycle)
Sent when the host machine boots or shuts down, or when the probe itself is managed.

| Field | Type | Description |
|-------|------|-------------|
| `action` | `enum` | `["boot", "shutdown", "relaunch"]`. |

### `error` (Internal Probe Error)
Sent when the probe encounters an issue collecting data or performing tasks.

| Field | Type | Description |
|-------|------|-------------|
| `message` | `string` | Error message description. |
| `code` | `string` | (Optional) Error code for easier filtering. |

---

## 4. Implementation Guidelines
- **Encoding:** Files MUST be encoded in UTF-8 without BOM.
- **Serialization:** Avoid pretty-printing; use compact JSON (one event per line).
- **Log Rotation:** Logs should be rotated daily. Suggested filename format: `data/<hostname>_<YYYY-MM-DD>.jsonl`.
- **Retention:** Keep local logs for at least 7 days before archiving or deleting.
- **Time Accuracy:** If possible, synchronize the system clock via NTP to ensure reliable telemetry.
