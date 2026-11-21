# 🕯️ Candlestick Terminal Visualization

Real-time terminal-based ASCII art visualization that displays fluctuating metrics as financial-style candlestick charts. Uses Rust's `crossterm` for terminal control with a volatility algorithm combining multiple system metrics into directional signals.

## 🎯 Features

- **Real-time updates**: Configurable refresh intervals
- **Multi-metric analysis**: Combines CPU, GPU, temperature, traffic, and time factors  
- **Candlestick charts**: Green/red display for upward/downward movements
- **Cross-platform**: Works on Linux, macOS, Windows terminals with ANSI support
- **Efficient Rust code**: Zero external dependencies (stdlib only)
- **Configurable**: TOML-based settings file

## 📦 Dependencies

No external crates required. Uses Rust standard library only.

## 🚀 Quick Start

### Prerequisites

- **Rust toolchain**: Install from https://rustup.rs if needed
- **Modern terminal**: With ANSI escape code support (xterm, GNOME Terminal, iTerm2, etc.)
- **Linux/macOS/Windows with POSIX shell**

### Installation & Building

```bash
# Navigate to project directory
cd /path/to/your/project

# Build optimized release binary
cargo build --release

# Or run directly without building  
cargo run --release
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

## 🔬 How It Works

### The Concept

This terminal visualization displays real-time system metrics as a candlestick-style chart. Think of it like a financial stock chart, but visualizing your computer's health metrics instead:

- **Green candles**: System state trending "upward" (e.g., increasing load)
- **Red candles**: System state trending "downward"

The algorithm combines multiple hardware metrics (CPU usage, GPU utilization, temperature, network traffic, time-of-day) into a single fluctuating value. This creates an organic, unpredictable pattern similar to real market movements.

### Installation & Running

**On your own machine:**

1. **Clone or download** the repository
2. **Install Rust** (if you haven't already): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. **Add to PATH**: Add `~/.cargo/bin` to your shell's `$PATH`
4. **Build locally**: `cargo build --release`
5. **Run the binary**: `./target/release/your-app-name`

**Usage examples:**
```bash
# Run continuously (no time limit)
cargo run --release

# Run for demo period
cargo run --release -- --demo 60  # Runs for 60 seconds
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

### Compile with Cargo

```bash
cd /path/to/your/project
cargo build --release  # Optimized binary
cargo run              # Debug builds

# Check for warnings
cargo clippy           # Optional: code quality
```

### Terminal Display

Colors appear automatically if your terminal supports ANSI escape codes (most do). If not:

- Set `TERM=xterm-256color` in your shell
- Or configure your terminal emulator's profile

## 🐛 Troubleshooting Colors

If green/red doesn't appear in your terminal, try:

```bash
# Export 256-color terminal support
export TERM=xterm-256color

# Then run the application again
./target/release/hardware-candlestick
```

## 🧪 Testing Tips

- Test with different hardware configurations (CPU-only, GPU-present, etc.)
- Try reducing `sample_rate` for faster updates or increasing for smoother rendering  
- Experiment with weight values to find your preferred "volatility feel"

## 📚 Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [crossterm documentation](https://docs.rs/crossterm)
- [ratatui GitHub](https://github.com/ratatui/ratatui)
- [Candlestick charts: Wikipedia](https://en.wikipedia.org/wiki/Candlestick_chart)

## 📄 License

MIT License - Feel free to use, modify, and distribute this project.
