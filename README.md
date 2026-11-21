# 🕯️ Candlestick Terminal Visualization - Rust Project

Real-time terminal-based ASCII art visualization that displays fluctuating metrics as financial-style candlestick charts. Uses Rust's `crossterm` for terminal control with a volatility algorithm combining multiple system metrics into directional signals.

## 🎯 Features

- **Real-time updates**: Configurable refresh intervals
- **Multi-metric analysis**: Combines CPU, GPU, temperature, traffic, and time factors
- **Candlestick charts**: Green/red display for upward/downward movements
- **Cross-platform**: Works on Linux, macOS, Windows terminals with ANSI support
- **Efficient Rust code**: Zero external dependencies (stdlib only)
- **Configurable**: TOML-based settings file

## 📦 Dependencies

No external crates required. Uses Rust standard library only:



## 🚀 Quick Start

### Prerequisites

- **Rust toolchain**: Install from https://rustup.rs if needed
- **Modern terminal**: With ANSI escape code support (xterm, GNOME Terminal, iTerm2, etc.)
- **Linux/macOS/Windows with POSIX shell**

### Build and Run

```bash
# Navigate to project directory
cd /path/to/stock-price-ascii

# Check if Rust is installed
rustc --version
cargo --version

# Build optimized release binary
cargo build --release

# Run the application
./target/release/hardware-candlestick
```

### Test Briefly

```bash
# Run for 30 seconds as demo (if duration flag is implemented)
# cargo run --release
# Or just let it run in background for continuous monitoring
```

## 📁 Project Structure

```
stock-price-ascii/
├── Cargo.toml          # Rust package manifest and dependencies
├── README.md           # This documentation file
├── .gitignore          # Git ignore rules for build artifacts
├── config/             # TOML configuration files (optional)
│   └── candlestick.toml  # Main application configuration
├── src/                # Rust source code
│   └── main.rs        # Application entry point
└── target/             # Build output (ignored in git, see .gitignore)
```

## ⚙️ Configuration (Optional)

Advanced configuration via TOML file:

```toml
# config/candlestick.toml example structure
[algorithm]
cpu_weight = 0.35
gpu_weight = 0.22
temp_weight = 0.18
traffic_weight = 0.17
time_weight = 0.12

[noise]
 gaussian_std = 0.08
 uniform_range = "-0.06,+0.06"
 weight = 0.12

[timing]
sample_rate = 0.5  # seconds between updates
duration = null     # null for continuous, or set for demo run
```

## 🔨 Building the Project

### Compile with cargo

```bash
cd /path/to/stock-price-ascii
cargo build --release  # Optimized binary
cargo run              # Debug builds

# Check for warnings
cargo clippy           # Optional: code quality
```

## 🔬 How the Volatility Algorithm Works

## 🧪 Running and Testing

## 🧪 Testing Tips

- Test with different hardware configurations (CPU-only, GPU-present, etc.)
- Try reducing `sample_rate` for faster updates or increasing for smoother rendering
- Experiment with weight values to find your preferred "volatility feel"
- Use `--duration 60` for demo purposes before continuous monitoring

### Terminal Display

Colors appear automatically if your terminal supports ANSI escape codes (most do). If not:
- Set `TERM=xterm-256color` in your shell
- Or configure your terminal emulator's profile

### Troubleshooting Colors

If green/red doesn't appear in your terminal:

```bash
# Check terminal color support
term -xterm-256color  # Linux
```

Or set TERM explicitly:

```bash
export TERM=xterm-256color
./target/release/hardware-candlestick
```
