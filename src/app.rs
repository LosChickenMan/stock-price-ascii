use std::time::Instant;

use crate::{
    config::Config,
    hardware::{HardwareCollector, HardwareMetrics},
    market::Market,
};

pub struct App {
    pub market: Market,
    pub metrics: HardwareMetrics,
    pub paused: bool,
    pub sample_interval_ms: u64,
    pub samples: u64,
    pub started_at: Instant,

    hardware: HardwareCollector,
    last_sample_at: Instant,
    starting_price: f64,
}

impl App {
    pub fn new(config: Config) -> Self {
        let starting_price = config.starting_price;
        let market = Market::new(
            config.starting_price,
            config.market_floor,
            config.market_ceiling,
            config.history_limit,
        );

        Self {
            market,
            metrics: HardwareMetrics::default(),
            paused: false,
            sample_interval_ms: config.sample_interval_ms,
            samples: 0,
            started_at: Instant::now(),
            hardware: HardwareCollector::new(),
            last_sample_at: Instant::now(),
            starting_price,
        }
    }

    pub fn sample_hardware(&mut self) {
        let now = Instant::now();
        let elapsed = now
            .duration_since(self.last_sample_at)
            .as_secs_f64()
            .max(0.1);

        self.metrics = self.hardware.sample(elapsed);
        self.last_sample_at = now;
        self.samples = self.samples.saturating_add(1);
    }

    pub fn update_market(&mut self) {
        self.market.tick(&self.metrics);
    }

    pub fn reset_market(&mut self) {
        self.market.reset(self.starting_price);
    }
}
