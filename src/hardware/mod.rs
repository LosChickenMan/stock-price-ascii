// Hardware metrics collection

use anyhow::{Context, Result};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct HardwareState {
    pub timestamp: DateTime<Utc>,
    pub cpu_utilization_percent: Option<f64>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub network_upload_throughput: Option<f64>,
    pub network_download_throughput: Option<f64>,
    pub cpu_temperature_celsius: Option<f64>,
}

impl Default for HardwareState {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            cpu_utilization_percent: None,
            memory_used_bytes: None,
            memory_total_bytes: None,
            network_upload_throughput: Some(0.0),
            network_download_throughput: Some(0.0),
            cpu_temperature_celsius: None,
        }
    }
}

pub struct HardwareCollector;

impl HardwareCollector {
    pub fn init_hardware() -> Result<Self> {
        Ok(HardwareCollector)
    }

    pub fn collect(state: &mut crate::hardware::HardwareState) -> Result<()> {
        // CPU load from /proc/loadavg
        if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if !parts.is_empty() {
                if let Ok(load) = parts[0].parse::<f64>() {
                    state.cpu_utilization_percent = Some((load * 100.0).min(100.0));
                }
            }
        }

        // Memory from /proc/meminfo
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let key = parts[0];
                    if let Ok(bytes) = parts[1].parse::<u64>() {
                        match key {
                            "MemTotal" => {
                                state.memory_total_bytes = Some(bytes);
                            }
                            "MemAvailable" | "MemFree" => {
                                if state.memory_used_bytes.is_none() {
                                    let total = state.memory_total_bytes.unwrap_or(0);
                                    let used = total.saturating_sub(bytes);
                                    state.memory_used_bytes = Some(used);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Temperature (if available)
        if let Ok(entries) = std::fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.exists() && path.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(temp) = content.trim().parse::<f64>() {
                            state.cpu_temperature_celsius = Some(temp);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl HardwareState {
    pub fn collect() -> Result<Self> {
        let mut state = Self::default();
        hardware::collector::collect(&mut state)?;
        Ok(state)
    }
}

mod collector {
    use super::*;

    pub fn collect(state: &mut HardwareState) -> Result<()> {
        // Same as above but with proper function signature
        if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if !parts.is_empty() {
                if let Ok(load) = parts[0].parse::<f64>() {
                    state.cpu_utilization_percent = Some((load * 100.0).min(100.0));
                }
            }
        }

        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let key = parts[0];
                    if let Ok(bytes) = parts[1].parse::<u64>() {
                        match key {
                            "MemTotal" => {
                                state.memory_total_bytes = Some(bytes);
                            }
                            "MemAvailable" | "MemFree" => {
                                if state.memory_used_bytes.is_none() {
                                    let total = state.memory_total_bytes.unwrap_or(0);
                                    let used = total.saturating_sub(bytes);
                                    state.memory_used_bytes = Some(used);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if let Ok(entries) = std::fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.exists() && path.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(temp) = content.trim().parse::<f64>() {
                            state.cpu_temperature_celsius = Some(temp);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
