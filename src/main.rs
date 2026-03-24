mod models;
mod collector;

use crate::collector::{LinuxCollector, MetricCollector};
use crate::models::Event;
use std::time::Duration;
use tracing::info;
use tracing_appender::non_blocking;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let collector = LinuxCollector::new();
    let hostname = collector.get_hostname();

    // Setup daily log rotation
    let file_appender = tracing_appender::rolling::daily("logs", format!("{}_", hostname));
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

    let collector = Arc::new(Mutex::new(collector));

    // Tick Loop (5s)
    let tick_collector = Arc::clone(&collector);
    tokio::spawn(async move {
        loop {
            let event = {
                let mut c = tick_collector.lock().await;
                c.collect_tick()
            };
            if let Ok(json) = serde_json::to_string(&event) {
                info!(target: "nod", "{}", json);
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    // Window Monitoring Loop (2s)
    let window_collector = Arc::clone(&collector);
    let window_hostname = hostname.clone();
    tokio::spawn(async move {
        let mut last_window: Option<String> = None;
        loop {
            let event = {
                let c = window_collector.lock().await;
                c.get_active_window_event()
            };

            if let Some(Event::Window { window, title, .. }) = event {
                if Some(window.clone()) != last_window {
                    last_window = Some(window.clone());
                    let full_event = Event::Window {
                        hostname: window_hostname.clone(),
                        timestamp: chrono::Utc::now(),
                        v: 1,
                        window,
                        title,
                    };
                    if let Ok(json) = serde_json::to_string(&full_event) {
                        info!(target: "nod", "{}", json);
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
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
