// Candlestick chart rendering for Ratatui

use crate::market::candle::Candle;
use ratatui::{prelude::*, widgets::*};

/// Candlestick chart that renders OHLC bars in the terminal
#[derive(Debug, Clone)]
pub struct CandlestickChart {
    /// Current render area
    area: Rect,
    
    /// Historical candles to display
    candles: Vec<Candle>,
    
    /// Current price indicator
    current_price: f64,
    
    /// Y-axis price range
    price_min: f64,
    price_max: f64,
}

impl CandlestickChart {
    /// Create a new chart widget
    pub fn new() -> Self {
        CandlestickChart {
            area: Rect::default(),
            candles: Vec::new(),
            current_price: 50.0,
            price_min: 45.0,
            price_max: 80.0,
        }
    }

    /// Set the render area for the chart
    pub fn with_area(mut self, area: Rect) -> Self {
        self.area = area;
        self
    }

    /// Add candles to the chart history
    pub fn with_candles(mut self, candles: &[Candle]) -> Self {
        self.candles.extend(candles.iter().cloned());
        self
    }

    /// Set current price indicator
    pub fn with_current_price(mut self, price: f64) -> Self {
        self.current_price = price;
        self
    }

    /// Set visible price range
    pub fn with_price_range(mut self, min: f64, max: f64) -> Self {
        self.price_min = min;
        self.price_max = max;
        self
    }

    /// Build the actual widget for rendering
    pub fn build_widget(&self) -> Widget {
        let candle_count = self.candles.len().min(10); // Max 10 candles visible
        let width = self.area.width as usize - 2;
        
        if candle_count == 0 {
            return Paragraph::new("Loading chart...")
                .block(Block::default()
                    .title("Candlestick Chart")
                    .borders(Borders::ALL));
        }

        // Calculate Y-axis scaling
        let price_range = (self.price_max - self.price_min) as i32;
        let rows_per_dollar = if price_range > 0 {
            (self.area.height as f64 / price_range as f64).min(5.0)
        } else {
            1.0
        };

        // Build a visual representation of recent candles
        let mut display = String::new();
        
        for row in 0..(self.area.height - 2) {
            let y_level = self.price_max - (row as f64 * rows_per_dollar);
            
            print!("│");
            
            // Check if this level intersects any candle
            for candle in self.candles.iter().rev() {
                let is_bullish = candle.is_bullish();
                
                // Wick range check
                if y_level < candle.high && y_level > candle.low {
                    print!("│");
                } else if y_level >= candle.open && y_level <= candle.close 
                           || (y_level >= candle.close && y_level <= candle.open) {
                    
                    // Body portion - use color based on direction
                    let char = if is_bullish { '█' } else { '▄' };
                    print!("{}", if is_bullish { GREEN } else { RED }.to_string());
                }
            }
            
            // Price label every other row or at boundaries
            if row % 2 == 0 || row == 0 || row == self.area.height - 3 {
                let formatted_price = format!("{:.1}", y_level);
                print!(" {}", &formatted_price[formatted_price.len()-3..]);
            }
        }

        // Add X-axis time labels
        display.push('\n');
        
        // Build the final widget with proper formatting
        Paragraph::new(display)
            .block(Block::default()
                .title(format!("Current: ${:.2}", self.current_price))
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)))
    }
}

/// Grid line drawing utility
pub fn draw_grid_frame(frame: &mut ratatui::Frame, area: Rect, candles: &[Candle]) {
    let price_min = candles.first().map(|c| c.low).unwrap_or(45.0);
    let price_max = candles.last().map(|c| c.high).unwrap_or(80.0);
    
    frame.render_widget(Clear, area);
    
    // Draw horizontal grid lines
    let row_height = (area.height as f64 / ((price_max - price_min) as f64)).min(2.0);
    
    for y in 0..area.height {
        let price_level = price_max - y as f64 * row_height;
        
        if y % 3 == 0 || y == area.height - 1 || y == 0 {
            // Draw grid line and price label
            frame.render_widget(
                Paragraph::new(format!("─ ${:.1}", price_level))
                    .style(Style::default().fg(Color::DarkGray)),
                Rect { x: area.x + 2, y, width: area.width as usize - 4, height: 1 }
            );
        } else {
            // Empty row with left border only
            frame.render_widget(
                Paragraph::new("│").style(Style::default().fg(Color::DarkGray)),
                Rect { x: area.x + 2, y, width: 1, height: 1 }
            );
        }
    }
}
