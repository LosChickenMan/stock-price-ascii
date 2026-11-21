# 🕯️ Hardware Candlestick Terminal - Professional Edition

A professional terminal application that visualizes your computer's hardware performance as a stock market simulation. Your system becomes the marketplace!

## Features

### 📊 Candlestick Chart
- Real-time OHLC (Open, High, Low, Close) candlesticks
- Green bullish candles | Red bearish candles  
- Wicks showing price volatility
- Historical candle retention
- Professional dark terminal theme

### 🔧 Hardware Metrics Display
- **CPU**: Utilization percentage and load average
- **Memory**: Used/total with usage percentage
- **Network**: Upload/download throughput
- **Temperature**: CPU thermal readings (when available)
- **GPU**: GPU utilization (when nvidia-smi is available)

### 🎯 Market Simulation
- Fictional stock price driven by hardware activity  
- Configurable volatility and noise
- Professional Bloomberg-style terminal interface
- Dark theme with color-coded indicators

## Requirements

- Linux system (primary target, Windows/macOS supported via sysinfo crate)
- Rust toolchain (rustc 1.70+)
- Terminal with ANSI color support
- Modern terminal emulator (iTerm2, Alacritty, Kitty, xterm)

## Quick Start

### Installation

```bash
cd /mnt/ai/stock-price-ascii && cargo build --release
```

### Running the Application

**Terminal version (recommended):**

```bash
cd /mnt/ai/stock-price-ascii && cargo run --release
```

**Direct binary execution:**

```bash
cd /mnt/ai/stock-price-ascii && ./target/release/hardware-candlestick
```

### Configuration

Edit `config/candlestick.toml` to customize:

- Price range (min/max)
- Candle history length
- Weight for each hardware metric
- Noise level and volatility settings
- Update interval timing

## Usage Controls

When running the application, use keyboard controls:

| Key | Action |
|-----|--------|
| `Q` | Quit application |
| `P` | Pause/resume updates |
| `R` | Reset simulation to initial price |
| `+` | Increase update speed (faster refresh) |
| `-` | Decrease update speed (slower refresh) |

## Project Structure

```
/mnt/ai/stock-price-ascii/
├── src/
│   ├── main.rs             # Application entry point
│   ├── market/              # Market simulation engine
│   └── hardware/            # Hardware metrics collection
├── config/
│   └── candlestick.toml     # Configuration file
├── Cargo.toml               # Dependencies
├── README.md                # This file
└── target/release/          # Compiled binary
```

## Technology Stack

- **Rust** - Systems programming language
- **ratatui** - Terminal UI framework  
- **crossterm** - Cross-platform terminal input/output
- **chrono** - Time handling and UTC timestamps
- **rand** - Random number generation for market noise
- **anyhow** - Error handling
- **serde/toml** - Configuration parsing

## License

MIT License - Feel free to use, modify, and distribute.

---

**Enjoy watching your hardware performance visualized as a thriving stock market! 📈🎉**
