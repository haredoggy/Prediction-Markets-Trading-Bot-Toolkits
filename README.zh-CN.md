# Polymarket 工具包

<div align="center">

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-1.0.0-green.svg)

<img width="1472" height="615" alt="image" src="https://github.com/user-attachments/assets/ce5fb363-f2dc-4f79-a2a1-1f944e20b756" />

**高性能 Rust 语言开发的 Polymarket CLOB 交易工具包**

[功能特性](#功能特性) • [安装指南](#安装指南) • [配置说明](#配置说明) • [使用指南](#使用指南) • [安全机制](#安全与风险管理)

---

### 🌐 Language / 语言

[English](README.md) • [简体中文](#polymarket-工具包)

</div>

---

## 项目概述

Polymarket 工具包是一个生产就绪的 Rust 应用程序，用于在 Polymarket 中央限价订单簿（CLOB）上进行自动化交易。该工具包提供实时监控大额交易、智能订单执行、风险管理和全面的安全机制。

### 核心能力

- **实时交易监控**：基于 WebSocket 的区块链事件监控，实现即时交易检测
- **自动化订单执行**：智能订单下达，支持 FAK（全部成交或取消）和 GTD（指定日期前有效）订单
- **风险管理**：内置熔断机制和安全防护，保护交易免受不利市场条件影响
- **订单重新提交**：部分成交和失败订单的自动重试逻辑
- **市场数据缓存**：高效缓存市场元数据和订单簿数据
- **高性能**：针对低延迟执行优化，包含连接池和异步 I/O

## 功能特性

### 🚀 核心功能

- **大额交易复制**：自动检测并复制监控钱包地址的交易
- **多种订单类型**：支持市价单（FAK）、限价单和 GTD 订单
- **智能仓位管理**：可配置的缩放比例和概率性仓位管理，实现最优持仓
- **价格缓冲管理**：基于交易层级和市场条件的动态价格缓冲
- **部分成交处理**：部分成交后自动重新提交剩余订单数量

### 🛡️ 安全功能

- **熔断系统**：连续大额交易后自动停止交易
- **订单簿深度检查**：订单执行前验证流动性是否充足
- **风险防护**：多层风险评估，可配置阈值
- **交易规模限制**：强制执行最小交易规模，避免负期望值
- **市场活跃度检测**：自动检测市场状态（活跃/关闭）

### ⚡ 性能特性

- **异步架构**：基于 Tokio 构建，支持高并发异步操作
- **连接池**：优化的 HTTP 客户端，支持连接复用
- **内存效率**：栈分配缓冲区和高效数据结构
- **零拷贝解析**：优化的区块链事件解析，最小化内存分配

## 安装指南

### 前置要求

- **Rust**：版本 1.70 或更高（[安装 Rust](https://www.rust-lang.org/zh-CN/tools/install)）
- **Alchemy API 密钥**：用于区块链数据访问（[获取 API 密钥](https://www.alchemy.com/)）
- **Polymarket 账户**：在 Polygon 网络上拥有 USDC 余额的钱包
- **交易所授权**：在 Polymarket 交易所授权 USDC 支出

### 从源码构建

```bash
# 克隆仓库
git clone <repository-url>
cd Polymarket-Toolkits

# 以发布模式构建（优化版本）
cargo build --release

# 运行应用程序
cargo run --release
```

### 开发构建

```bash
# 以调试模式构建（编译更快，运行较慢）
cargo build

# 使用调试日志运行
RUST_LOG=debug cargo run
```

## 配置说明

### 配置文件

在项目根目录创建 `config.yaml` 文件（或复制 `config.yaml.example`）：

```yaml
# 钱包配置
main_wallet: "0x0000000000000000000000000000000000000000"  # 您的钱包地址
private_key: "your_64_character_hex_private_key"          # 请妥善保管！
funder_address: "0x0000000000000000000000000000000000000000"  # 代理钱包地址

# API 配置
alchemy_api_key: "your_alchemy_api_key"

# WebSocket 配置
wss_url: "wss://clob.polymarket.com"

# 交易配置
enable_trading: true   # 设置为 false 可禁用交易（仅监控）
mock_trading: false    # 设置为 true 可在不执行真实订单的情况下测试

# 熔断器 / 风险防护配置
cb_large_trade_shares: 1500.0      # 触发熔断器的最小交易份额
cb_consecutive_trigger: 2          # 触发前的连续大额交易次数
cb_sequence_window_secs: 30        # 跟踪连续交易的时间窗口（秒）
cb_min_depth_usd: 200.0            # 最小订单簿深度（美元）
cb_trip_duration_secs: 120         # 熔断器保持触发状态的持续时间（秒）
```

### 环境变量

您可以使用环境变量覆盖配置文件路径：

```bash
export CONFIG=/path/to/your/config.yaml
cargo run --release
```

### 安全注意事项

⚠️ **重要提示**：切勿将 `config.yaml` 文件提交到版本控制系统。该文件包含敏感的私钥。

- 将 `config.yaml` 添加到 `.gitignore`
- 在 CI/CD 部署中使用环境变量
- 在生产环境中将私钥存储在安全的密钥管理系统中

## 使用指南

### 基本使用

1. **配置设置**：在 `config.yaml` 中配置您的设置
2. **确保钱包有 USDC**：在 Polygon 网络上确保钱包有 USDC 余额
3. **授权交易所**：在 [Polymarket.com](https://polymarket.com) 授权交易所（进行一笔测试交易）
4. **运行应用程序**：

```bash
cargo run --release
```

### 监控模式

在不执行订单的情况下监控交易：

```yaml
enable_trading: false
```

### 模拟交易模式

在不进行真实交易的情况下测试订单逻辑：

```yaml
mock_trading: true
```

### 编程式使用

```rust
use polymarket_toolkits::client::create_authenticated_clob_client;
use polymarket_toolkits::config::BotConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = BotConfig::load()?;
    
    // 创建认证客户端
    let (client, credentials) = create_authenticated_clob_client(
        config.private_key.clone(),
        config.funder_address.clone(),
    ).await?;
    
    // 使用客户端进行交易操作...
    
    Ok(())
}
```

## 架构设计

### 核心组件

```
src/
├── main.rs           # 主入口点和 WebSocket 事件循环
├── lib.rs            # 核心库导出和工具函数
├── client.rs         # CLOB 客户端创建和认证
├── processor.rs      # 订单处理和执行逻辑
├── risk_guard.rs     # 风险管理和熔断器
├── market_cache.rs   # 市场数据缓存层
├── orders.rs         # 订单下达和管理
├── config.rs         # 配置管理
├── models.rs         # 数据模型和类型
└── settings.rs       # 应用程序设置和常量
```

### 数据流程

```
WebSocket 事件 → 事件解析器 → 风险防护 → 订单处理器 → CLOB API
                                                      ↓
                                              重新提交队列
```

### 关键设计决策

- **异步优先**：所有 I/O 操作均为异步，实现最大吞吐量
- **线程安全**：共享状态使用 `Arc` 实现安全并发访问
- **错误处理**：使用 `anyhow::Result` 进行全面的错误处理
- **资源管理**：连接池和高效内存使用

## 安全与风险管理

### 熔断器系统

熔断器在以下情况下自动停止交易：
- 在时间窗口内连续发生多次大额交易
- 订单簿深度不足
- 市场条件变得不利

### 风险防护功能

- **快速路径检查**：对小额交易的快速风险评估
- **订单簿验证**：对较大交易的深度检查
- **触发持续时间**：熔断器触发后可配置的冷却期
- **连续交易跟踪**：监控交易序列以进行模式检测

### 最佳实践

1. **从小开始**：首先使用 `mock_trading: true` 测试您的设置
2. **密切监控**：在前几笔真实交易中密切关注日志
3. **设置限制**：配置适当的熔断器阈值
4. **定期更新**：保持依赖项更新以获得安全补丁
5. **资金备份**：永远不要使用超过您能承受损失的资金

## 性能指标

### 基准测试

- **事件处理**：每个事件 < 1ms
- **订单执行**：端到端延迟 < 100ms
- **内存使用**：基线约 50MB，随缓存大小扩展
- **CPU 使用**：在现代硬件上 < 5%

### 优化建议

- 在生产环境中使用发布版本：`cargo build --release`
- 根据您的市场覆盖范围调整缓存大小
- 监控连接池指标
- 使用适当的 WebSocket ping 间隔

## 故障排除

### 常见问题

**"INSUFFICIENT_BALANCE/ALLOWANCE"（余额/授权不足）**
- 确保钱包在 Polygon 上有 USDC
- 在 Polymarket.com 授权交易所
- 检查 `funder_address` 是否与您的代理钱包匹配

**"RISK_BLOCKED"（风险阻止）**
- 熔断器已触发
- 等待触发持续时间或调整阈值
- 检查订单簿深度要求

**"NETWORK"（网络错误）或 "HTTP_ERROR"（HTTP 错误）**
- 检查互联网连接
- 验证 API 端点是否可访问
- 检查 Alchemy API 密钥的有效性

**WebSocket 断开连接**
- 正常行为 - 自动重连
- 检查网络稳定性
- 检查 ping 超时设置

## 贡献指南

欢迎贡献！请遵循以下指南：

1. **Fork 仓库**
2. **创建功能分支**：`git checkout -b feature/amazing-feature`
3. **遵循 Rust 约定**：运行 `cargo fmt` 和 `cargo clippy`
4. **添加测试**：为新功能添加测试
5. **更新文档**：根据需要更新文档
6. **提交 Pull Request**

### 开发环境设置

```bash
# 安装开发依赖
cargo install cargo-watch  # 可选：用于自动重新编译

# 运行测试
cargo test

# 检查格式
cargo fmt --check

# 运行 linter
cargo clippy -- -D warnings
```

## 许可证

本项目采用 MIT 许可证 - 有关详细信息，请参阅 LICENSE 文件。

## 免责声明

⚠️ **交易风险警告**：本软件仅供教育和研究目的。交易加密货币和预测市场涉及重大损失风险。过往表现不能保证未来结果。使用风险自负。

- **无担保**：软件按"原样"提供，不提供任何形式的担保
- **非财务建议**：这不是投资或财务建议
- **合规性**：确保遵守当地法规和 Polymarket 的服务条款
- **测试**：在使用真实资金之前，始终在模拟模式下进行充分测试

## 支持

如有问题、疑问或贡献：

- **问题反馈**：[GitHub Issues](https://github.com/your-repo/issues)
- **电子邮件**：10xAngel.dev@gmail.com

## 致谢

- 使用 [Polymarket Client SDK](https://github.com/Polymarket/polymarket-client-sdk-rs) 构建
- 由 [Tokio](https://tokio.rs/) 异步运行时提供支持
- 使用 [Alloy](https://github.com/alloy-rs/alloy) 进行以太坊交互

---

<div align="center">

**为 Polymarket 社区用心制作 ❤️**

[⬆ 返回顶部](#polymarket-工具包)

</div>
