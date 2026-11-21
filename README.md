# Hardware Candlestick

A Rust terminal dashboard that turns live system telemetry into a simulated market index and renders it as a scrolling candlestick chart.

## Features

- Live CPU and memory usage from `sysinfo`
- Network receive/transmit throughput from byte-counter deltas
- OHLC candles with wicks and bodies
- Bounded historical candle buffer and dynamic chart scaling
- Ratatui terminal UI with pause, reset, and sampling-speed controls
- TOML configuration with sensible defaults

The market price is a visualization, not a financial quote. Hardware metrics influence the simulated price alongside noise and mean reversion.

## Run

Install a recent stable Rust toolchain, then:

```bash
cargo run --release
```

## Controls

| Key | Action |
|---|---|
| `q` / `Esc` | Quit |
| `Space` / `p` | Pause or resume market candles |
| `r` | Reset market history |
| `+` / `=` | Sample faster |
| `-` | Sample slower |
| `1` | 1 second interval |
| `2` | 2.5 second interval |
| `3` | 5 second interval |

## Configuration

Edit `config/candlestick.toml`:

```toml
starting_price = 100.0
history_limit = 72
sample_interval_ms = 2500
market_floor = 20.0
market_ceiling = 250.0
```

The app looks for `config/candlestick.toml`, then `candlestick.toml` in the current working directory. Missing or invalid configuration falls back to defaults.

## Notes

- CPU usage is sampled using `sysinfo`; it is not inferred from Linux load average.
- Network rates are calculated from cumulative byte-counter differences.
- Temperature, GPU, and per-disk metrics are intentionally not shown yet because support varies by operating system and hardware.
- This is a terminal visualization only; it does not place trades or represent real market data.
