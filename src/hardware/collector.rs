// Hardware collector - reads metrics from system files

use anyhow::{Context, Result};
use chrono::Utc;
use crate::hardware::metrics::{HardwareState, MetricError};
use std::io;

/// Main hardware collector that gathers all available metrics
pub struct HardwareCollector {
    cpu_load_avg_path: String,
    meminfo_path: String,
}

impl HardwareCollector {
    /// Initialize the hardware collector
    pub fn init_hardware() -> Result<Self> {
        // On Linux, use default paths. On other platforms, return defaults.
        let cpu_path = if cfg!(target_os = "linux") {
            "/proc/loadavg".to_string()
        } else {
            String::new()
        };
        
        let mem_path = if cfg!(target_os = "linux") {
            "/proc/meminfo".to_string()
        } else {
            String::new()
        };
        
        Ok(HardwareCollector { cpu_load_avg_path, meminfo_path })
    }

    /// Collect all hardware metrics for the current moment
    pub fn collect(&self) -> Result<HardwareState> {
        let mut state = HardwareState::default();
        state.last_metric_collection_time = Utc::now();
        
        // CPU metrics (Linux /proc/loadavg)
        if !self.cpu_load_avg_path.is_empty() {
            self.read_cpu_load(&mut state).map_err(|e| {
                eprintln!("Warning: Could not read CPU load: {}", e);
                anyhow!(MetricError::NotFound(self.cpu_load_avg_path.clone()))
            })?;
        } else if cfg!(windows) {
            // On Windows, use WMI or PowerShell (sysinfo crate handles this)
            self.read_windows_cpu(&mut state)?;
        }
        
        // Memory metrics (Linux /proc/meminfo)
        if !self.meminfo_path.is_empty() {
            self.read_memory(&mut state).map_err(|e| {
                eprintln!("Warning: Could not read memory: {}", e);
                anyhow!(MetricError::NotFound(self.meminfo_path.clone()))
            })?;
        } else if cfg!(windows) {
            // On Windows, sysinfo will handle this
            self.read_windows_memory(&mut state)?;
        }
        
        // Network throughput (calculate from difference)
        self.read_network_throughput(&mut state).context("Failed to read network")?;
        
        // Temperature
        self.read_temperature(&mut state).map_err(|e| {
            eprintln!("Warning: Could not read temperature: {}", e);
            anyhow!(MetricError::NotFound("/sys/class/thermal".to_string()))
        })?;
        
        // Disk metrics (sysinfo handles most platforms)
        self.read_disk(&mut state)?;
        
        // GPU metrics (Linux nvidia-smi or sysinfo)
        if cfg!(target_os = "linux") {
            self.read_gpu_linux(&mut state).context("GPU unavailable on this system")?;
        }
        
        Ok(state)
    }

    /// Read CPU load from /proc/loadavg (Linux)
    fn read_cpu_load(&self, state: &mut HardwareState) -> Result<()> {
        if let Ok(content) = std::fs::read_to_string(&self.cpu_load_avg_path) {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 1 {
                if let Ok(val) = parts[0].parse::<f64>() {
                    state.cpu_load_avg_1m = Some(val);
                    // Use 5-minute load as CPU utilization proxy
                    if parts.len() > 2 {
                        if let Ok(util) = parts[1].parse::<f64>() {
                            state.cpu_utilization_percent = Some((util * 100.0).min(100.0));
                        }
                    }
                } else {
                    state.cpu_load_avg_1m = Some(parts[0].parse().unwrap_or(0.0));
                }
            }
        }
        
        Ok(())
    }

    /// Read CPU load on Windows using sysinfo (falls back to default)
    fn read_windows_cpu(&self, state: &mut HardwareState) -> Result<()> {
        // sysinfo crate will handle Windows CPU metrics
        state.cpu_load_avg_1m = Some(0.0); // Will be updated by sysinfo
        Ok(())
    }

    /// Read memory from /proc/meminfo (Linux)
    fn read_memory(&self, state: &mut HardwareState) -> Result<()> {
        if let Ok(content) = std::fs::read_to_string(&self.meminfo_path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let key = parts[0];
                    let val_bytes: u64 = match parts[1].parse() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    match key {
                        "MemTotal" => {
                            if state.memory_total_bytes.is_none() {
                                state.memory_total_bytes = Some(val_bytes);
                            }
                        }
                        "MemAvailable" | "MemFree" => {
                            if state.memory_used_bytes.is_none() {
                                // Available or Free = Total - Used
                                let total = state.memory_total_bytes.unwrap_or(0);
                                if let Some(total) = total.checked_sub(val_bytes) {
                                    state.memory_used_bytes = Some(total);
                                    let percent = 100.0 - (val_bytes as f64 * 100.0 / total as f64);
                                    state.memory_percent_used = Some(percent.min(100.0));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// Read memory on Windows using sysinfo
    fn read_windows_memory(&self, state: &mut HardwareState) -> Result<()> {
        // Will be handled by sysinfo crate
        Ok(())
    }

    /// Calculate network throughput from recent samples
    fn read_network_throughput(&self, state: &mut HardwareState) -> Result<()> {
        // On Linux/Unix, we can read /proc/net/dev
        if cfg!(target_os = "linux") {
            self.read_linux_network()?;
        } else {
            // sysinfo will handle on other platforms
            let _ = sysinfo::SystemExt::refresh_system();
        }
        
        Ok(())
    }

    /// Read network from /proc/net/dev (Linux)
    fn read_linux_network(&self) -> Result<()> {
        let mut total_rx: u64 = 0;
        let mut total_tx: u64 = 0;
        
        if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && !parts[1].is_empty() {
                    // Skip interface name (first field)
                    let bytes_in: u64 = match parts[1].parse() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    
                    let bytes_out: u64 = match parts.len() > 9 {
                        true => match parts[9].parse() {
                            Ok(v) => v,
                            Err(_) => continue,
                        },
                        false => 0,
                    };
                    
                    total_rx += bytes_in;
                    total_tx += bytes_out;
                }
            }
        }
        
        // Note: For accurate throughput, we'd need to compare with previous sample
        // Here we just store the current values for rate calculation in main loop
        if let Some(rx) = total_rx {
            state.network_download_throughput = Some(0.0); // Placeholder - real implementation tracks delta
        }
        
        Ok(())
    }

    /// Read temperature from /sys/class/thermal
    fn read_temperature(&self, state: &mut HardwareState) -> Result<()> {
        let mut found_temp = false;
        
        for entry in std::fs::read_dir("/sys/class/thermal")? {
            let entry = entry.ok();
            if let Some(entry) = entry {
                let path = entry.path();
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                
                // Skip non-temperature files
                if !filename.starts_with("thermal_zone") && filename != "temp1_input" {
                    continue;
                }
                
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(temp) = content.trim().parse::<f64>() {
                        // First valid temperature found
                        if state.cpu_temperature_celsius.is_none() {
                            state.cpu_temperature_celsius = Some(temp);
                            found_temp = true;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Read disk metrics using sysinfo (cross-platform)
    fn read_disk(&self, state: &mut HardwareState) -> Result<()> {
        let _ = sysinfo::SystemExt::refresh_disks();
        // Disk reading requires tracking previous values for throughput
        // For now, mark as unavailable until proper implementation
        Ok(())
    }

    /// Read GPU metrics on Linux (nvidia-smi or /proc/driver/nvidia/queries)
    fn read_gpu_linux(&self, state: &mut HardwareState) -> Result<bool> {
        // Try nvidia-smi first (requires sudo usually, but we try without)
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .arg("--query-gpu=utilization.gpu")
            .arg("--format=no")
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if !text.trim().is_empty() {
                    if let Ok(util) = text.trim().parse::<f64>() {
                        state.gpu_utilization_percent = Some(util.min(100.0));
                        return Ok(true);
                    }
                }
            }
        }
        
        // Fallback: check for nvidia device files (no metrics, but GPU exists)
        if std::path::Path::new("/dev/nvidia0").exists() {
            return Ok(false); // GPU exists but we couldn't read utilization
        }
        
        Ok(false)
    }
}
