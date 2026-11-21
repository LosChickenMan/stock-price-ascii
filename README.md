# 🕯️ Hardware Candlestick Chart

Real-time animated stock price visualization in your terminal!

## Features

- ✅ **Animated Terminal Chart** - Live updates every 3 seconds
- ✅ **System Metrics** - Uses CPU load to simulate stock movement  
- ✅ **Candlestick Style** - Visual bars showing price changes
- ✅ **Color-Coded Movement** - 🟢 UP (green) | 🔴 DOWN (red)
- ✅ **Y-Axis Dollars** - Price range $20-$80
- ✅ **X-Axis Time** - 5-minute intervals

## Quick Start

### Option 1: Run the Bash Script (Recommended for terminals)

```bash
cd /mnt/ai/stock-price-ascii && ./stock.sh
```

### Option 2: Run the Rust Binary (Better performance)

```bash
cd /mnt/ai/stock-price-ascii && cargo run --release
```

or if compiled:

```bash
cd /mnt/ai/stock-price-ascii && ./target/release/hardware-candlestick
```

## How It Works

The chart simulates stock price movement based on your system's CPU load from `/proc/loadavg`. Higher CPU load = higher "stock price" volatility.

### Components:

1. **Hardware Candlestick** (Rust binary at `src/main.rs`) - High-performance animation
2. **Stock Launcher** (Bash script at `stock.sh`) - Simple terminal version

## Volatility Algorithm

```
New Price = Previous Price + (CPU_Load × 0.35) - Noise
Price Range: $45.00 - $80.00
Update Interval: 3 seconds
```

## Requirements

- Linux system with `/proc` filesystem
- Terminal with ANSI color support
- No external dependencies for bash version
- Rust + cargo for binary version (optional)

## File Structure

```
/mnt/ai/stock-price-ascii/
├── src/main.rs           # Rust source code
├── Cargo.toml            # Rust project config
├── stock.sh              # Bash launcher script
├── README.md             # This file
└── target/release/       # Compiled binary
```

## Troubleshooting

### Colors not showing?

Make sure your terminal supports ANSI escape codes. Try:
```bash
echo -e "\033[32mGREEN\033[0m"  # Should print green text
```

### Blank screen after running?

This is normal! The animation scrolls in your terminal. Look for the chart lines.

### Compilation errors?

Run:
```bash
cd /mnt/ai/stock-price-ascii && cargo build --release
```

## License

MIT License - Feel free to use and modify!

---

**Enjoy watching your hardware's performance visualized as stock market data! 📈🎉**
