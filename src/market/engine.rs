// Market engine - drives the stock simulation based on hardware metrics

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use crate::hardware::metrics::HardwareState;
use crate::market::candle::Candle;
use rand::Rng;

/// Configuration for market simulation behavior
#[derive(Debug, Clone)]
pub struct MarketConfig {
    /// Initial starting price
    pub initial_price: f64,
    
    /// Minimum allowed price (floor)
    pub min_price: f64,
    
    /// Maximum allowed price (ceiling)  
    pub max_price: f64,
    
    /// How many candles to keep in history
    pub candle_history_count: usize,
    
    /// Weight for CPU load in price changes
    pub cpu_weight: f64,
    
    /// Weight for GPU utilization
    pub gpu_weight: f64,
    
    /// Weight for memory pressure (negative effect)
    pub memory_weight: f64,
    
    /// Weight for network traffic
    pub network_weight: f64,
    
    /// Weight for temperature deviation
    pub temp_weight: f64,
    
    /// Base noise level in price movement
    pub noise_level: f64,
    
    /// Volatility multiplier based on conditions
    pub volatility_base: f64,
}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            initial_price: 50.0,
            min_price: 45.0,
            max_price: 80.0,
            candle_history_count: 20,
            
            // CPU has highest influence (positive and negative)
            cpu_weight: 0.35,
            
            // GPU contributes to volatility  
            gpu_weight: 0.22,
            
            // Memory pressure can create downward pressure
            memory_weight: -0.15,
            
            // Network activity adds trading volume/volatility
            network_weight: 0.17,
            
            // Temperature stress (high temp = high volatility)
            temp_weight: 0.18,
            
            // Base noise to prevent market from being too predictable
            noise_level: 0.12,
            
            // Base volatility factor
            volatility_base: 1.0,
        }
    }
}

impl MarketConfig {
    /// Create config from TOML-like table structure
    pub fn from_weights(cpu: f64, gpu: f64, temp: f64, traffic: f64, time: f64) -> Self {
        // Normalize weights so they sum to roughly 1.0
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
                volatility_base: 1.0,
            }
        } else {
            MarketConfig::default()
        }
    }
}

/// Source of price movement and its magnitude
#[derive(Debug, Clone)]
pub struct PriceChangeSource {
    /// What caused this change (cpu, gpu, temp, etc.)
    pub source: PriceChangeSourceKind,
    /// How much this contributed to the change
    pub contribution: f64,
}

/// Type of factor causing price movement
#[derive(Debug, Clone)]
pub enum PriceChangeSourceKind {
    CpuLoad(f64),       // CPU utilization percentage
    GpuUtilization(f64), // GPU usage percentage
    MemoryPressure(f64), // Memory used as percentage  
    NetworkTraffic(f64), // Network throughput KB/s
    Temperature(f64),   // Temperature deviation from baseline
    Noise,              // Random noise component
}

/// Market engine that simulates trading based on hardware
pub struct MarketEngine {
    /// Current market price
    pub current_price: f64,
    
    /// Previous close (for candle formation)
    prev_close: f64,
    
    /// Configuration
    config: MarketConfig,
    
    /// Historical candles
    candles: Vec<Candle>,
    
    /// Track time for next update
    last_update_time: Option<DateTime<Utc>>,
    update_interval_ms: u64,
}

impl Default for MarketEngine {
    fn default() -> Self {
        MarketEngine::new(MarketConfig::default())
    }
}

impl MarketEngine {
    /// Create a new market engine with given configuration
    pub fn new(config: MarketConfig) -> Self {
        let initial = config.initial_price;
        
        MarketEngine {
            current_price: initial,
            prev_close: initial,
            config,
            candles: Vec::new(),
            last_update_time: None,
            update_interval_ms: 2500, // Update every ~2.5 seconds
        }
    }

    /// Initialize with a starting price from hardware state
    pub fn init_with_state(state: &HardwareState) -> Result<Self> {
        let config = MarketConfig::default();
        
        // Use CPU load to determine initial direction/momentum
        let start_price = if cfg!(target_os = "linux") {
            state.cpu_load_avg_1m.unwrap_or(1.0) * 5.0 + config.initial_price - 25.0
        } else {
            config.initial_price
        };
        
        Ok(MarketEngine {
            current_price: start_price.clamp(config.min_price, config.max_price),
            prev_close: start_price,
            config,
            candles: Vec::new(),
            last_update_time: None,
            update_interval_ms: 2500,
        })
    }

    /// Process a new hardware state and form a candle if needed
    pub fn process_state(&mut self, state: &HardwareState) -> Result<()> {
        let now = Utc::now();
        
        // Check if enough time has passed for new candle (configurable interval)
        if self.last_update_time.is_some() 
           && now.signed_duration_since(self.last_update_time.unwrap()).num_milliseconds() < self.update_interval_ms as i64
        {
            return Ok(());
        }

        self.last_update_time = Some(now);

        // Calculate price movement from hardware metrics
        let mut change_factors: Vec<PriceChangeSource> = Vec::new();
        
        // CPU load (positive influence)
        if let Some(cpu_load) = state.cpu_load_avg_1m {
            let cpu_factor = cpu_factor_from_load(cpu_load, self.config.cpu_weight);
            change_factors.push(PriceChangeSource {
                source: PriceChangeSourceKind::CpuLoad(cpu_load),
                contribution: cpu_factor,
            });
        }

        // GPU utilization (volatile)
        if let Some(gpu_util) = state.gpu_utilization_percent {
            let gpu_factor = gpu_factor_from_util(gpu_util, self.config.gpu_weight);
            change_factors.push(PriceChangeSource {
                source: PriceChangeSourceKind::GpuUtilization(gpu_util),
                contribution: gpu_factor,
            });
        }

        // Memory pressure (negative influence)
        if let Some(mem_percent) = state.memory_percent_used {
            let mem_factor = memory_pressure_factor(mem_percent, self.config.memory_weight);
            change_factors.push(PriceChangeSource {
                source: PriceChangeSourceKind::MemoryPressure(mem_percent),
                contribution: mem_factor,
            });
        }

        // Network traffic
        if let Some(net_up) = state.network_upload_throughput {
            let net_factor = network_traffic_factor(net_up, self.config.network_weight);
            change_factors.push(PriceChangeSource {
                source: PriceChangeSourceKind::NetworkTraffic(net_up),
                contribution: net_factor,
            });
        }

        // Temperature (stress = volatility)
        if let Some(temp) = state.cpu_temperature_celsius {
            let temp_factor = temperature_factor(temp, self.config.temp_weight);
            change_factors.push(PriceChangeSource {
                source: PriceChangeSourceKind::Temperature(temp),
                contribution: temp_factor,
            });
        }

        // Random noise (prevents overly predictable market)
        let noise = rand::random::<f64>() * 2.0 - 1.0; // [-1, 1]
        change_factors.push(PriceChangeSource {
            source: PriceChangeSourceKind::Noise,
            contribution: self.config.noise_level * noise,
        });

        // Apply all factors to calculate new price
        let total_change: f64 = change_factors.iter().map(|c| c.contribution).sum();
        
        // Clamp price changes
        let max_change_per_candle = (self.config.max_price - self.config.min_price) / 3.0;
        let clamped_change = total_change.clamp(-max_change_per_candle, max_change_per_candle);

        // Form new OHLC values for this candle
        let close = self.current_price + clamped_change;
        
        // Calculate realistic wicks and open/close
        let open = self.current_price;
        let high = (open.max(close) + rand::random::<f64>() * 2.0).clamp(
            self.config.min_price, 
            self.config.max_price
        );
        let low = (open.min(close) - rand::random::<f64>() * 2.0).clamp(
            self.config.min_price, 
            self.config.max_price
        );

        // Create new candle with valid OHLC
        let new_candle = Candle::new(now, open, close, high, low, 100);
        
        // Add to history (maintain recent candles)
        self.candles.push(new_candle.clone());
        
        if self.candles.len() > self.config.candle_history_count {
            self.candles.remove(0);
        }

        // Update current and previous close for next iteration
        self.current_price = new_candle.close;
        self.prev_close = new_candle.open;

        Ok(())
    }

    /// Get current price
    pub fn get_current_price(&self) -> f64 {
        self.current_price
    }

    /// Get candle history
    pub fn candles(&self) -> &[Candle] {
        &self.candles
    }

    /// Format market state for display
    pub fn format_market(&self) -> String {
        let mut s = String::new();
        
        s.push_str(&format!("Current Price: ${:.2}\n", self.current_price));
        s.push_str(&format!("Previous Close: ${:.2}\n", self.prev_close));
        
        let change = self.current_price - self.prev_close;
        let percent_change = (change / self.prev_close) * 100.0;
        
        s.push_str(&format!(
            "Change: {:+.3} ({:+.2}%)\n",
            change, percent_change
        ));

        if self.is_bullish() {
            s.push_str("Direction: 🟢 UP\n");
        } else {
            s.push_str("Direction: 🔴 DOWN\n");
        }

        // Volatility info
        let session_high = self.candles.iter().map(|c| c.high).fold(f64::NAN, f64::max);
        let session_low = self.candles.iter().map(|c| c.low).fold(f64::NAN, f64::min);

        s.push_str(&format!("Session High: ${:.2}\n", session_high));
        s.push_str(&format!("Session Low: ${:.2}\n", session_low));
        
        // Volume simulation (based on activity)
        let simulated_volume = self.candles.iter().map(|c| c.volume).sum();
        s.push_str(&format!("Volume: {} ticks\n", simulated_volume));

        s
    }

    /// Check current market direction
    fn is_bullish(&self) -> bool {
        self.current_price > self.prev_close + 0.01
    }
}

/// Calculate price contribution from CPU load
fn cpu_factor_from_load(load: f64, weight: f64) -> f64 {
    // Normalize load (typically 0-4 on Linux multi-core) to [0, 1]
    let normalized = load / 4.0;
    
    // Sigmoid-like function that makes high CPU both good and bad
    // High sustained load = market activity (+), but extreme = instability (-)
    let optimal_load = 2.0; // Optimal around 50% utilization
    
    let distance_from_optimal = (normalized * 4.0 - optimal_load).abs();
    let distance_penalty = if distance_from_optimal < 1.0 {
        -distance_from_optimal * weight
    } else {
        0.0
    };

    // Base positive influence from CPU activity
    normalized.abs() * weight + distance_penalty
}

/// Calculate price contribution from GPU utilization  
fn gpu_factor_from_util(util: f64, weight: f64) -> f64 {
    // GPU = volatile but can add value
    (util as f64 / 100.0).abs() * weight
}

/// Memory pressure creates downward market pressure
fn memory_pressure_factor(percent: f64, weight: f64) -> f64 {
    // High memory usage = negative pressure
    -(percent - 50.0).max(0.0) / 100.0 * weight.abs()
}

/// Network traffic adds trading volume and some volatility
fn network_traffic_factor(throughput: f64, weight: f64) -> f64 {
    let kb_per_sec = throughput; // Already in appropriate units
    (kb_per_sec / 1024.0).abs() * weight
}

/// Temperature deviation from baseline increases volatility
fn temperature_factor(temp: f64, weight: f64) -> f64 {
    // Assume optimal temp around 45C for typical servers
    let base_temp = 45.0;
    let deviation = (temp - base_temp).abs() / 10.0;
    
    // High temp adds risk/volatility (can be positive or negative)
    deviation * weight * (rand::random::<f64>() * 2.0 - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_constraints() {
        let mut engine = MarketEngine::default();
        
        // Process multiple states to create candle history
        for _ in 0..5 {
            let state = HardwareState::default();
            engine.process_state(&state).unwrap();
        }

        // Verify all candles satisfy constraints
        for candle in engine.candles() {
            assert!(candle.high >= candle.open);
            assert!(candle.high >= candle.close);
            assert!(candle.low <= candle.open);
            assert!(candle.low <= candle.close);
            assert!(candle.high >= candle.low);
        }
    }

    #[test]
    fn test_bullish_detection() {
        let mut engine = MarketEngine::default();
        
        // Create bullish move
        let state = HardwareState {
            cpu_load_avg_1m: Some(3.0),
            ..Default::default()
        };
        engine.process_state(&state).unwrap();

        assert!(engine.is_bullish());
    }
}
