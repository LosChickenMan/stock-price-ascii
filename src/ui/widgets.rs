// Utility widgets and common UI components

use ratatui::{prelude::*, widgets::*};

/// Create a bordered box with centered text
pub fn create_bordered_box(text: &str, title: &str) -> Paragraph {
    Paragraph::new(text)
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL))
}

/// Create a status indicator widget
pub fn create_status_indicator(status: &str) -> Paragraph {
    let style = match status.to_ascii_uppercase().as_str() {
        "OPEN" => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        "CLOSED" => Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::Yellow),
    };
    
    Paragraph::new(status).style(style)
}

/// Create a color-coded number display
pub fn create_number_display(value: f64, decimal_places: usize) -> Paragraph {
    let text = format!("{:.decimal_places}", value);
    Paragraph::new(&text)
        .style(Style::default().add_modifier(Modifier::BOLD))
}

/// Create a progress bar widget
pub fn create_progress_bar(current: f64, total: f64, color: Color) -> Widget {
    let percentage = (current / total * 100.0).min(100.0);
    
    ProgressBar::default()
        .percentage((percentage * 100.0) as u32)
        .style(Style::default().fg(color))
}

/// Create a gauge for CPU/metric visualization  
pub fn create_gauge(label: &str, value: f64, max_value: f64) -> Paragraph {
    let percentage = (value / max_value * 100.0).min(100.0);
    
    // Use █ characters for gauge bars based on percentage
    let bar_width = (percentage / 5.0) as usize;
    let empty_width = 20 - bar_width;
    
    let bars = "█".repeat(bar_width) + "░".repeat(empty_width);
    Paragraph::new(format!("│ {} │", bars))
        .style(Style::default().add_modifier(Modifier::BOLD))
}

/// Create a simple sparkline for price visualization
pub fn create_sparkline(points: &[f64], area: Rect) -> Widget {
    if points.is_empty() || points.len() < 2 {
        return Paragraph::new("No data").block(Block::default());
    }
    
    let min_price = points.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_price = points.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let price_range = (max_price - min_price).max(1.0);
    
    // Simple bar chart representation
    let mut display = String::new();
    let width = area.width as usize - 2;
    
    for x in 0..width.min(50) {
        let normalized_x = x as f64 / (width.min(50) as f64);
        let interpolated_price = min_price + (max_price - min_price) * normalized_x;
        
        // Find closest data point and draw bar
        if let Some(&price) = points.iter().find(|&&p| p >= interpolated_price).last() {
            let is_bullish = price > interpolated_price;
            
            if is_bullish {
                display.push('█');
            } else {
                display.push('▄');
            }
        }
    }
    
    Paragraph::new(display)
        .block(Block::default()
            .title("Price History")
            .borders(Borders::ALL))
}

/// Create a price level indicator (horizontal line at specific price)
pub fn create_price_indicator(price: f64, area: Rect) -> Widget {
    // Convert price to row in terminal space
    let row = (area.height as f64 - ((price - 45.0) / (80.0 - 45.0) * area.height as f64)) 
               as usize;
    
    if row < area.height {
        Line::from(vec![
            ("├─".to_string(), Span::styled(format!(" ${:.1}", price), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            (" ".to_string(), Span::raw("".repeat(area.width as usize - 3)))
        ])
        .block(Block::default())
    } else {
        Paragraph::new("")
    }
}
