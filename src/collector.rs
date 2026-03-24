use sysinfo::{System, Disks, Networks};
use battery::Manager;
use crate::models::Event;
use chrono::Utc;
use std::process::{Command, Stdio};
use tokio::process::Command as TokioCommand;

pub trait MetricCollector {
    fn collect_tick(&mut self) -> Event;
    fn get_active_window_event(&self) -> Option<Event>;
    fn get_hostname(&self) -> String;
}

pub struct LinuxCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
    hostname: String,
}

impl LinuxCollector {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());

        Self { sys, disks, networks, hostname }
    }

    fn get_cpu_usage(&mut self) -> String {
        self.sys.refresh_cpu_all();
        let usage = self.sys.global_cpu_usage();
        format!("{:.1}%", usage)
    }

    fn get_mem_usage(&mut self) -> String {
        self.sys.refresh_memory();
        let total = self.sys.total_memory();
        let used = self.sys.used_memory();
        format!("{:.1}GB/{:.1}GB", used as f64 / 1024.0 / 1024.0 / 1024.0, total as f64 / 1024.0 / 1024.0 / 1024.0)
    }

    fn get_disk_usage(&mut self) -> String {
        self.disks.refresh_list();
        let disk = self.disks.first(); 
        if let Some(d) = disk {
            let total = d.total_space();
            let available = d.available_space();
            let used = total - available;
            format!("{:.0}GB/{:.0}GB", used as f64 / 1024.0 / 1024.0 / 1024.0, total as f64 / 1024.0 / 1024.0 / 1024.0)
        } else {
            "N/A".to_string()
        }
    }

    fn get_network_type(&mut self) -> String {
        self.networks.refresh_list();
        for (interface_name, data) in &self.networks {
            if data.received() > 0 || data.transmitted() > 0 {
                if interface_name.starts_with("wl") || interface_name.contains("wifi") {
                    return "wifi".to_string();
                }
                if interface_name.starts_with("eth") || interface_name.starts_with("en") {
                    return "wired".to_string();
                }
            }
        }
        "none".to_string()
    }

    fn get_power_source(&self) -> String {
        if let Ok(manager) = Manager::new() {
            if let Ok(mut batteries) = manager.batteries() {
                if let Some(Ok(battery)) = batteries.next() {
                    match battery.state() {
                        battery::State::Charging | battery::State::Full => return "AC".to_string(),
                        _ => return "Battery".to_string(),
                    }
                }
            }
        }
        "AC".to_string()
    }

    fn get_idle_secs(&self) -> u64 {
        0
    }

    fn get_uptime(&self) -> String {
        let seconds = System::uptime();
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{} days, {:02}:{:02}", days, hours, minutes)
    }

    fn get_linux_active_window(&self) -> Option<(String, String)> {
        let output = Command::new("xprop")
            .args(["-root", "_NET_ACTIVE_WINDOW"])
            .output()
            .ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let id = stdout.split("#").nth(1)?.trim().split(",").next()?.trim();
        if id == "0x0" { return None; }

        let title_output = Command::new("xprop")
            .args(["-id", id, "_NET_WM_NAME", "WM_NAME"])
            .output()
            .ok()?;
        let title_stdout = String::from_utf8_lossy(&title_output.stdout);
        let title = title_stdout.split("=").nth(1)?.trim().trim_matches('"').to_string();

        let class_output = Command::new("xprop")
            .args(["-id", id, "WM_CLASS"])
            .output()
            .ok()?;
        let class_stdout = String::from_utf8_lossy(&class_output.stdout);
        let app_name = class_stdout.split("=").nth(1)?
            .split(",")
            .last()?
            .trim()
            .trim_matches('"')
            .to_string();

        Some((app_name, title))
    }

    pub fn spawn_window_spy() -> Option<tokio::process::Child> {
        TokioCommand::new("xprop")
            .args(["-spy", "-root", "_NET_ACTIVE_WINDOW"])
            .stdout(Stdio::piped())
            .spawn()
            .ok()
    }
}

impl MetricCollector for LinuxCollector {
    fn get_hostname(&self) -> String {
        self.hostname.clone()
    }

    fn collect_tick(&mut self) -> Event {
        Event::Tick {
            hostname: self.hostname.clone(),
            timestamp: Utc::now(),
            v: 1,
            network: self.get_network_type(),
            cpu: self.get_cpu_usage(),
            mem: self.get_mem_usage(),
            disk: self.get_disk_usage(),
            idle_secs: self.get_idle_secs(),
            uptime: self.get_uptime(),
            power: self.get_power_source(),
        }
    }

    fn get_active_window_event(&self) -> Option<Event> {
        self.get_linux_active_window().map(|(app, title)| {
            Event::Window {
                hostname: self.hostname.clone(),
                timestamp: Utc::now(),
                v: 1,
                window: app,
                title: Some(title),
            }
        })
    }
}
