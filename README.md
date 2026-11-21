# 🕯️ Hardware Candlestick Chart - Rust Terminal Visualization

Real-time terminal-based ASCII art animation that visualizes system hardware metrics as financial candlestick charts. Uses Rust's `ratatui` and `crossterm` for terminal UI rendering with a volatility algorithm combining CPU/GPU/Temp/Traffic metrics into a single directional signal.

## 🎯 Features

- **Real-time monitoring**: Updates every configurable interval (default 0.5s)
- **Volatility-driven**: Combines multiple hardware metrics + noise injection for organic fluctuations  
- **Candlestick visualization**: Green/red bars showing "up" or "down" movement
- **Cross-platform**: Works on Linux, macOS, Windows terminals
- **Rust performance**: Zero-copy design, efficient CPU usage
- **Configurable**: TOML-based configuration file

## 📦 Dependencies

```toml
crossterm = "0.28"    # Terminal abstraction layer
ratatui = "0.29"      # Modern TUI framework
rand = "0.8"          # Random number generation
serde = { version = "1.0", features = ["derive"] }  # Config serialization
toml = "0.8"          # TOML file parsing
tracing = "0.1"       # Structured logging
```

## 🚀 Quick Start

### 1. Setup Rust Development Environment

Rust is already installed on your system (`rustc` and `cargo` found in `/usr/bin/`).

### 2. Build the Project

```bash
cd /home/ismail/.openclaw/workspace/coordinator
cargo build --release
```

### 3. Run the Application

```bash
./target/release/candlestick
# Or with configuration:
./candlestick --config config/candlestick.toml
```

### 4. Test briefly

```bash
./candlestick --duration 10  # Run for 10 seconds as demo
```

## 📁 Project Structure

```
hardware-candlestick/
├── Cargo.toml          # Rust package manifest and dependencies
├── README.md           # This documentation file
├── .gitignore          # Git ignore rules for build artifacts
├── config/             # TOML configuration files
│   └── candlestick.toml  # Main application configuration
├── src/                # Rust source code
│   ├── main.rs        # Application entry point
│   └── lib.rs         # Shared library modules (if split later)
├── assets/             # Static resources, fonts if needed
└── target/             # Build output (ignored in git)
```

## ⚙️ Configuration Options

Edit `config/candlestick.toml` to customize:

### Volatility Algorithm Weights
- Adjust how much each metric influences the signal:
  - `cpu`: CPU load percentage (highest weight)
  - `gpu`: GPU utilization
  - `temp`: Temperature deviation from baseline
  - `traffic`: Network throughput
  - `time`: Hour-of-day factor

### Noise Parameters  
- Control volatility intensity:
  - `gaussian_std`: Standard deviation as fraction (0.08 = 8%)
  - `uniform_range`: Uniform jitter range (±0.06)
  - `noise_weight`: How much noise affects output (0.12)

### Timing Settings
- `sample_rate`: Update frequency in seconds (0.5s recommended)
- `duration`: Max runtime for testing (set to 0 or disabled for continuous mode)

## 🛠️ Development Workflow

### Build & Run Locally

```bash
# Development build (with debug output)
cargo run

# Production build (optimized, smaller binary)
cargo build --release

# Check for compilation warnings
cargo clippy  # Optional: code quality checker
```

### Git Version Control

The project is already in your workspace directory. To initialize and push to GitHub:

```bash
cd /home/ismail/.openclaw/workspace/coordinator
git init
git add .
git commit -m "Initial hardware candlestick chart"

# Add remote repository (replace with your actual repo URL)
git remote add origin git@github.com:your-username/hardware-candlestick.git
git push -u origin main
```

## 🔬 Volatility Algorithm

The Rust implementation uses this formula:

```rust
V(t) = w_cpu * N(cpu, t) + 
       w_gpu * N(gpu, t) + 
       w_temp * N(temp, t) + 
       w_traffic * N(traffic, t) + 
       w_time * N(time, t) + 
       NOISE_COMPONENT
```

Where `NOISE_COMPONENT = gaussian_noise + uniform_jitter`.

See `src/main.rs` for implementation details and the volatility calculation function.

## 🧪 Testing Tips

- Test with different hardware configurations (CPU-only, GPU-present, etc.)
- Try reducing `sample_rate` for faster updates or increasing for smoother rendering
- Experiment with weight values to find your preferred "volatility feel"
- Use `--duration 60` for demo purposes before continuous monitoring

## 🐛 Debugging

Enable verbose logging in `config/candlestick.toml`:

```toml
[debug]
verbose_logging = true
show_raw_metrics = true
```

This will output metric values and intermediate calculations to the terminal.

## 📚 Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [crossterm documentation](https://docs.rs/crossterm)
- [ratatui GitHub](https://github.com/ratatui/ratatui)
- [Candlestick charts: Wikipedia](https://en.wikipedia.org/wiki/Candlestick_chart)

## 📄 License

MIT License - See LICENSE file for details.

## 👤 Author

Ismail - hardware-candlestick project contributor
