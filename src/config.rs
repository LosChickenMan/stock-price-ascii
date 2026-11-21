use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub starting_price: f64,
    pub history_limit: usize,
    pub sample_interval_ms: u64,
    pub market_floor: f64,
    pub market_ceiling: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            starting_price: 50.0,
            history_limit: 72,
            sample_interval_ms: 2_500,
            market_floor: 1.0,
            market_ceiling: 100.0,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let candidates = [
            Path::new("config/candlestick.toml"),
            Path::new("candlestick.toml"),
        ];

        for path in candidates {
            if let Ok(contents) = fs::read_to_string(path) {
                if let Ok(config) = toml::from_str::<Config>(&contents) {
                    return config;
                }
            }
        }

        Self::default()
    }
}
