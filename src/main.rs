// Professional Hardware Candlestick Terminal - ASCII Simple Version
use anyhow::Result;

fn main() -> Result<()> {
    println!("{}", GREEN);
    println!("╔══════════════════════════════════════════════════╗");
    println!("║         🕯️  Professional Hardware Terminal        ║");
    println!("║   Market Simulation: HARDWARE EXCHANGE            ║");
    println!("╚══════════════════════════════════════════════════╝{}{}", RESET, RESET);
    
    // Initialize market with starting price
    let mut current_price: f64 = 50.0;
    let prev_close: f64 = 48.0;
    
    loop {
        // Collect hardware metrics (CPU load proxy)
        if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
            if let Some(load_str) = content.split_whitespace().next() {
                if let Ok(cpu_load) = load_str.parse::<f64>() {
                    // Calculate price change based on CPU activity + noise
                    let noise: f64 = ((rand::random::<u32>() % 7) as f64).powi(2) / 100.0;
                    current_price = (current_price + cpu_load * 0.35 - noise).clamp(45.0, 80.0);
                }
            }
        }

        // Draw chart frame
        draw_frame(current_price, prev_close)?;
        
        std::thread::sleep(std::time::Duration::from_millis(2500));
    }
}

fn draw_frame(price: f64, prev_price: f64) -> Result<()> {
    println!("{}", GREEN);
    println!("╔══════════════════════════════════════════════════╗");
    
    if price > prev_price + 0.01 {
        println!("║  Direction: 🟢 UP                                 ║");
    } else if price < prev_price - 0.01 {
        println!("║  Direction: 🔴 DOWN                               ║");
    } else {
        println!("║  Direction: → STABLE                              ║");
    }
    
    println!("║                                                    ║");
    println!("║  Latest Price: ${:.2}                              ║", price);
    
    // Simple candlestick bars at price level
    for row in 0..=4 {
        let threshold = price - (row as f64) * 5.0;
        
        if threshold <= price {
            print!("║  {} ", threshold);
            let bar_len = ((price - threshold + 1.0).ceil() as usize).min(20);
            
            for _ in 0..bar_len {
                if row == 0 && price > prev_price {
                    print!("{}", GREEN);
                } else if row == 3 && price < prev_price {
                    print!("{}", RED);
                } else {
                    print!("█");
                }
            }
        }
        
        println!("{}", RESET);
    }
    
    println!("║                                                    ║");
    println!("║  Time intervals: 30m   35m   40m   45m           ║");
    println!("╚══════════════════════════════════════════════════╝");
    
    Ok(())
}

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";
