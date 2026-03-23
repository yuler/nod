use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "event")]
pub enum Event {
    #[serde(rename = "tick")]
    Tick {
        hostname: String,
        timestamp: DateTime<Utc>,
        v: u32,
        network: String,
        cpu: String,
        mem: String,
        disk: String,
        idle_secs: u64,
        uptime: String,
        power: String,
    },
    #[serde(rename = "network")]
    Network {
        hostname: String,
        timestamp: DateTime<Utc>,
        v: u32,
        prev_network: String,
        new_network: String,
    },
    #[serde(rename = "screen")]
    Screen {
        hostname: String,
        timestamp: DateTime<Utc>,
        v: u32,
        state: String,
        window: Option<String>,
    },
    #[serde(rename = "window")]
    Window {
        hostname: String,
        timestamp: DateTime<Utc>,
        v: u32,
        window: String,
        title: Option<String>,
    },
    #[serde(rename = "lifecycle")]
    Lifecycle {
        hostname: String,
        timestamp: DateTime<Utc>,
        v: u32,
        action: String,
    },
    #[serde(rename = "error")]
    Error {
        hostname: String,
        timestamp: DateTime<Utc>,
        v: u32,
        message: String,
        code: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tick_serialization() {
        let now = Utc::now();
        let event = Event::Tick {
            hostname: "test-host".to_string(),
            timestamp: now,
            v: 1,
            network: "wifi".to_string(),
            cpu: "12.5%".to_string(),
            mem: "4.5GB/32GB".to_string(),
            disk: "120GB/1TB".to_string(),
            idle_secs: 300,
            uptime: "2 days, 04:12:05".to_string(),
            power: "AC".to_string(),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let json: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(json["event"], "tick");
        assert_eq!(json["hostname"], "test-host");
        assert_eq!(json["v"], 1);
        assert_eq!(json["cpu"], "12.5%");
        assert_eq!(json["power"], "AC");
    }
}
