// Candle struct - represents a single OHLC candlestick bar

use chrono::{DateTime, Utc};

/// Represents a single candlestick bar in the market simulation
#[derive(Debug, Clone)]
pub struct Candle {
    /// Timestamp when candle was recorded
    pub timestamp: DateTime<Utc>,
    
    /// Open price at start of candle period
    pub open: f64,
    
    /// Highest price during candle period  
    pub high: f64,
    
    /// Lowest price during candle period
    pub low: f64,
    
    /// Close price at end of candle period
    pub close: f64,
    
    /// Volume (trading activity simulated from hardware)
    pub volume: u64,
}

impl Candle {
    /// Create a new candle with valid OHLC values
    pub fn new(
        timestamp: DateTime<Utc>,
        open: f64,
        close: f64,
        high: f64,
        low: f64,
        volume: u64,
    ) -> Self {
        // Ensure OHLC constraints are satisfied
        Candle::normalize(open, close, high, low) {
            timestamp,
            open,
            close,
            high,
            low,
            volume,
        }
    }

    /// Normalize candle to satisfy: high >= open, high >= close, low <= open, low <= close, high >= low
    fn normalize(open: f64, close: f64, mut high: f64, mut low: f64) -> (f64, f64, f64, f64) {
        // Ensure high bounds
        if high < open {
            high = open;
        }
        if high < close {
            high = close;
        }
        
        // Ensure low bounds  
        if low > open {
            low = open;
        }
        if low > close {
            low = close;
        }
        
        // Ensure high >= low
        if high < low {
            high = low;
        }
        
        (open, close, high, low)
    }

    /// Calculate the body size
    pub fn body_size(&self) -> f64 {
        self.high.max(self.close) - self.low.min(self.open)
    }

    /// Determine if candle is bullish (green) or bearish (red)
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open - 0.001 // Allow tiny floating point tolerance
    }

    /// Get price direction
    pub fn direction(&self) -> &'static str {
        if self.is_bullish() {
            "up"
        } else {
            "down"
        }
    }

    /// Calculate candle body percentage relative to visible range
    pub fn body_ratio(&self, min_price: f64, max_price: f64) -> f64 {
        let range = (max_price - min_price).max(1.0);
        let price_range = self.high - self.low;
        (price_range / range * 100.0).min(100.0)
    }

    /// Calculate wick length on top
    pub fn upper_wick(&self, min_price: f64) -> f64 {
        let visible_high = self.high.max(min_price);
        visible_high - self.close
    }

    /// Calculate wick length on bottom  
    pub fn lower_wick(&self, max_price: f64) -> f64 {
        let visible_low = self.low.min(max_price);
        self.open - visible_low
    }

    /// Format for display
    pub fn format(&self, price_label_width: usize) -> String {
        let is_bullish = self.is_bullish();
        
        if (self.close - self.open).abs() < 0.01 {
            // Doji candle (open ~= close)
            format!(
                "Doji (${:.2} ~ ${:.2})",
                self.open, self.close
            )
        } else if is_bullish {
            format!("🟢 UP: ${:.2}", self.close)
        } else {
            format!("🔴 DOWN: ${:.2}", self.close)
        }
    }

    /// Generate a realistic-looking fake candle for testing/initialization
    pub fn from_price(price: f64, noise_factor: f64) -> Self {
        let rng = rand::random::<f64>() * 2.0 - 1.0; // Random in [-1, 1]
        
        let open = price + (rng * noise_factor).min(3.0);
        let change = (rand::random::<f64>() - 0.5) * noise_factor * 0.5;
        let close = open + change;
        
        let high = open.max(close) + (rand::random::<f64>() * 2.0).min(1.0);
        let low = open.min(close) - (rand::random::<f64>() * 2.0).min(1.0);
        
        Candle::new(
            Utc::now(),
            open,
            close,
            high,
            low,
            rand::random::<u64>() % 1000,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_constraints() {
        // Test that normalization enforces high >= open, close and low <= open, close
        let c = Candle::new(
            Utc::now(),
            50.0,
            52.0,
            48.0,  // High lower than open - should be fixed
            49.0,   // Low higher than close - should be fixed
            100,
        );

        assert!(c.high >= c.open);
        assert!(c.high >= c.close);
        assert!(c.low <= c.open);
        assert!(c.low <= c.close);
        assert!(c.high >= c.low);
    }

    #[test]
    fn test_bullish_detection() {
        let bullish = Candle::new(Utc::now(), 50.0, 52.0, 53.0, 49.0, 100);
        assert!(bullish.is_bullish());

        let bearish = Candle::new(Utc::now(), 52.0, 50.0, 53.0, 49.0, 100);
        assert!(!bearish.is_bullish());
    }
}
