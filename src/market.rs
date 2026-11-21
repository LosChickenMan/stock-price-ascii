use rand::Rng;

use crate::hardware::HardwareMetrics;

const MARKET_CENTER: f64 = 50.0;

#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Candle {
    fn new(open: f64, close: f64, extra_volatility: f64, floor: f64, ceiling: f64) -> Self {
        let spread = (open - close).abs() * 0.28 + extra_volatility;
        Self {
            open,
            close,
            high: (open.max(close) + spread).min(ceiling),
            low: (open.min(close) - spread).max(floor),
        }
    }

    pub fn bullish(&self) -> bool {
        self.close >= self.open
    }
}

pub struct Market {
    pub candles: Vec<Candle>,
    pub price: f64,
    pub change_percent: f64,
    pub high_24: f64,
    pub low_24: f64,
    floor: f64,
    ceiling: f64,
    history_limit: usize,
    activity_baseline: Option<f64>,
}

impl Market {
    pub fn new(starting_price: f64, floor: f64, ceiling: f64, history_limit: usize) -> Self {
        let floor = floor.min(ceiling - 1.0).max(0.01);
        let ceiling = ceiling.max(floor + 1.0);
        let price = starting_price.clamp(floor, ceiling);

        Self {
            candles: Vec::new(),
            price,
            change_percent: 0.0,
            high_24: price,
            low_24: price,
            floor,
            ceiling,
            history_limit: history_limit.max(12),
            activity_baseline: None,
        }
    }

    pub fn tick(&mut self, metrics: &HardwareMetrics) {
        let mut rng = rand::thread_rng();
        let open = self.price;

        // Build a 0..1 activity score from whichever real sensors are available.
        // Missing GPU/temperature sensors are omitted and the remaining weights
        // are re-normalized rather than being treated as zero.
        let activity = hardware_activity(metrics);

        // Track the host's own normal operating level. This means a quiet,
        // powerful server isn't permanently penalized for having low CPU usage.
        let baseline = *self.activity_baseline.get_or_insert(activity);
        self.activity_baseline = Some(baseline * 0.94 + activity * 0.06);

        // Revert gently toward 50 so the index remains centered over time.
        // This also actively pushes the price away from either hard boundary.
        let mean_reversion = (MARKET_CENTER - open) * 0.055;

        // Hardware affects direction, but only changes in activity produce a
        // strong impulse. Absolute load adds a smaller persistent influence.
        let activity_delta = activity - baseline;
        let activity_impulse = (activity_delta * 10.0).clamp(-1.4, 1.4);
        let activity_level_bias = ((activity - 0.30) * 0.45).clamp(-0.20, 0.30);
        let hardware_move = activity_impulse + activity_level_bias;

        // Use additive index points rather than percentage compounding. This
        // keeps volatility similar near 1 and near 100.
        let regular_noise = rng.gen_range(-4.2..=4.2);
        let event_spike = if rng.gen_bool(0.07) {
            rng.gen_range(-5.5..=5.5)
        } else {
            0.0
        };

        let raw_close = open + mean_reversion + hardware_move + regular_noise + event_spike;
        let close = self.reflect_at_bounds(raw_close);

        // Wicks represent intraperiod volatility around the real open/close.
        let wick_size = 0.35 + regular_noise.abs() * 0.22 + event_spike.abs() * 0.18;
        let candle = Candle::new(open, close, wick_size, self.floor, self.ceiling);

        self.price = close;
        self.change_percent = if open.abs() > f64::EPSILON {
            (close / open - 1.0) * 100.0
        } else {
            0.0
        };
        self.high_24 = self.high_24.max(candle.high);
        self.low_24 = self.low_24.min(candle.low);
        self.candles.push(candle);

        if self.candles.len() > self.history_limit {
            self.candles.remove(0);
        }
    }

    fn reflect_at_bounds(&self, value: f64) -> f64 {
        // Reflect overshoots back into the range instead of clamping and
        // allowing several candles to flatten against a boundary.
        let reflected = if value > self.ceiling {
            self.ceiling - (value - self.ceiling) * 0.65
        } else if value < self.floor {
            self.floor + (self.floor - value) * 0.65
        } else {
            value
        };

        reflected.clamp(self.floor, self.ceiling)
    }

    pub fn reset(&mut self, starting_price: f64) {
        self.candles.clear();
        self.price = starting_price.clamp(self.floor, self.ceiling);
        self.change_percent = 0.0;
        self.high_24 = self.price;
        self.low_24 = self.price;
        self.activity_baseline = None;
    }
}

fn hardware_activity(metrics: &HardwareMetrics) -> f64 {
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    add_signal(
        &mut weighted_sum,
        &mut total_weight,
        Some(metrics.cpu_percent / 100.0),
        0.32,
    );
    add_signal(
        &mut weighted_sum,
        &mut total_weight,
        Some(metrics.memory_percent / 100.0),
        0.22,
    );
    add_signal(
        &mut weighted_sum,
        &mut total_weight,
        metrics.gpu_percent.map(|value| value / 100.0),
        0.28,
    );

    let network_kbps = metrics.network_rx_kbps + metrics.network_tx_kbps;
    let network_score = (network_kbps.max(0.0).ln_1p() / 11.0).clamp(0.0, 1.0);
    add_signal(
        &mut weighted_sum,
        &mut total_weight,
        Some(network_score),
        0.10,
    );

    let hottest_temp = match (metrics.cpu_temperature_c, metrics.gpu_temperature_c) {
        (Some(cpu), Some(gpu)) => Some(cpu.max(gpu)),
        (Some(cpu), None) => Some(cpu),
        (None, Some(gpu)) => Some(gpu),
        (None, None) => None,
    };
    let thermal_score = hottest_temp.map(|temp| ((temp - 35.0) / 65.0).clamp(0.0, 1.0));
    add_signal(&mut weighted_sum, &mut total_weight, thermal_score, 0.08);

    if total_weight > 0.0 {
        (weighted_sum / total_weight).clamp(0.0, 1.0)
    } else {
        0.25
    }
}

fn add_signal(sum: &mut f64, weight_sum: &mut f64, value: Option<f64>, weight: f64) {
    if let Some(value) = value {
        if value.is_finite() {
            *sum += value.clamp(0.0, 1.0) * weight;
            *weight_sum += weight;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::HardwareMetrics;

    #[test]
    fn prices_always_stay_inside_configured_range() {
        let mut market = Market::new(50.0, 1.0, 100.0, 100);
        let metrics = HardwareMetrics::default();

        for _ in 0..20_000 {
            market.tick(&metrics);
            assert!((1.0..=100.0).contains(&market.price));
        }
    }

    #[test]
    fn quiet_server_does_not_collapse_to_the_floor() {
        let mut market = Market::new(50.0, 1.0, 100.0, 100);
        let metrics = HardwareMetrics {
            cpu_percent: 3.0,
            memory_percent: 25.0,
            ..HardwareMetrics::default()
        };

        let mut total = 0.0;
        let samples = 10_000;
        for _ in 0..samples {
            market.tick(&metrics);
            total += market.price;
        }

        let average = total / samples as f64;
        assert!(
            (35.0..=65.0).contains(&average),
            "unexpected long-run average: {average}"
        );
        assert!(market.price > 5.0, "market unexpectedly pinned near floor");
    }

    #[test]
    fn candles_have_valid_ohlc_ordering() {
        let mut market = Market::new(50.0, 1.0, 100.0, 100);
        let metrics = HardwareMetrics::default();

        for _ in 0..500 {
            market.tick(&metrics);
            let candle = market.candles.last().unwrap();
            assert!(candle.high >= candle.open.max(candle.close));
            assert!(candle.low <= candle.open.min(candle.close));
        }
    }
}
