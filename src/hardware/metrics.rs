// Hardware metrics - defines metric types and data structures

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

/// Hardware state structure for a single moment in time
#[derive(Debug, Clone)]
pub struct HardwareState {
    pub timestamp: DateTime<Utc>,
    
    // CPU metrics
    pub cpu_utilization_percent: Option<f64>,
    pub cpu_load_avg_1m: Option<f64>,
    pub cpu_load_avg_5m: Option<f64>,
    pub cpu_load_avg_15m: Option<f64>,
    
    // Memory metrics  
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub memory_percent_used: Option<f64>,
    
    // Network metrics (throughput in bytes/sec)
    pub network_upload_throughput: Option<f64>,
    pub network_download_throughput: Option<f64>,
    
    // Temperature (if supported)
    pub cpu_temperature_celsius: Option<f64>,
    
    // Disk metrics (if available)
    pub disk_read_bytes: Option<u64>,
    pub disk_write_bytes: Option<u64>,
    
    // GPU metrics (if available on Linux/nvidia-smi)
    pub gpu_utilization_percent: Option<f64>,
    
    // Volatile but tracked metrics
    pub last_metric_collection_time: DateTime<Utc>,
}

impl Default for HardwareState {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            
            cpu_utilization_percent: None,
            cpu_load_avg_1m: Some(0.0),
            cpu_load_avg_5m: Some(0.0),
            cpu_load_avg_15m: Some(0.0),
            
            memory_used_bytes: None,
            memory_total_bytes: None,
            memory_percent_used: None,
            
            network_upload_throughput: Some(0.0),
            network_download_throughput: Some(0.0),
            
            cpu_temperature_celsius: None,
            
            disk_read_bytes: None,
            disk_write_bytes: None,
            
            gpu_utilization_percent: None,
            
            last_metric_collection_time: Utc::now(),
        }
    }
}

impl HardwareState {
    /// Returns metrics that were successfully collected (not None)
    pub fn available_metrics(&self) -> usize {
        self.cpu_load_avg_1m.is_some() as usize
            + self.memory_total_bytes.is_some() as usize
            + self.network_upload_throughput.is_some() as usize
            + self.network_download_throughput.is_some() as usize
            + (self.cpu_temperature_celsius.is_some() as usize)
            + (self.gpu_utilization_percent.is_some() as usize)
    }

    /// Formats metrics for display (N/A for missing values)
    pub fn format_metrics(&self) -> String {
        let mut s = String::new();
        
        s.push_str(&format!(
            "CPU: {:.1}%\n",
            self.cpu_utilization_percent.unwrap_or(0.0)
        ));
        
        if let (Some(total), Some(used)) = (self.memory_total_bytes, self.memory_used_bytes) {
            s.push_str(&format!(
                "Memory: {} / {} ({:.1}%) {}\n",
                format_size(*total),
                format_size(*used),
                used as f64 * 100.0 / total as f64,
                if used.is_none() { "[N/A]" } else { "" }
            ));
        }
        
        s.push_str(&format!(
            "Network: {} KB/s (UP) / {} KB/s (DN)\n",
            format_throughput(self.network_upload_throughput.unwrap_or(0.0) / 1024.0),
            format_throughput(self.network_download_throughput.unwrap_or(0.0) / 1024.0),
        ));
        
        if let Some(temp) = self.cpu_temperature_celsius {
            s.push_str(&format!("Temperature: {:.1}°C\n", temp));
        } else {
            s.push_str("Temperature: N/A\n");
        }
        
        s
    }
}

/// Convert bytes to human-readable size
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{}", bytes)
    }
}

/// Convert throughput to KB/s display
fn format_throughput(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.1} B/s", bytes_per_sec)
    }
}

/// Error type for hardware metric collection failures
#[derive(Debug)]
pub enum MetricError {
    /// File system read error
    Io(std::io::Error),
    /// Metric source not available (e.g., /proc doesn't exist on some systems)
    NotFound(String),
    /// Failed to parse metric value
    ParseFailure(String),
    /// GPU metrics unavailable (no nvidia-smi, no device detected)
    GpuUnavailable,
}

impl std::fmt::Display for MetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricError::Io(e) => write!(f, "IO error: {}", e),
            MetricError::NotFound(path) => write!(f, "Metric source not found: {}", path),
            MetricError::ParseFailure(msg) => write!(f, "Parse failure: {}", msg),
            MetricError::GpuUnavailable => write!(f, "GPU metrics unavailable"),
        }
    }
}

impl std::error::Error for MetricError {}
