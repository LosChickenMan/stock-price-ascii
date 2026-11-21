// Hardware dashboard for displaying system metrics

use crate::hardware::collector::HardwareCollector;
use crate::market::candle::Candle;
use ratatui::{prelude::*, widgets::*};
use crossterm::style::Modifier;

/// Hardware metrics display widget
#[derive(Debug)]
pub struct HardwareDashboard {
    /// Render area
    area: Rect,
    
    /// Current hardware state
    state: crate::hardware::metrics::HardwareState,
    
    /// Recent candles for reference
    candles: Vec<Candle>,
}

impl HardwareDashboard {
    pub fn new(area: Rect, state: &crate::hardware::metrics::HardwareState, candles: &[Candle]) -> Self {
        HardwareDashboard {
            area,
            state: state.clone(),
            candles: candles.to_vec(),
        }
    }

    pub fn build_widget(&self) -> Widget {
        let state = &self.state;
        
        // Create a paragraph with hardware metrics
        let mut text = String::new();
        
        // Header
        text.push_str("═══════════════════════════════════════\n");
        text.push_str("🖥️  HARDWARE METRICS\n");
        text.push_str("═══════════════════════════════════════\n\n");
        
        // CPU Metrics
        text.push_str(&format!("CPU:\n"));
        
        if let Some(util) = state.cpu_utilization_percent {
            text.push_str(&format!("  Utilization: {:.1}% {}", util, 
                if util > 90.0 { "🔥 HIGH" } else if util < 20.0 { "❄️ LOW" } else { "→ OK" }
            ));
        } else {
            text.push_str("  Utilization: N/A\n");
        }
        
        // Memory Metrics
        text.push_str(&format!("\nMemory:\n"));
        
        if let Some(total) = state.memory_total_bytes {
            if let Some(used) = state.memory_used_bytes {
                let percent = used as f64 * 100.0 / total as f64;
                text.push_str(&format!(
                    "  Used: {} ({:.1}%)\n",
                    bytes_to_human_readable(used),
                    percent
                ));
                
                if percent > 90.0 {
                    text.push_str("  Status: 🟠 HIGH USAGE\n");
                } else if percent < 30.0 {
                    text.push_str("  Status: 🔵 LOW USAGE\n");
                } else {
                    text.push_str("  Status: → NORMAL\n");
                }
            } else {
                text.push_str("  Usage: N/A (total available)\n");
            }
        } else {
            text.push_str("  Memory: N/A\n");
        }
        
        // Network Metrics
        text.push_str(&format!("\nNetwork:\n"));
        
        if let Some(upload) = state.network_upload_throughput {
            if let Some(download) = state.network_download_throughput {
                text.push_str(&format!(
                    "  Upload: {:.1} KB/s\n",
                    upload / 1024.0
                ));
                text.push_str(&format!(
                    "  Download: {:.1} KB/s\n", 
                    download / 1024.0
                ));
                
                // Indicate activity level
                let total_throughput = (upload + download) / 1024.0;
                if total_throughput > 5000.0 {
                    text.push_str("  Status: 🔥 HEAVY TRAFFIC\n");
                } else if total_throughput < 100.0 {
                    text.push_str("  Status: ❄️ LIGHT TRAFFIC\n");
                } else {
                    text.push_str("  Status: → NORMAL\n");
                }
            } else {
                text.push_str("  Download: N/A\n");
            }
        } else {
            text.push_str("  Network: N/A\n");
        }
        
        // Temperature
        text.push_str(&format!("\nTemperature:\n"));
        
        if let Some(temp) = state.cpu_temperature_celsius {
            text.push_str(&format!("  CPU Temp: {:.1}°C", temp));
            
            if temp > 85.0 {
                text.push_str(" 🟠 WARNING\n");
            } else if temp < 40.0 {
                text.push_str(" 🔵 COLD\n");
            }
        } else {
            text.push_str("  CPU Temp: N/A (not supported on this system)\n");
        }
        
        // GPU
        text.push_str(&format!("\nGPU:\n"));
        
        if let Some(util) = state.gpu_utilization_percent {
            text.push_str(&format!(
                "  Utilization: {:.1}%", util,
                if util > 80.0 { " 🔥" } else if util < 20.0 { " ❄️" } else { " →" }
            ));
        } else {
            text.push_str("  GPU Utilization: N/A\n");
        }
        
        // Volatile indicator (market status)
        text.push_str(&format!("\n═══════════════════════════════════════\n"));
        text.push_str("Market Status: ");
        
        if let Some(last_candle) = self.candles.last() {
            if last_candle.is_bullish() {
                text.push_str("🟢 BULLISH");
            } else {
                text.push_str("🔴 BEARISH");
            }
        }
        
        text.push_str("\n═══════════════════════════════════════\n");
        
        Paragraph::new(text)
            .block(Block::default()
                .title("Hardware Dashboard")
                .borders(Borders::ALL))
    }
}

/// Convert bytes to human-readable format
fn bytes_to_human_readable(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB)
    } else {
        format!("{}", bytes)
    }
}
