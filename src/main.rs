// Hardware Candlestick Chart - Simplified Stable Version
use std::fs;
use rand;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const GRAY: &str = "\x1b[90m";
const YELLOW: &str = "\x1b[33m";

fn get_cpu_load() -> f64 {
    let content = match fs::read_to_string("/proc/loadavg") {
        Ok(s) => s,
        Err(_) => String::from("1"),
    };
    
    let first_field = content.split_whitespace().next().unwrap_or("1").to_string();
    first_field.parse::<f64>().unwrap_or(1.0)
}

fn main() {
    println!("{}", RESET);
    println!("╔══════════════════════════════════════════════════╗");
    println!("║         🕯️  Hardware Candlestick Chart            ║");
    println!("║       Real-time Volatility Index                  ║");
    println!("╚══════════════════════════════════════════════════╝{}{}", RESET, RESET);
    
    let mut price: f64 = 50.0;
    let mut prev_close: f64 = 48.0;
    
    loop {
        // Get system metrics
        let cpu_load = get_cpu_load();
        
        // Simulate OHLC candle formation
        let noise = ((rand::random::<u32>() % 7) as f64).powi(2) / 100.0;
        let change = (cpu_load * 0.35) - noise;
        
        // Calculate new price level
        let new_price = price.clamp(45.0, 80.0);
        
        // Form candlestick data
        let open = prev_close;
        let close = new_price;
        let high = (open.max(close) + change.abs().min(2.0)).max(45.0);
        let low = (open.min(close) - change.abs().min(2.0)).min(80.0);
        
        // Store for next iteration
        prev_close = close;
        price = new_price;
        
        // Draw chart frame with colors
        draw_candlestick(&Candle { open, high, low, close });
        
        clear_screen();
        
        std::thread::sleep(std::time::Duration::from_millis(2500));
    }
}

struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

fn draw_candlestick(candle: &Candle) {
    println!("{}", GREEN);
    println!("╔══════════════════════════════════════════════════╗");
    
    // Price range for Y-axis scaling
    let min_price = candle.low.max(45.0);
    let max_price = candle.high.min(80.0);
    
    // Draw Y-axis with price labels
    println!("║  {}┌───────────────────────────────────────────┐{} ║", GRAY, RESET);
    println!("║  {:?}Y-Axis: ${:.1} - ${:.1}", GRAY, min_price, max_price);
    
    // Draw horizontal grid with candle body/wick
    let price_range = (max_price - min_price).max(1.0);
    let rows = 10;
    let height_per_row = price_range / rows as f64;
    
    for row in 0..rows {
        let y_level = min_price + height_per_row * row as f64;
        
        print!("║  {}│", GRAY);
        
        // Check if this row intersects with candle body or wicks
        if y_level >= candle.low && y_level <= candle.high {
            // Draw wick (thin vertical lines at top/bottom)
            if y_level < candle.low || y_level > candle.high {
                print!("│");
            } else if (y_level >= candle.open && y_level <= candle.close) 
                    || (y_level >= candle.close && y_level <= candle.open) {
                // Draw candle body with color
                let is_up = candle.close >= candle.open;
                
                // Simple body representation
                print!("{}", if is_up { GREEN } else { RED });
            }
        }
        
        // Print price label on right side (every other row)
        if row % 2 == 0 {
            println!("{} ${:.1}", GRAY, y_level);
        } else {
            println!("{}", GRAY);
        }
    }
    
    // X-axis time intervals
    println!("║  {}├────────────────────────────────────────────┤{}", GRAY, RESET);
    for i in 0..6 {
        print!("║  {}│", GRAY);
        if i % 2 == 0 {
            print!(" │");
        } else {
            println!("{}", GRAY);
        }
        
        // Time labels at intervals
        if i % 3 == 0 {
            print!(" {}", 30 + i * 5);
        }
    }
    
    println!("║  {:?}└───────────────────────────────────────────────{}", YELLOW, RESET);
    
    // Legend
    println!("║  {}🟢 UP (Green) | 🔴 DOWN (Red)                  {}", GREEN, RESET);
    println!("╚══════════════════════════════════════════════════╝");
}

fn clear_screen() {
    print!("\x1b[2J\x1b[H"); // Clear screen and home cursor
}
