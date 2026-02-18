# Polymarket Toolkits

<div align="center">

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-1.0.0-green.svg)

<img width="1472" height="615" alt="image" src="https://github.com/user-attachments/assets/ce5fb363-f2dc-4f79-a2a1-1f944e20b756" />

**High-performance Rust-based trading toolkit for Polymarket CLOB**

[Features](#features) • [Installation](#installation) • [Configuration](#configuration) • [Usage](#usage) • [Safety](#safety--risk-management)

---

### 🌐 Language / 语言

[English](#polymarket-toolkits) • [简体中文](README.zh-CN.md)

</div>

---

## Overview

Polymarket Toolkits is a production-ready Rust application for automated trading on Polymarket's Central Limit Order Book (CLOB). The toolkit provides real-time monitoring of whale trades, intelligent order execution, risk management, and comprehensive safety mechanisms.

### Key Capabilities

- **Real-time Trade Monitoring**: WebSocket-based monitoring of blockchain events for instant trade detection
- **Automated Order Execution**: Intelligent order placement with FAK (Fill-or-Kill) and GTD (Good-Till-Date) support
- **Risk Management**: Built-in circuit breakers and safety guards to protect against adverse market conditions
- **Order Resubmission**: Automatic retry logic for partial fills and failed orders
- **Market Data Caching**: Efficient caching of market metadata and orderbook data
- **High Performance**: Optimized for low-latency execution with connection pooling and async I/O

## Features

### 🚀 Core Features

- **Whale Trade Copying**: Automatically detect and copy trades from monitored wallet addresses
- **Multiple Order Types**: Support for market orders (FAK), limit orders, and GTD orders
- **Intelligent Sizing**: Configurable scaling ratios and probabilistic sizing for optimal position management
- **Price Buffer Management**: Dynamic price buffers based on trade tier and market conditions
- **Partial Fill Handling**: Automatic resubmission of remaining order size after partial fills

### 🛡️ Safety Features

- **Circuit Breaker System**: Automatic trading halt after consecutive large trades
- **Orderbook Depth Checks**: Verify sufficient liquidity before order execution
- **Risk Guard**: Multi-layer risk assessment with configurable thresholds
- **Trade Size Limits**: Minimum trade size enforcement to avoid negative expected value
- **Market Liveness Detection**: Automatic detection of market status (live/closed)

### ⚡ Performance Features

- **Async Architecture**: Built on Tokio for high-concurrency async operations
- **Connection Pooling**: Optimized HTTP client with connection reuse
- **Memory Efficiency**: Stack-allocated buffers and efficient data structures
- **Zero-Copy Parsing**: Optimized blockchain event parsing with minimal allocations

## Installation

### Prerequisites

- **Rust**: Version 1.70 or higher ([Install Rust](https://www.rust-lang.org/tools/install))
- **Alchemy API Key**: For blockchain data access ([Get API Key](https://www.alchemy.com/))
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

## Configuration

### Configuration File

Create a `config.yaml` file in the project root (or copy `config.yaml.example`):

```yaml
# Wallet Configuration
main_wallet: "0x0000000000000000000000000000000000000000"  # Your wallet address
private_key: "your_64_character_hex_private_key"          # KEEP SECRET!
funder_address: "0x0000000000000000000000000000000000000000"  # Proxy wallet address

# API Configuration
alchemy_api_key: "your_alchemy_api_key"

# WebSocket Configuration
wss_url: "wss://clob.polymarket.com"

# Trading Configuration
enable_trading: true   # Set to false to disable trading (monitoring only)
mock_trading: false    # Set to true for testing without real orders

# Circuit Breaker / Risk Guard Configuration
cb_large_trade_shares: 1500.0      # Minimum shares to trigger circuit breaker
cb_consecutive_trigger: 2          # Consecutive large trades before trip
cb_sequence_window_secs: 30        # Time window for tracking consecutive trades
cb_min_depth_usd: 200.0            # Minimum orderbook depth in USD
cb_trip_duration_secs: 120         # Duration circuit breaker stays tripped (seconds)
```

### Environment Variables

You can override the config file path using:

```bash
export CONFIG=/path/to/your/config.yaml
cargo run --release
```

### Security Notes

⚠️ **CRITICAL**: Never commit your `config.yaml` file to version control. It contains sensitive private keys.

- Add `config.yaml` to `.gitignore`
- Use environment variables for CI/CD deployments
- Store private keys in secure secret management systems for production

## Usage

### Basic Usage

1. **Configure your settings** in `config.yaml`
2. **Ensure wallet has USDC** on Polygon network
3. **Approve exchange** at [Polymarket.com](https://polymarket.com) (make a test trade)
4. **Run the application**:

```bash
cargo run --release
```

### Monitoring Mode

To monitor trades without executing orders:

```yaml
enable_trading: false
```

### Mock Trading Mode

To test order logic without real transactions:

```yaml
mock_trading: true
```

### Programmatic Usage

```rust
use polymarket_toolkits::client::create_authenticated_clob_client;
use polymarket_toolkits::config::BotConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = BotConfig::load()?;
    
    // Create authenticated client
    let (client, credentials) = create_authenticated_clob_client(
        config.private_key.clone(),
        config.funder_address.clone(),
    ).await?;
    
    // Use client for trading operations...
    
    Ok(())
}
```

## Architecture

### Core Components

```
src/
├── main.rs           # Main entry point and WebSocket event loop
├── lib.rs            # Core library exports and utilities
├── client.rs         # CLOB client creation and authentication
├── processor.rs      # Order processing and execution logic
├── risk_guard.rs     # Risk management and circuit breaker
├── market_cache.rs   # Market data caching layer
├── orders.rs         # Order placement and management
├── config.rs         # Configuration management
├── models.rs         # Data models and types
└── settings.rs       # Application settings and constants
```

### Data Flow

```
WebSocket Events → Event Parser → Risk Guard → Order Processor → CLOB API
                                                      ↓
                                              Resubmit Queue
```

### Key Design Decisions

- **Async-first**: All I/O operations are async for maximum throughput
- **Thread Safety**: Shared state uses `Arc` for safe concurrent access
- **Error Handling**: Comprehensive error handling with `anyhow::Result`
- **Resource Management**: Connection pooling and efficient memory usage

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

1. **Start Small**: Begin with `mock_trading: true` to test your setup
2. **Monitor Closely**: Watch logs for the first few real trades
3. **Set Limits**: Configure appropriate circuit breaker thresholds
4. **Regular Updates**: Keep dependencies updated for security patches
5. **Backup Funds**: Never use more than you can afford to lose

## Performance

### Benchmarks

- **Event Processing**: < 1ms per event
- **Order Execution**: < 100ms end-to-end latency
- **Memory Usage**: ~50MB baseline, scales with cache size
- **CPU Usage**: < 5% on modern hardware

### Optimization Tips

- Use release builds for production: `cargo build --release`
- Adjust cache sizes based on your market coverage
- Monitor connection pool metrics
- Use appropriate WebSocket ping intervals

## Troubleshooting

### Common Issues

**"INSUFFICIENT_BALANCE/ALLOWANCE"**
- Ensure wallet has USDC on Polygon
- Approve exchange at Polymarket.com
- Check `funder_address` matches your proxy wallet

**"RISK_BLOCKED"**
- Circuit breaker has tripped
- Wait for trip duration or adjust thresholds
- Check orderbook depth requirements

**"NETWORK" or "HTTP_ERROR"**
- Check internet connection
- Verify API endpoints are accessible
- Review Alchemy API key validity

**WebSocket Disconnections**
- Normal behavior - automatic reconnection
- Check network stability
- Review ping timeout settings

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
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

⚠️ **Trading Risk Warning**: This software is provided for educational and research purposes. Trading cryptocurrencies and prediction markets involves substantial risk of loss. Past performance does not guarantee future results. Use at your own risk.

- **No Warranty**: The software is provided "as is" without warranty of any kind
- **Not Financial Advice**: This is not investment or financial advice
- **Compliance**: Ensure compliance with local regulations and Polymarket's terms of service
- **Testing**: Always test thoroughly in mock mode before using real funds

## Support

For issues, questions, or contributions:

- Go to [here](.SUPPORT.md)

## Acknowledgments

- Built with [Polymarket Client SDK](https://github.com/Polymarket/polymarket-client-sdk-rs)
- Powered by [Tokio](https://tokio.rs/) async runtime
- Uses [Alloy](https://github.com/alloy-rs/alloy) for Ethereum interactions

---

<div align="center">

**Made with ❤️ for the Polymarket community**

[⬆ Back to Top](#polymarket-toolkits)

</div>
