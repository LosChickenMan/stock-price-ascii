// Market simulation engine

use chrono::{DateTime, Utc};
use rand::Rng;
use std::io;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Candle {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Candle {
    pub fn new(open: f64, close: f64, high: f64, low: f64) -> Self {
        let mut c = Candle {
            timestamp: Utc::now(),
            open,
            close,
            high,
            low,
        };

        // Ensure constraints: high >= open/close, low <= open/close, high >= low
        if c.high < c.open { c.high = c.open; }
        if c.high < c.close { c.high = c.close; }
        if c.low > c.open { c.low = c.open; }
        if c.low > c.close { c.low = c.close; }
        if c.low > c.high { c.low = c.high; }

        c
    }

    pub fn is_bullish(&self) -> bool {
        self.close >= self.open - 0.01
    }
}

#[derive(Debug, Clone)]
pub struct MarketConfig {
    pub initial_price: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub candle_history_count: usize,
    pub cpu_weight: f64,
    pub gpu_weight: f64,
    pub memory_weight: f64,
    pub network_weight: f64,
    pub temp_weight: f64,
    pub noise_level: f64,
}

#[derive(Debug, Clone)]
pub struct MarketEngine {
    pub current_price: f64,
    prev_close: f64,
    config: MarketConfig,
    candles: Vec<Candle>,
}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            initial_price: 50.0,
            min_price: 45.0,
            max_price: 80.0,
            candle_history_count: 20,
            cpu_weight: 0.35,
            gpu_weight: 0.22,
            memory_weight: -0.15,
            network_weight: 0.17,
            temp_weight: 0.18,
            noise_level: 0.12,
        }
    }
}

impl MarketConfig {
    pub fn from_weights(cpu: f64, gpu: f64, temp: f64, traffic: f64, time: f64) -> Self {
        let total = cpu + gpu + temp + traffic + time;
        
        if total > 0.0 {
            MarketConfig {
                initial_price: 50.0,
                min_price: 45.0,
                max_price: 80.0,
                candle_history_count: 20,
                cpu_weight: (cpu / total).min(1.0),
                gpu_weight: (gpu / total).min(0.3),
                memory_weight: -((traffic * 0.7) / total).min(0.5),
                network_weight: traffic / total,
                temp_weight: temp / total,
                noise_level: 0.12,
            }
        } else {
            Default::default()
        }
    }
}

impl Default for MarketEngine {
    fn default() -> Self {
        Self::new(MarketConfig::default())
    }
}

impl MarketEngine {
    pub fn new(config: MarketConfig) -> Self {
        let start = config.initial_price;
        MarketEngine {
            current_price: start,
            prev_close: start,
            config,
            candles: Vec::new(),
        }
    }

    pub fn current_price(&self) -> f64 {
        self.current_price
    }

    pub fn is_bullish(&self) -> bool {
        self.current_price > self.prev_close + 0.01
    }

    pub fn candles(&self) -> &[Candle] {
        &self.candles
    }

    pub fn process_state(&mut self, state: &crate::hardware::HardwareState) -> Result<()> {
        let mut rng = rand::thread_rng();
        
        // Calculate price movement based on CPU load (proxy from hardware state or noise)
        let cpu_noise = if cfg!(target_os = "linux") {
            // Try to read CPU load from /proc/loadavg
            if let Ok(load_str) = std::fs::read_to_string("/proc/loadavg") {
                let parts: Vec<&str> = load_str.split_whitespace().collect();
                parts.get(0).and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0) * 0.35
            } else {
                rng.gen_range(-0.2..=0.2)
            }
        } else {
            // Non-Linux systems: use pure noise
            rng.gen_range(-0.2..=0.2)
        };

        let noise = rng.gen_range(-self.config.noise_level..=self.config.noise_level);
        
        // Calculate new price with bounds checking
        let change = cpu_noise + noise;
        let new_price = self.current_price + change;
        
        let close = new_price.clamp(self.config.min_price, self.config.max_price);
        let open = self.prev_close;
        
        // Create candle with valid OHLC
        let high = (open.max(close) + rng.gen_range(0.0..=1.0)).min(self.config.max_price);
        let low = (open.min(close) - rng.gen_range(0.0..=1.0)).max(self.config.min_price);
        
        let candle = Candle::new(open, close, high, low);
        
        self.candles.push(candle);
        
        if self.candles.len() > self.config.candle_history_count {
            self.candles.remove(0);
        }
        
        self.prev_close = open;
        self.current_price = close;

        Ok(())
    }
}

// Hardware state for collecting system metrics
#[derive(Debug, Clone, Default)]
pub struct HardwareState {
    pub timestamp: DateTime<Utc>,
    pub cpu_utilization_percent: Option<f64>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
}

impl HardwareState {
    /// Collect system metrics (Linux /proc based)
    pub fn collect() -> Result<Self> {
        let mut state = Self::default();
        state.timestamp = Utc::now();

        // CPU load from /proc/loadavg (Linux only)
        if cfg!(target_os = "linux") {
            if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
                let parts: Vec<&str> = content.split_whitespace().collect();
                if !parts.is_empty() {
                    if let Ok(load) = parts[0].parse::<f64>() {
                        state.cpu_utilization_percent = Some((load * 100.0).min(100.0));
                    }
                }
            }
        }

        // Memory from /proc/meminfo (Linux only)  
        if cfg!(target_os = "linux") {
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
        }

        Ok(state)
    }
}
