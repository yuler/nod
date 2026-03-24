mod models;
mod collector;

use crate::collector::{LinuxCollector, MetricCollector};
use crate::models::Event;
use std::time::Duration;
use tracing::info;
use tracing_appender::non_blocking;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() {
    let collector = LinuxCollector::new();
    let hostname = collector.get_hostname();

    // Setup daily log rotation
    let file_appender = tracing_appender::rolling::daily("data", format!("{}_", hostname));
    let (non_blocking, _guard) = non_blocking(file_appender);
    
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .without_time()
        .with_level(false)
        .with_target(false)
        .compact()
        .init();

    info!(target: "nod", "{}", serde_json::to_string(&Event::Lifecycle {
        hostname: hostname.clone(),
        timestamp: chrono::Utc::now(),
        v: 1,
        action: "boot".to_string(),
    }).unwrap());

    let collector_shared = Arc::new(Mutex::new(collector));

    // 1. Tick Loop (60s)
    let tick_collector = Arc::clone(&collector_shared);
    tokio::spawn(async move {
        loop {
            let event = {
                let mut c = tick_collector.lock().await;
                c.collect_tick()
            };
            if let Ok(json) = serde_json::to_string(&event) {
                info!(target: "nod", "{}", json);
            }
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

    // 2. Window Monitoring Loop (Event-driven via xprop -spy)
    let window_collector = Arc::clone(&collector_shared);
    tokio::spawn(async move {
        if let Some(mut child) = LinuxCollector::spawn_window_spy() {
            let stdout = child.stdout.take().expect("Failed to open stdout");
            let mut reader = BufReader::new(stdout).lines();

            // Initial capture
            {
                let c = window_collector.lock().await;
                if let Some(event) = c.get_active_window_event() {
                    if let Ok(json) = serde_json::to_string(&event) {
                        info!(target: "nod", "{}", json);
                    }
                }
            }

            while let Ok(Some(_line)) = reader.next_line().await {
                // xprop -spy outputted a change, trigger collection
                let event = {
                    let c = window_collector.lock().await;
                    c.get_active_window_event()
                };

                if let Some(event) = event {
                    if let Ok(json) = serde_json::to_string(&event) {
                        info!(target: "nod", "{}", json);
                    }
                }
            }
        } else {
            // Fallback to polling if spy fails
            let mut last_window: Option<String> = None;
            loop {
                let event = {
                    let c = window_collector.lock().await;
                    c.get_active_window_event()
                };

                if let Some(Event::Window { window, .. }) = &event {
                    if Some(window.clone()) != last_window {
                        last_window = Some(window.clone());
                        if let Ok(json) = serde_json::to_string(&event) {
                            info!(target: "nod", "{}", json);
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    });

    // Wait forever
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
    
    info!(target: "nod", "{}", serde_json::to_string(&Event::Lifecycle {
        hostname: hostname.clone(),
        timestamp: chrono::Utc::now(),
        v: 1,
        action: "shutdown".to_string(),
    }).unwrap());
}
