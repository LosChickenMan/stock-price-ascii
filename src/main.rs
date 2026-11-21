// Hardware Candlestick Chart - Terminal Visualization
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";  
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";

fn main() {
    // Get current CPU load as volatility score proxy
    let load_str = std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_else(|_| "0.50".to_string());
    let cpu_load: f64 = load_str.split_whitespace().next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.5);
    
    // Current hour as another factor  
    let current_hour = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO)
        .as_secs() as u32 % 24;
    
    // Calculate volatility score (simplified formula)
    let volatility = cpu_load + ((current_hour as f64) / 24.0 * 0.3);
    
    // Determine direction
    let is_up = volatility > 1.0;
    let display_color = if is_up { GREEN } else { RED };
    let direction_str = if is_up { "🟢 UP" } else { "🔴 DOWN" };
    
    println!("{}", CYAN);
    println!("╔══════════════════════════════════════════════════╗");
    println!("║         🕯️  Hardware Candlestick Chart             ║");
    println!("║              Real-time Volatility Index            ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    
    // Display metrics
    println!("System Metrics:");
    println!("  CPU Load:      {:.2} ({}%)", cpu_load, cpu_load * 100.0);
    println!("  Hour of Day:   {:2}:00", current_hour);
    println!();
    
    // Volatility calculation result
    println!("Volatility Calculation:");
    println!("  Formula: CPU(0.65) + HourFactor(0.35 * normalized)");
    println!("  Score:         {:.4}", volatility);
    let display_color = if is_up { GREEN } else { RED };
    println!("  Direction:     {}{}{}", display_color, direction_str, RESET);
    println!();
    
    // Candlestick chart visualization
    println!("📊 Simplified Chart:");
    println!("{:─<70}", "");
    println!("Time →   30m     35m     40m     45m     50m     55m     60m");
    println!("{}", "──────────────────────────────────────────────────────────────────────────────");
    
    // Show bars representing volatility level
    let bar_count = ((volatility - 0.3) as usize).min(8);
    for i in 0..bar_count {
        let pos = i % 2;
        if pos == 0 {
            print!("{} ", GREEN);
        } else {
            print!("██");
        }
    }
    println!("{}", RESET);
    
    println!("{}", "──────────────────────────────────────────────────────────────────────────────");
    
    // Current state indicator
    let status = if volatility > 1.0 { "RISING" } else { "FALLING" };
    println!("   {}→ {}", if is_up { GREEN } else { RED }, status);
    
    // Test colors
    println!();
    println!("🎨 Terminal Color Test:");
    println!("  Green: {}", GREEN);
    println!("  Red:   {}", RED);  
    println!("  Cyan:  {}", CYAN);
}
