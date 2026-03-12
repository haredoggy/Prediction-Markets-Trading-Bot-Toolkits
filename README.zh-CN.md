# Polymarket 工具包

<div align="center">

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-1.0.0-green.svg)

<img width="1472" height="615" alt="image" src="https://github.com/user-attachments/assets/b6c51ba1-14c6-4582-858c-e9441516dd1d" />

**高性能 Rust 语言开发的 Polymarket CLOB 交易工具包**

[功能特性](#功能特性) • [安装指南](#安装指南) • [配置说明](#配置说明) • [使用指南](#使用指南) • [架构设计](#架构设计) • [安全与风险管理](#安全与风险管理)

---

### 🌐 Language / 语言

[English](README.md) • [简体中文](#polymarket-工具包)

</div>

---

## 项目概述

Polymarket 工具包是一个生产就绪的 Rust 应用程序，用于在 Polymarket 中央限价订单簿（CLOB）上进行自动化交易。提供现代化终端界面（TUI）、多种交易机器人策略、实时仓位监控、智能订单执行与完善的安全机制。

## 重要说明

本软件为**跟单机器人**，会在 Polymarket 上按所选钱包复制其仓位。当前版本（v1）不包含内置策略逻辑、风控或收益保证，盈亏取决于跟单钱包、你的仓位大小以及市场情况。你可以在现有代码基础上扩展或改写，自行实现风控与策略层。后续版本尚未公开，我们会持续更新并提供支持。

### 联系方式
- [讨论区](../../discussions)
- [WhatsApp](https://wa.me/16286666724?text=Hello%20there)
- [Telegram](https://t.me/haredoggy)
- [Discord](https://discord.com/users/1114372741672488990)

### 核心能力一览

| 方面     | 说明 |
|----------|------|
| **TUI**  | 机器人选择菜单与实时日志 |
| **跟单** | 自动跟随指定钱包的交易 |
| **安全** | 熔断器、风险防护、模拟运行模式 |
| **性能** | 异步 I/O、限流、市场数据缓存 |

- **终端用户界面** — 基于 `ratatui` 的交互式 TUI：机器人选择、实时滚动日志、按级别着色（Info / Warning / Error / Success）、时间戳。
- **跟单交易机器人** — 自动检测并复制被监控钱包的交易。
- **仓位跟踪** — 按可配置间隔轮询仓位；检测新建、平仓或数量变化；支持多钱包。
- **订单执行** — 按跟单比例计算仓位、通过 Gamma API 解析市场 ID（带缓存）、限流下的并发下单、支持买入与卖出。
- **风险与安全** — 熔断器、订单簿深度检查、风险防护、交易规模限制、模拟运行模式。
- **性能** — 异步 I/O、连接池、可配置限流、市场 ID 缓存。

---

## 背景与动机

### 为什么选择 Rust？

Rust 不仅是技术选型，更是适合这类场景的基础。在速度、精确与可信必须并存的环境里，Rust 以高性能与可靠性说话。

其所有权模型消除了整类运行时故障，为以市场节奏运行的系统带来结构上的稳定。配合 async Rust 与 Tokio 处理并发（API 调用、流与执行），在成熟工具生态之上保持协调一致。

最终，Rust 带来的不仅是速度或安全，更是信心。对实时与市场交互的软件而言，这份信心让本工具包得以快速、可靠地运行。

### Polymarket 的转变：从延迟套利到跟单交易

2026 年 2 月上旬至中旬，Polymarket 悄然移除了加密市场吃单（市价单）约 500ms 的人为延迟。这一小改动带来大影响：减少延迟套利、抑制利用延迟的机器人、提升市场公平与效率。

**发生了什么变化？**

大量依赖延迟的策略失去优势。依赖该延迟的机器人与策略（如微套利、快速吃单、做市方撤单窗口等）变得不可靠。依赖结构性时间优势的经典 HFT 玩法在快速波动的短期市场中大多式微。

**当前趋势**

许多交易者不再追逐日益缩小的微观低效或重建复杂做市策略，而是转向跟单交易——跟随已验证钱包的 alpha。思路很直接：

- 通过跟踪表现稳定的交易者来跟随聪明资金
- 受益于由人类判断与仓位驱动的真实市场时机
- 无需重基础设施或延迟调优，保持简单
- 通过增加优质钱包自然扩展

### 为什么先做跟单交易机器人？

跟单交易机器人是本工具包中第一个完整可用的机器人，因为：

1. **市场现实**：延迟取消后，跟单是当前最可行的自动化策略之一
2. **易用性**：比复杂套利策略更易理解与配置
3. **风险管理**：可通过跟单比例与钱包选择控制风险
4. **透明度**：所有交易可记录、可查看，便于审计与改进
5. **基础能力**：跟单所需的基础设施（仓位跟踪、订单执行）也是未来其他机器人的基础

本工具包旨在通过稳健、可靠的跟单工具，帮助交易者适应新的 Polymarket 环境——利用成功交易者的智慧，而非参与延迟军备竞赛。

### 这个机器人是什么？

它是一个简单的跟单机器人，跟随目标钱包的动向。没有内置策略、没有复杂风控，也不承诺收益——这是 v1。盈亏取决于你跟的钱包、仓位大小与市场环境。你可以在其之上构建自己的用法。后续版本会持续迭代改进。

---

## 功能特性

### 🚀 核心功能

- **多种机器人类型**
  - **跟单交易机器人**（✅ 可用）：自动复制被跟踪钱包的交易
  - **套利机器人**（🚧 计划中）：识别并执行套利机会
  - **狙击机器人**（🚧 计划中）：针对时效机会的快速下单

- **终端用户界面**
  - 交互式机器人选择菜单
  - 实时日志与滚动
  - 按级别着色（Info、Warning、Error、Success）
  - 带时间戳的日志条目

- **仓位监控**
  - 按可配置间隔轮询 Polymarket 仓位 API
  - 检测仓位变化（新建、平仓、数量变化）
  - 支持同时跟踪多个钱包地址

- **智能交易执行**
  - 可配置跟单比例
  - 通过 Gamma API 自动解析市场 ID
  - 基于信号量的限流并发执行
  - 支持买入与卖出

### 🛡️ 安全功能

- **熔断系统**：连续大额交易后自动暂停交易
- **订单簿深度检查**：下单前验证流动性
- **风险防护**：可配置阈值的多层风险评估
- **交易规模限制**：最小交易规模，避免负期望
- **交易开关**：无需改代码即可启用/禁用交易
- **模拟运行模式**：不执行真实订单即可测试配置

### ⚡ 性能特性

- **异步架构**：基于 Tokio 的高并发异步操作
- **连接池**：可复用连接的 HTTP 客户端
- **内存效率**：栈上缓冲与高效数据结构
- **限流**：可配置 API 限流（默认 25 次/10 秒）
- **市场 ID 缓存**：减少 API 调用

---

## 安装指南

### 前置要求

- **Rust**：1.70 或更高（[安装 Rust](https://www.rust-lang.org/zh-CN/tools/install)）
- **Polymarket 账户**：Polygon 上有 USDC 的钱包
- **交易所授权**：在 Polymarket 交易所授权 USDC 支出

### 从源码构建

```bash
# 克隆仓库
git clone <repository-url>
cd Polymarket-Toolkits

# 发布模式构建（优化）
cargo build --release

# 运行
cargo run --release
```

### 开发构建

```bash
# 调试模式构建（编译快、运行慢）
cargo build

# 带调试日志运行
RUST_LOG=debug cargo run
```

---

## 配置说明

### 配置文件

项目采用**分离配置**以兼顾安全与版本管理：

1. **`config.json`** — 非敏感配置（可提交到 git）
2. **`config.yaml`** — 仅敏感凭证（切勿提交到 git）

### 配置步骤

1. **复制示例配置**：
   ```bash
   cp config.yaml.example config.yaml
   # config.json 已存在默认内容
   ```

2. **编辑 `config.yaml`**（敏感信息）：
   ```yaml
   bot:
     # 钱包私钥（64 位十六进制，不要 0x 前缀）
     private_key: "your_64_character_hex_private_key"
     # 代理钱包地址（funder）
     funder_address: "0x0000000000000000000000000000000000000000"
   ```

3. **编辑 `config.json`**（交易与风险偏好）：
   ```json
   {
     "bot": {
       "wallets_to_track": ["0x63ce342161250d705dc0b16df89036c8e5f9ba9a"],
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

### 配置项说明

#### 机器人（`config.json`）

- `wallets_to_track`：要跟单的钱包地址列表
- `enable_trading`：设为 `false` 可仅监控不交易（模拟模式）

#### 站点（`config.json`）

- `gamma_api_base`：Gamma API 基础 URL（市场数据）
- `clob_api_base`：CLOB API 基础 URL（下单）
- `clob_wss_url`：WebSocket URL
- `data_api_base`：仓位 API 基础 URL（默认同 gamma_api_base）

#### 交易（`config.json`）

- `copy_percentage`：跟单比例（默认 20.0）
- `rate_limit`：最大并发请求数（默认 25）
- `poll_interval`：仓位轮询间隔（秒，默认 5）
- `price_buffer`、`scaling_ratio`、`min_cash_value`、`min_share_count` 等可选

#### 风险（`config.json`）

- `large_trade_shares`：触发熔断的最小份额（默认 1500.0）
- `consecutive_trigger`：连续几次大额交易触发（默认 2）
- `sequence_window_secs`：统计连续交易的时间窗（秒，默认 30）
- `min_depth_usd`：最小订单簿深度（美元，默认 200.0）
- `trip_duration_secs`：熔断持续时间（秒，默认 120）

### 安全注意事项

> ⚠️ **重要**：切勿将 `config.yaml` 提交到版本控制，其中包含私钥等敏感信息。
>
> - 将 `config.yaml` 加入 `.gitignore`（项目已包含）
> - `config.json` 可安全提交（无敏感内容）
> - 生产环境使用环境变量或密钥管理系统存放私钥

---

## 使用指南

### 基本使用

1. 在 `config.json` 和 `config.yaml` 中完成配置
2. 确保钱包在 Polygon 上有 USDC
3. 在 [Polymarket.com](https://polymarket.com) 完成交易所授权（可先做一笔测试交易）
4. 运行程序：
   ```bash
   cargo run --release
   ```
5. 在 TUI 中选择机器人：方向键选择，Enter 确认，`q` 退出
6. 在界面中查看实时日志（自动滚动、按级别着色、带时间戳）

### 仅监控模式

不执行订单、仅观察跟单逻辑时，在 `config.json` 中设置：

```json
{
  "bot": {
    "enable_trading": false
  }
}
```

### 跟单交易机器人流程

1. 按配置间隔轮询被跟踪钱包的仓位
2. 检测仓位变化（新建、平仓、数量变化）
3. 按 `copy_percentage` 计算下单量
4. 通过 Gamma API 解析市场 ID（使用缓存）
5. 在限流下并发下单
6. 在 TUI 中实时输出日志

示例日志：
```
[2026-02-16 10:30:45] INFO - Initializing Copy Trading Bot...
[2026-02-16 10:30:46] INFO - Initialized 0x63ce34... with 15 position(s)
[2026-02-16 10:30:51] INFO - Detected BUY for 0x63ce34...: 100.5 shares of Bitcoin Up or Down
[2026-02-16 10:30:51] INFO - Copying buy for btc-updown-15m-1771884900: 20.1 shares
[2026-02-16 10:30:52] SUCCESS - Order placed successfully. Order ID: 0x1234...
```

### 键盘操作

- **方向键**：在机器人菜单中移动
- **Enter**：选择当前项
- **q**：退出应用
- **Esc**：在机器人界面中退出

---

## 架构设计

### 项目结构

```
src/
├── main.rs              # 入口与 TUI 编排
├── lib.rs               # 库导出与工具
├── config/
│   ├── mod.rs           # 统一配置（AppConfig）
│   └── settings.rs      # 常量与设置
├── bot/
│   ├── mod.rs
│   ├── copy_trading.rs  # 跟单机器人
│   ├── arbitrage.rs     # 套利（占位）
│   └── sniper.rs        # 狙击（占位）
├── service/
│   ├── mod.rs
│   ├── client.rs        # CLOB 客户端与认证
│   ├── trader.rs        # CopyTrader 执行
│   ├── positions.rs     # 仓位获取与变化检测
│   ├── orders.rs        # 下单与管理
│   └── market_cache.rs  # 市场数据缓存
├── ui/
│   ├── mod.rs
│   ├── layout.rs        # TUI 布局与事件
│   └── components/
│       ├── mod.rs
│       └── logs.rs      # 日志组件
├── utils/
│   ├── mod.rs
│   └── risk_guard.rs    # 风险与熔断
└── models.rs            # 数据模型
```

### 跟单数据流

```
加载配置（config.json + config.yaml）
    ↓
初始化 CopyTrader（已认证 CLOB 客户端）
    ↓
按 poll_interval 轮询仓位 API
    ↓
对比前后仓位，检测变化
    ↓
对每个变化：计算跟单量 → 解析 market_id（Gamma，带缓存）→ 限流下下单 → 写 TUI 日志
    ↓
更新仓位缓存 → 重复轮询
```

### 设计要点

- **分离配置**：敏感信息与可公开配置分开
- **异步优先**：I/O 全异步以提升吞吐
- **模块划分**：机器人、服务、UI 清晰分离
- **线程安全**：共享状态使用 `Arc<Mutex<>>`
- **错误处理**：统一使用 `anyhow::Result`
- **限流与缓存**：信号量限流、市场 ID 缓存

---

## 安全与风险管理

### 熔断系统

在以下情况会自动暂停交易：

- 在时间窗内连续多次大额交易
- 订单簿深度不足
- 市场条件不利

### 风险防护

- **快速路径**：对小额交易做轻量风险判断
- **订单簿校验**：对大额交易做深度检查
- **熔断时长**：触发后可配置的冷却时间
- **连续交易**：按序列检测异常模式

### 建议做法

1. 先用 `enable_trading: false` 仅监控
2. 确认仓位与跟单逻辑正确后再开启交易
3. 前几笔实盘时密切看日志
4. 合理设置熔断与深度阈值
5. 跟单比例从小开始（如 5%–10%）
6. 定期更新依赖以获取安全修复
7. 只用可承受损失的资金

---

## 性能

### 参考指标

- **事件处理**：单事件 &lt; 1ms
- **下单**：端到端 &lt; 100ms
- **仓位轮询**：约 200ms/钱包（含限流）
- **内存**：约 50MB 基线，随缓存增长
- **CPU**：现代硬件上 &lt; 5%

### 优化建议

- 生产环境使用 `cargo build --release`
- 按需求调整 `poll_interval`（越小越及时，越大越省 API）
- 根据接口限制设置 `rate_limit`
- 关注市场 ID 缓存命中情况

---

## 故障排除

### 常见问题

**「Failed to parse positions response as JSON」**
- 检查 `config.json` 中 `data_api_base`
- 确认接口可访问与限流设置

**「Failed to get market_id for slug」**
- 检查 `gamma_api_base` 与网络
- 注意 API 限流

**「Trade execution returned None」**
- 查看具体错误日志
- 确认 `enable_trading` 为 `true`
- 确认钱包 USDC 与交易所授权

**「INSUFFICIENT_BALANCE/ALLOWANCE」**
- 确保 Polygon 上有 USDC
- 在 Polymarket.com 完成交易所授权
- 核对 `funder_address` 与代理钱包一致

**「RISK_BLOCKED」**
- 熔断已触发，等待冷却或调整阈值
- 检查订单簿深度要求

**TUI 显示异常**
- 确认终端支持 ANSI 颜色
- 建议终端至少 80×24
- 可换一个终端模拟器试试

---

## 贡献

欢迎贡献，请尽量：

1. Fork 本仓库
2. 新建分支：`git checkout -b feature/amazing-feature`
3. 遵守 Rust 惯例：`cargo fmt`、`cargo clippy`
4. 为新功能补充测试
5. 同步更新文档
6. 提交 Pull Request

### 开发环境

```bash
cargo install cargo-watch   # 可选：自动重编
cargo test
cargo fmt --check
cargo clippy -- -D warnings
RUST_LOG=debug cargo run
```

---

## 许可证

本项目采用 MIT 许可证，详见 LICENSE 文件。

## 免责声明

> ⚠️ **交易风险**：本软件仅供教育与研究。加密货币与预测市场交易存在重大亏损风险，过往表现不保证未来结果，请自行承担风险。
>
> - **无担保**：按「原样」提供，不提供任何担保
> - **非投资建议**：不构成投资或财务建议
> - **合规**：请遵守当地法规与 Polymarket 服务条款
> - **测试**：使用真实资金前请务必用 `enable_trading: false` 充分测试

## 支持

问题、建议或贡献：[参见 SUPPORT.md](SUPPORT.md)

## 致谢

- 基于 [Polymarket Client SDK](https://github.com/Polymarket/polymarket-client-sdk-rs)
- 使用 [Tokio](https://tokio.rs/) 异步运行时
- 使用 [Alloy](https://github.com/alloy-rs/alloy) 进行以太坊交互
- TUI 使用 [ratatui](https://github.com/ratatui-org/ratatui)

---

<div align="center">

**为 Polymarket 社区用心制作 ❤️**

[⬆ 返回顶部](#polymarket-工具包)

</div>
