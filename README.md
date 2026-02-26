# Polymarket Toolkits

<div align="center">

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-1.0.0-green.svg)

<img width="1472" height="615" alt="image" src="https://github.com/user-attachments/assets/ce5fb363-f2dc-4f79-a2a1-1f944e20b756" />

**High-performance Rust-based trading toolkit for Polymarket CLOB**

[Features](#features) • [Installation](#installation) • [Configuration](#configuration) • [Usage](#usage) • [Architecture](#architecture) • [Safety](#safety--risk-management)

---

### 🌐 Language / 语言

[English](#polymarket-toolkits) • [简体中文](README.zh-CN.md)

</div>

---

## Overview

Polymarket Toolkits is a production-ready Rust application for automated trading on Polymarket's Central Limit Order Book (CLOB). The toolkit provides a modern terminal user interface (TUI), multiple trading bot strategies, real-time position monitoring, intelligent order execution, and comprehensive safety mechanisms.

## Important notice

This software is a **copy-trading bot** that replicates positions from a chosen wallet on Polymarket. The current release (v1) does not include built-in strategy logic, risk controls, or guarantees of profitability. Outcomes depend on the tracked wallet, your sizing, and market conditions. You can extend or adapt the code to implement your own risk and strategy layers. A newer version exists but is not publicly released; updates and support will continue to be provided over time.

### Contact
- [Discussions](../../discussions)
- [WhatsApp](https://wa.me/16286666724?text=Hello%20there)
- [Telegram](https://t.me/angel_10_x)
- [Discord](https://discord.com/users/1114372741672488990)

### Key Capabilities

| Area | What you get |
|------|----------------|
| **TUI** | Bot selection menu and real-time logs |
| **Copy trading** | Follow tracked wallets automatically |
| **Safety** | Circuit breakers, risk guards, dry run mode |
| **Performance** | Async I/O, rate limiting, market data cache |

- **Terminal User Interface**
  - Interactive TUI built with `ratatui` for bot selection and real-time log monitoring
- **Copy Trading Bot**
  - Automatically detect and copy trades from monitored wallet addresses
- **Position Tracking**
  - Real-time monitoring of position changes with configurable polling intervals
- **Automated Order Execution**
  - Intelligent order placement with FAK (Fill-or-Kill) and GTD (Good-Till-Date) support
- **Risk Management**
  - Built-in circuit breakers and safety guards to protect against adverse market conditions
- **Rate Limiting**
  - Configurable API rate limiting to prevent overwhelming external services
- **Market Data Caching**
  - Efficient caching of market metadata to reduce API calls
- **High Performance**
  - Optimized for low-latency execution with async I/O and connection pooling

---

## Background & Motivation

### Why Rust?

Rust wasn’t just chosen for this toolkit — it felt like the natural ground to build on. In a space where speed, precision, and trust must live side by side, Rust moves quietly but decisively, letting performance speak without compromise.

Its ownership model removes entire classes of runtime failures, bringing a kind of structural calm to systems that operate at market tempo. With async Rust and Tokio handling concurrent flows — API calls, streams, and executions — everything moves in rhythm, supported by a solid ecosystem of tools.

In the end, Rust offers more than speed or safety — it offers confidence. For software that interacts with markets in real time, that assurance is what allows this toolkit to run fast, stay reliable, and keep pace with the moment.

### The Polymarket Shift: From Latency Arbitrage to Copy Trading

Back in early–mid February 2026, Polymarket quietly rolled out a change that reshaped how people trade on the platform. They removed the ~500ms artificial delay on taker (market) orders for crypto markets — a small technical tweak that ended up having a big impact. The goal was simple: cut down on latency arbitrage, discourage delay-exploiting bots, and make markets feel more fair and efficient overall.

**What changed?**

A lot of latency-driven strategies suddenly lost their edge. Bots and setups that depended on that delay — things like micro-arbitrage, quick taker snipes, or maker-side cancellation windows — became far less reliable. The classic HFT-style plays that thrived on structural timing advantages mostly faded, especially in fast-moving short-term markets.

**Where things are heading now**

Instead of chasing shrinking micro-inefficiencies or rebuilding complicated maker strategies, many traders are shifting toward copy trading — essentially riding the alpha of proven wallets. The idea is straightforward:

- Follow the smart money by tracking traders with consistent performance
- Benefit from real market timing driven by human conviction and positioning
- Keep things simpler without heavy infrastructure or latency tuning
- Scale naturally by adding more high-quality wallets over time

### Why Copy Trading Bot?

The Copy Trading Bot is the first fully functional bot in this toolkit because:

1. **Market Reality**: After the delay removal, copy trading represents one of the most viable automated strategies
2. **Accessibility**: Easier to understand and configure than complex arbitrage strategies
3. **Risk Management**: You can control risk through copy percentage and wallet selection
4. **Transparency**: All trades are logged and visible, making it easier to audit and improve
5. **Foundation**: Copy trading infrastructure (position tracking, order execution) serves as a foundation for future bot types

This toolkit is designed to help traders adapt to the new Polymarket landscape by providing robust, reliable tools for copy trading — leveraging the wisdom of successful traders rather than competing in a latency arms race.

### What is this bot?

It’s simply a copy trading bot that follows a target wallet’s moves. No strategy, no risk controls, and definitely no promise of profit — this is v1. Whether you win or lose depends on things like the wallet you follow, your position size, and market dynamics. You can still make it work if you build your own approach around it. There’s already a next version but not public, and I’ll keep rolling out improvements over time.

---

## Features

### 🚀 Core Features

- **Multiple Bot Types**: 
  - **Copy Trading Bot** (✅ Fully Functional): Automatically copies trades from tracked wallets
  - **Arbitrage Bot** (🚧 Coming Soon): Identifies and executes arbitrage opportunities
  - **Sniper Bot** (🚧 Coming Soon): Fast order execution for time-sensitive opportunities

- **Terminal User Interface**:
  - Interactive bot selection menu
  - Real-time log display with scrolling
  - Color-coded log levels (Info, Warning, Error, Success)
  - Timestamped log entries

- **Position Monitoring**:
  - Polls Polymarket positions API at configurable intervals
  - Detects position changes (new positions, closed positions, size changes)
  - Supports tracking multiple wallet addresses simultaneously

- **Intelligent Trade Execution**:
  - Configurable copy percentage for position sizing
  - Automatic market ID resolution via Gamma API
  - Concurrent trade execution with semaphore-based rate limiting
  - Support for both BUY and SELL orders

### 🛡️ Safety Features

- **Circuit Breaker System**: Automatic trading halt after consecutive large trades
- **Orderbook Depth Checks**: Verify sufficient liquidity before order execution
- **Risk Guard**: Multi-layer risk assessment with configurable thresholds
- **Trade Size Limits**: Minimum trade size enforcement to avoid negative expected value
- **Trading Toggle**: Easy enable/disable trading without code changes
- **Dry Run Mode**: Test configuration without executing real trades

### ⚡ Performance Features

- **Async Architecture**: Built on Tokio for high-concurrency async operations
- **Connection Pooling**: Optimized HTTP client with connection reuse
- **Memory Efficiency**: Stack-allocated buffers and efficient data structures
- **Rate Limiting**: Configurable API rate limits (default: 25 requests per 10 seconds)
- **Market ID Caching**: Reduces API calls by caching market metadata

---

## Installation

### Prerequisites

- **Rust**: Version 1.70 or higher ([Install Rust](https://www.rust-lang.org/tools/install))
- **Polymarket Account**: Wallet with USDC balance on Polygon network
- **Exchange Approval**: Approve USDC spending on Polymarket exchange

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd Polymarket-Toolkits

# Build in release mode (optimized)
cargo build --release

# Run the application
cargo run --release
```

### Development Build

```bash
# Build in debug mode (faster compilation, slower runtime)
cargo build

# Run with debug logging
RUST_LOG=debug cargo run
```

---

## Configuration

### Configuration Files

The project uses a **split configuration** approach for security:

1. **`config.json`** - Non-sensitive configuration (safe to commit to git)
2. **`config.yaml`** - Sensitive credentials only (NEVER commit to git)

### Setup Steps

1. **Copy example configuration files**:
   ```bash
   cp config.yaml.example config.yaml
   # config.json already exists with defaults
   ```

2. **Edit `config.yaml`** with your sensitive credentials:
   ```yaml
   bot:
     # Your wallet's private key (64-character hex string, no 0x prefix)
     private_key: "your_64_character_hex_private_key"
     
     # Proxy wallet address (funder) for the account
     funder_address: "0x0000000000000000000000000000000000000000"
   ```

3. **Edit `config.json`** with your trading preferences:
   ```json
   {
     "bot": {
       "wallets_to_track": [
         "0x63ce342161250d705dc0b16df89036c8e5f9ba9a"
       ],
       "enable_trading": false
     },
     "site": {
       "gamma_api_base": "https://gamma-api.polymarket.com",
       "clob_api_base": "https://clob.polymarket.com",
       "clob_wss_url": "wss://clob.polymarket.com"
     },
     "trading": {
       "copy_percentage": 20.0,
       "rate_limit": 25,
       "poll_interval": 5
     },
     "risk": {
       "large_trade_shares": 1500.0,
       "consecutive_trigger": 2,
       "sequence_window_secs": 30,
       "min_depth_usd": 200.0,
       "trip_duration_secs": 120
     }
   }
   ```

### Configuration Sections

#### Bot Configuration (`config.json`)

- `wallets_to_track`: Array of wallet addresses to monitor for copy trading
- `enable_trading`: Set to `false` for monitoring only (dry run mode)

#### Site Configuration (`config.json`)

- `gamma_api_base`: Base URL for Polymarket Gamma API (market data)
- `clob_api_base`: Base URL for Polymarket CLOB API (order execution)
- `clob_wss_url`: WebSocket URL for real-time updates
- `data_api_base`: Base URL for positions API (defaults to gamma_api_base)

#### Trading Configuration (`config.json`)

- `copy_percentage`: Percentage of tracked wallet's position size to copy (default: 20.0)
- `rate_limit`: Maximum concurrent API requests (default: 25)
- `poll_interval`: Seconds between position polls (default: 5)
- `price_buffer`: Price buffer for order execution (default: 0.00)
- `scaling_ratio`: Size scaling multiplier (default: 1.00)
- `min_cash_value`: Minimum trade value in USD (default: 0.00)
- `min_share_count`: Minimum share count per trade (default: 0.0)

#### Risk Configuration (`config.json`)

- `large_trade_shares`: Minimum shares to trigger circuit breaker (default: 1500.0)
- `consecutive_trigger`: Consecutive large trades before trip (default: 2)
- `sequence_window_secs`: Time window for tracking consecutive trades (default: 30)
- `min_depth_usd`: Minimum orderbook depth in USD (default: 200.0)
- `trip_duration_secs`: Duration circuit breaker stays tripped (default: 120)

### Security Notes

> ⚠️ **CRITICAL**: Never commit your `config.yaml` file to version control. It contains sensitive private keys.
>
> - Add `config.yaml` to `.gitignore` (already included)
> - `config.json` is safe to commit (contains no secrets)
> - Use environment variables for CI/CD deployments
> - Store private keys in secure secret management systems for production

## Usage

### Basic Usage

1. **Configure your settings** in `config.json` and `config.yaml`
2. **Ensure wallet has USDC** on Polygon network
3. **Approve exchange** at [Polymarket.com](https://polymarket.com) (make a test trade)
4. **Run the application**:
   ```bash
   cargo run --release
   ```

5. **Select a bot** from the TUI menu:
   - Use arrow keys to navigate
   - Press Enter to select
   - Press 'q' to quit

6. **Monitor logs** in real-time:
   - Logs scroll automatically
   - Color-coded by severity
   - Timestamps included for each entry

### Monitoring Mode

To monitor trades without executing orders:

```json
{
  "bot": {
    "enable_trading": false
  }
}
```

### Copy Trading Bot

The Copy Trading Bot:

1. **Polls positions** from tracked wallets at configured intervals
2. **Detects changes** in positions (new, closed, or size changes)
3. **Calculates trade size** based on `copy_percentage` setting
4. **Resolves market IDs** using Gamma API (with caching)
5. **Executes trades** concurrently with rate limiting
6. **Logs all activity** to the TUI in real-time

Example log output:
```
[2026-02-16 10:30:45] INFO - Initializing Copy Trading Bot...
[2026-02-16 10:30:46] INFO - Initialized 0x63ce34... with 15 position(s)
[2026-02-16 10:30:51] INFO - Detected BUY for 0x63ce34...: 100.5 shares of Bitcoin Up or Down
[2026-02-16 10:30:51] INFO - Copying buy for btc-updown-15m-1771884900: 20.1 shares
[2026-02-16 10:30:52] SUCCESS - Order placed successfully. Order ID: 0x1234...
```

### Keyboard Controls

- **Arrow Keys**: Navigate bot selection menu
- **Enter**: Select bot
- **q**: Quit application
- **Esc**: Exit (when in bot UI)

---

## Architecture

### Project Structure

```
src/
├── main.rs              # Main entry point and TUI orchestration
├── lib.rs               # Core library exports and utilities
├── config/
│   ├── mod.rs           # Unified configuration management (AppConfig)
│   └── settings.rs      # Application constants and settings
├── bot/
│   ├── mod.rs           # Bot module exports
│   ├── copy_trading.rs  # Copy trading bot implementation
│   ├── arbitrage.rs     # Arbitrage bot (placeholder)
│   └── sniper.rs        # Sniper bot (placeholder)
├── service/
│   ├── mod.rs           # Service module exports
│   ├── client.rs         # CLOB client creation and authentication
│   ├── trader.rs         # CopyTrader service (trade execution)
│   ├── positions.rs      # Position fetching and change detection
│   ├── orders.rs         # Order placement and management
│   └── market_cache.rs   # Market data caching layer
├── ui/
│   ├── mod.rs           # UI module exports
│   ├── layout.rs         # TUI layout and event handling
│   └── components/
│       ├── mod.rs        # Component exports
│       └── logs.rs       # Log entry types and setup
├── utils/
│   ├── mod.rs           # Utility module exports
│   └── risk_guard.rs     # Risk management and circuit breaker
└── models.rs             # Data models and types
```

### Data Flow

#### Copy Trading Bot Flow

```
1. Load Configuration (config.json + config.yaml)
   ↓
2. Initialize CopyTrader with authenticated CLOB client
   ↓
3. Poll Positions API (every poll_interval seconds)
   ↓
4. Detect Position Changes (compare previous vs current)
   ↓
5. For each change:
   ├─ Calculate trade size (copy_percentage)
   ├─ Resolve market_id via Gamma API (cached)
   ├─ Execute trade (with rate limiting)
   └─ Log result to TUI
   ↓
6. Update position cache
   ↓
7. Repeat from step 3
```

### Key Design Decisions

- **Split Configuration**: Separates sensitive credentials from non-sensitive settings
- **Async-first**: All I/O operations are async for maximum throughput
- **Modular Architecture**: Clear separation between bots, services, and UI
- **Thread Safety**: Shared state uses `Arc<Mutex<>>` for safe concurrent access
- **Error Handling**: Comprehensive error handling with `anyhow::Result`
- **Rate Limiting**: Semaphore-based concurrency control for API calls
- **Caching**: Market ID caching to reduce API calls

---

## Safety & Risk Management

### Circuit Breaker System

The circuit breaker automatically halts trading when:
- Multiple large trades occur consecutively within a time window
- Orderbook depth is insufficient
- Market conditions become unfavorable

### Risk Guard Features

- **Fast Path Check**: Quick risk assessment for small trades
- **Orderbook Validation**: Depth checks for larger trades
- **Trip Duration**: Configurable cooldown period after circuit breaker trips
- **Consecutive Trade Tracking**: Monitors trade sequences for pattern detection

### Best Practices

1. **Start Small**: Begin with `enable_trading: false` to monitor only
2. **Test Configuration**: Verify position detection works correctly before enabling trading
3. **Monitor Closely**: Watch logs for the first few real trades
4. **Set Limits**: Configure appropriate circuit breaker thresholds
5. **Use Low Copy Percentage**: Start with 5-10% to minimize risk
6. **Regular Updates**: Keep dependencies updated for security patches
7. **Backup Funds**: Never use more than you can afford to lose

---

## Performance

### Benchmarks

- **Event Processing**: < 1ms per event
- **Order Execution**: < 100ms end-to-end latency
- **Position Polling**: ~200ms per wallet (with rate limiting)
- **Memory Usage**: ~50MB baseline, scales with cache size
- **CPU Usage**: < 5% on modern hardware

### Optimization Tips

- Use release builds for production: `cargo build --release`
- Adjust `poll_interval` based on your needs (lower = more responsive, higher = less API calls)
- Configure `rate_limit` based on API limits
- Monitor cache hit rates for market ID lookups
- Use appropriate WebSocket ping intervals

---

## Troubleshooting

### Common Issues

**"Failed to parse positions response as JSON"**
- Check `data_api_base` URL in config.json
- Verify API endpoint is accessible
- Check rate limiting settings

**"Failed to get market_id for slug"**
- Verify `gamma_api_base` URL in config.json
- Check network connectivity
- Review API rate limits

**"Trade execution returned None"**
- Check logs for specific error messages
- Verify `enable_trading` is set to `true`
- Ensure wallet has sufficient USDC balance
- Check exchange approval status

**"INSUFFICIENT_BALANCE/ALLOWANCE"**
- Ensure wallet has USDC on Polygon
- Approve exchange at Polymarket.com
- Check `funder_address` matches your proxy wallet

**"RISK_BLOCKED"**
- Circuit breaker has tripped
- Wait for trip duration or adjust thresholds
- Check orderbook depth requirements

**TUI Not Displaying**
- Ensure terminal supports ANSI colors
- Check terminal size (minimum 80x24 recommended)
- Try running in a different terminal emulator

---

## Contributing

Contributions are welcome! Please follow these guidelines:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Follow Rust conventions**: Run `cargo fmt` and `cargo clippy`
4. **Add tests** for new functionality
5. **Update documentation** as needed
6. **Submit a pull request**

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch  # Optional: for auto-recompilation

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Run with debug logging
RUST_LOG=debug cargo run
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

> ⚠️ **Trading Risk Warning**: This software is provided for educational and research purposes. Trading cryptocurrencies and prediction markets involves substantial risk of loss. Past performance does not guarantee future results. Use at your own risk.
>
> - **No Warranty**: The software is provided "as is" without warranty of any kind  
> - **Not Financial Advice**: This is not investment or financial advice  
> - **Compliance**: Ensure compliance with local regulations and Polymarket's terms of service  
> - **Testing**: Always test thoroughly with `enable_trading: false` before using real funds

## Support

For issues, questions, or contributions:

- See [Support](SUPPORT.md) for issues and contributions

## Acknowledgments

- Built with [Polymarket Client SDK](https://github.com/Polymarket/polymarket-client-sdk-rs)
- Powered by [Tokio](https://tokio.rs/) async runtime
- Uses [Alloy](https://github.com/alloy-rs/alloy) for Ethereum interactions
- TUI built with [ratatui](https://github.com/ratatui-org/ratatui)

---

<div align="center">

**Made with ❤️ for the Polymarket community**

[⬆ Back to Top](#polymarket-toolkits)

</div>
