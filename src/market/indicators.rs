// Market indicators - calculates volatility, moving averages, and other metrics

use crate::market::candle::Candle;

/// Volatility calculator for market analysis
#[derive(Debug, Clone)]
pub struct VolatilityCalculator {
    /// Period for calculation (e.g., 14 days)
    pub period: usize,
    
    /// Whether to use absolute or true range
    pub method: RangeMethod,
}

impl Default for VolatilityCalculator {
    fn default() -> Self {
        Self {
            period: 10,
            method: RangeMethod::HighLow,
        }
    }
}

impl VolatilityCalculator {
    /// Calculate volatility (ATR - Average True Range) from candle history
    pub fn calculate_atr(&self, candles: &[Candle]) -> f64 {
        if candles.is_empty() {
            return 0.0;
        }

        // True range for each candle: max(high-low, |high-prev_close|, |low-prev_close|)
        let mut ranges: Vec<f64> = Vec::new();
        
        for i in 1..candles.len() {
            let prev = &candles[i - 1];
            let curr = &candles[i];
            
            let true_range = std::cmp::max(
                curr.high - curr.low,
                std::cmp::max(
                    curr.high - prev.close.abs(),
                    (prev.close - curr.low).abs()
                )
            );
            
            ranges.push(true_range);
        }

        // Average of recent true ranges
        if ranges.is_empty() {
            return 0.0;
        }
        
        let recent_ranges: Vec<f64> = ranges.iter().rev().take(self.period).copied().collect();
        recent_ranges.iter().sum::<f64>() / recent_ranges.len() as f64
    }

    /// Calculate volatility as percentage of price range
    pub fn volatility_percentage(&self, candles: &[Candle], current_price: f64) -> f64 {
        if candles.is_empty() {
            return 0.0;
        }

        let price_range = (candles.last().unwrap().high - candles.first().unwrap().low).abs();
        (price_range / current_price) * 100.0
    }
}

/// Range method for volatility calculation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RangeMethod {
    /// Simple high - low range
    HighLow,
    /// True range (highest of: high-low, |high-prev_close|, |low-prev_close|)
    TrueRange,
}

/// Market indicators wrapper
#[derive(Debug, Clone)]
pub struct MarketIndicators {
    /// Moving average period
    pub ma_period: usize,
    
    /// Volatility calculator
    volatility: VolatilityCalculator,
    
    /// Current RSI (simplified for this simulation)
    rsi_period: usize,
}

impl Default for MarketIndicators {
    fn default() -> Self {
        Self {
            ma_period: 5,
            volatility: VolatilityCalculator::default(),
            rsi_period: 3, // Simplified RSI period
        }
    }
}

impl MarketIndicators {
    /// Calculate moving average of candle closes
    pub fn calculate_ma(&self, candles: &[Candle]) -> Option<f64> {
        if candles.is_empty() || candles.len() < self.ma_period {
            return None;
        }

        let recent: Vec<&Candle> = candles.iter().rev().take(self.ma_period).collect();
        let sum: f64 = recent.iter().map(|c| c.close).sum();
        Some(sum / self.ma_period as f64)
    }

    /// Get current RSI (simplified calculation)
    pub fn calculate_rsi(&self, candles: &[Candle]) -> Option<f64> {
        if candles.len() < self.rsi_period + 1 {
            return None;
        }

        // Simplified RSI based on recent changes
        let gains: Vec<f64> = candles.iter().rev().take(self.rsi_period)
            .map(|c| (c.close - c.open).max(0.0))
            .collect();
        
        let losses: Vec<f64> = candles.iter().rev().take(self.rsi_period)
            .map(|c| (c.open - c.close).max(0.0))
            .collect();

        let avg_gain: f64 = gains.iter().sum::<f64>() / gains.len() as f64;
        let avg_loss: f64 = losses.iter().sum::<f64>() / losses.len() as f64;

        if avg_loss == 0.0 {
            return Some(100.0); // Perfect bullish
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (rs + 1.0)))
    }

    /// Get current volatility level
    pub fn get_volatility(&self, candles: &[Candle]) -> f64 {
        self.volatility.calculate_atr(candles)
    }

    /// Determine market trend direction
    pub fn trend_direction(&self, candles: &[Candle]) -> Option<&'static str> {
        if let Some(current_ma) = self.calculate_ma(candles) {
            if let Some(prev_candle) = candles.last() {
                let prev_ma = candles[..candles.len().min(5)]
                    .iter()
                    .map(|c| c.close)
                    .take(5)
                    .collect::<Vec<_>>()
                    .iter()
                    .sum::<f64>() / 5.0;

                if current_ma > prev_ma {
                    return Some("up");
                } else if current_ma < prev_ma {
                    return Some("down");
                }
            }
        }
        
        None
    }

    /// Format indicators for display
    pub fn format_indicators(&self, candles: &[Candle], current_price: f64) -> String {
        let mut s = String::new();
        
        if let Some(rsi) = self.calculate_rsi(candles) {
            s.push_str(&format!("RSI ({:.0}):\n", rsi));
            s.push_str(&format!("  RSI: {:.2}\n", rsi));
            
            // Overbought/oversold indications
            if rsi > 70.0 {
                s.push_str("  Status: 🟠 OVERBOUGHT\n");
            } else if rsi < 30.0 {
                s.push_str("  Status: 🔵 OVERSOLD\n");
            } else {
                s.push_str("  Status: → NORMAL\n");
            }
        }

        if let Some(atr) = self.get_volatility(candles) {
            s.push_str(&format!("ATR Volatility:\n"));
            s.push_str(&format!("  ATR: {:.4}\n", atr));
            s.push_str(&format!("  %Volatility: {:.2}%\n", 
                (candles.first().unwrap().high - candles.last().unwrap().low).abs() / current_price * 100.0
            ));
        }

        if let Some(trend) = self.trend_direction(candles) {
            s.push_str(&format!("\nTrend: {} {}\n", match trend {
                "up" => "🟢 UPTREND",
                "down" => "🔴 DOWNTREND",
                _ => "→ FLAT",
            }));
        }

        s.push_str("──────────────────────\n");
        
        s
    }
}
