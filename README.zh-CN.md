# Polymarket 工具包

<div align="center">

<img width="1472" height="615" alt="Polymarket 工具包 TUI" src="https://github.com/user-attachments/assets/b6c51ba1-14c6-4582-858c-e9441516dd1d" />

### 多平台预测市场交易基础设施 — Polymarket · Kalshi · Limitless

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Version](https://img.shields.io/badge/version-1.0.0-brightgreen.svg?style=flat-square)](https://github.com)
[![Tokio](https://img.shields.io/badge/async-tokio-blue.svg?style=flat-square)](https://tokio.rs/)
[![Polymarket](https://img.shields.io/badge/venue-Polymarket-6e40c9.svg?style=flat-square)](https://polymarket.com)
[![Kalshi](https://img.shields.io/badge/venue-Kalshi-0066cc.svg?style=flat-square)](https://kalshi.com)
[![Limitless](https://img.shields.io/badge/venue-Limitless-00b894.svg?style=flat-square)](https://limitless.exchange)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macOS%20%7C%20windows-lightgrey.svg?style=flat-square)]()

[策略](#策略) • [引擎特性](#引擎特性) • [快速开始](#安装指南) • [配置说明](#配置说明) • [架构设计](#架构设计) • [安全与风险](#安全与风险管理) • [联系方式](#联系方式)

---

**🌐 Language / 语言:** [English](README.md) • [简体中文](#polymarket-工具包)

</div>

---

## 项目概述

基于 Rust 构建的生产级预测市场交易引擎，支持 [Polymarket](https://polymarket.com)、[Kalshi](https://kalshi.com) 和 [Limitless](https://limitless.exchange)。内置实时终端界面（TUI）、完整可用的跟单机器人、多平台适配层，以及为追求速度与安全并重的交易者打造的可扩展架构。

> **v1 已上线跟单功能。** 另有九种策略正在积极开发中：BTC 套利、跨平台套利（PM ↔ Kalshi）、方向猎取、价差耕作、体育执行、结算狙击、订单簿失衡、做市商、链上鲸鱼信号。所有策略共享同一 Rust 引擎、风控层与 TUI。

---

## 背景与动机

### 市场已经变了

2026 年初，Polymarket 移除了加密市场吃单约 500ms 的人为延迟。这一小改动让整类依赖延迟的策略一夜失效——微套利、快速吃单、撤单窗口期玩法，全部失去结构性优势。

率先适应的交易者转向了**信号跟随**：追踪持续产生 alpha 的钱包，跟随其仓位，用判断力替代毫秒。

这套工具包就是那层基础设施。为当下的市场现实而建，而非六个月前的那个世界。

### 为什么选 Rust？

不是为了噱头，而是为了保证。

Rust 的所有权模型在二进制运行前就消除了整类运行时错误。没有 GC 停顿，没有并发下单的数据竞争，没有空指针导致的意外崩溃。

Tokio 驱动的异步运行时，让并行仓位轮询、订单执行和 WebSocket 流在没有线程开销的情况下协调运转——精简、可预期，在压力下也稳定。

---

## 策略

十种机器人，各自针对不同的交易优势。底层共用同一套引擎。

---

### 1. BTC 5分钟 / 15分钟 / 1小时 套利机器人

> **适合：** 需要速度的短窗口 BTC Up/Down 市场交易者

<img src="docs/images/btc-arbitrage.svg" alt="BTC 套利机器人" width="100%"/>

监控 5分钟、15分钟、1小时时间窗口的 BTC Up/Down 市场。当出现定价低效或方向性机会时，机器人在窗口关闭前以低延迟发出 FAK 订单。支持模拟运行与实盘两种模式，上线前可充分验证行为。

| | |
|---|---|
| **市场** | BTC Up/Down — 5m、15m、1hr |
| **订单类型** | FAK（即时成交或取消） |
| **执行延迟** | ~42ms |
| **模式** | 模拟运行 + 实盘 |
| **状态** | 🚧 开发中 |

---

### 2. Polymarket ↔ Kalshi 跨平台套利机器人

> **适合：** 捕捉 15分钟窗口内的跨平台定价差异

<img src="docs/images/cross-market-arbitrage.svg" alt="跨平台套利机器人" width="100%"/>

同时监控 Polymarket 和 Kalshi 上的同一市场。当可配置价差（边缘阈值）被触发时，机器人在两个平台执行对冲腿——买入便宜端，卖出昂贵端，锁定价差。每笔执行与盈亏结果均完整记录。

| | |
|---|---|
| **平台** | Polymarket ↔ Kalshi |
| **时间窗口** | 15分钟 |
| **边缘阈值** | 可配置（如 ≥ 0.8¢） |
| **执行方式** | 对冲腿，两平台同时执行 |
| **记录** | 完整盈亏追踪 |
| **状态** | 🚧 开发中 |

---

### 3. 方向猎取机器人

> **适合：** 寻找短窗口动量与资金流向机会的方向性交易者

<img src="docs/images/direction-hunting.svg" alt="方向猎取机器人" width="100%"/>

持续扫描多个标的和时间窗口，寻找符合可配置入场条件的机会。信号触发后，机器人开仓并自动管理出场——止盈与止损均可配置。无论是否盯盘，实时信号通知都会第一时间推送。

| | |
|---|---|
| **扫描范围** | 可配置市场列表 |
| **时间窗口** | 5m、15m（可配置） |
| **入场条件** | 可配置动量/流向规则 |
| **出场管理** | TP + SL，自动出场逻辑 |
| **提醒** | 实时信号通知 |
| **状态** | 🚧 开发中 |

---

### 4. 价差耕作机器人

> **适合：** 寻求系统性、可重复微优势的交易者

<img src="docs/images/spread-farming.svg" alt="价差耕作机器人" width="100%"/>

通过有纪律的规则驱动入场与出场来捕捉买卖价差。机器人守候在价差处，等待成交条件对齐后以一致仓位执行。每笔交易均记录盈亏，构建持续的会话数据，方便评估表现与优化参数。

| | |
|---|---|
| **策略类型** | 市价做市 / 价差捕捉 |
| **入场/出场** | 可配置规则，系统化执行 |
| **记录** | 逐笔盈亏 + 会话汇总 |
| **优势类型** | 买卖价差，可重复 |
| **状态** | 🚧 开发中 |

---

### 5. Polymarket 跟单交易机器人

> **适合：** 自动复制顶级钱包，带可配置仓位与风控

<img src="docs/images/copy-trading.svg" alt="跟单交易机器人" width="100%"/>

追踪一个或多个高表现钱包，自动镜像其买入与卖出操作。跟单比例、最小交易规模和熔断阈值均可配置——你完全控制跟随紧密程度与风险敞口。工具包中唯一已正式发布的机器人，其他所有策略都建立在它奠定的基础设施之上。

| | |
|---|---|
| **跟踪钱包** | 多钱包同时跟踪 |
| **跟单比例** | 可配置百分比 |
| **订单类型** | FAK / GTD |
| **风控** | 熔断器 + 深度防护 |
| **模拟运行** | 完全支持 |
| **状态** | ✅ 生产就绪 |

---

### 6. 体育投注执行机器人

> **适合：** 需要点击即下单速度的手动体育市场交易者

<img src="docs/images/sports-betting.svg" alt="体育投注执行机器人" width="100%"/>

专注于实时体育市场的操作界面，结合实时赔率展示与快速 FAK 执行。选择赛事、选择 YES 或 NO、设定金额，点击执行——订单在 50ms 内下达。为手动决策的交易者提供自动化系统级的执行速度与可靠性。

| | |
|---|---|
| **体育类型** | NBA、NFL、足球等 |
| **界面** | 实时比分板 + 实时赔率 |
| **订单流** | FAK / 市价单风格 |
| **执行** | < 50ms |
| **模式** | 点击即执行 |
| **状态** | 🚧 开发中 |

---

### 7. 结算狙击机器人

> **适合：** 预测市场独有的高胜率、低波动策略

<img src="docs/images/resolution-sniper.svg" alt="结算狙击机器人" width="100%"/>

扫描所有活跃市场，寻找以接近确定性价格交易的结果——可配置阈值（如 YES ≥ 95% 或 NO ≤ 5%）。当市场在结算前满足时间窗口条件时，机器人买入接近确定的一侧，持有至 $1.00 保底结算。传统金融中没有等价策略；胜率高、波动低，建议搭配严格仓位限制使用。

| | |
|---|---|
| **确定性阈值** | 可配置（如 ≥ 95%） |
| **时间窗口** | 可配置（如距结算 < 15分钟） |
| **最高买入价** | 可配置上限 |
| **结算赔付** | 每份 $1.00 |
| **支持平台** | Polymarket · Kalshi · Limitless |
| **状态** | 🚧 开发中 |

---

### 8. 订单簿失衡机器人

> **适合：** 纯订单流信号，无需外部数据源

<img src="docs/images/orderbook-imbalance.svg" alt="订单簿失衡机器人" width="100%"/>

监控配置市场的实时买卖挂单深度比（订单簿失衡度 OBI）。当 OBI 超过可配置阈值时，机器人顺势进入主导方向——跟随厚买单做多，跟随厚卖单做空。信号完全来自 500ms 刷新的实时订单簿，无需任何外部数据源，策略自成一体、延迟极低。

| | |
|---|---|
| **信号来源** | 仅实时订单簿 |
| **OBI 阈值** | 可配置（如 ≥ 60%） |
| **刷新频率** | 500ms |
| **方向** | 顺势跟随主导方 |
| **支持平台** | Polymarket · Kalshi · Limitless |
| **状态** | 🚧 开发中 |

---

### 9. 做市商机器人

> **适合：** 在流动性不足的预测市场被动赚取价差收益

<img src="docs/images/market-making.svg" alt="做市商机器人" width="100%"/>

持续在流动性较低的市场双边挂单，被动赚取价差。与价差耕作（主动吃单）不同，本机器人**就是**订单簿本身——同时以 GTD 订单挂买单与卖单。库存倾斜机制在某侧成交过多时自动调整报价，重新平衡 YES/NO 敞口。成交后自动撤销对面挂单。

| | |
|---|---|
| **策略类型** | 双边 GTD 报价 |
| **订单管理** | 成交后自动撤单、自动补单 |
| **库存控制** | 可配置倾斜限制 |
| **记录** | 逐笔盈亏 + 胜率追踪 |
| **支持平台** | Polymarket · Kalshi |
| **状态** | 🚧 开发中 |

---

### 10. 链上鲸鱼信号机器人

> **适合：** 比仓位 API 快 3–30 秒的最高时效信号

<img src="docs/images/whale-signal.svg" alt="链上鲸鱼信号机器人" width="100%"/>

直接订阅 Polygon 区块数据，过滤被追踪大钱包与 Polymarket CLOB 合约的交互交易。当鲸鱼交易上链时，机器人解码 calldata（token ID、数量、方向）并立即镜像下单——通常比公开仓位 API 快 3–30 秒。这是目前 Polymarket 上可获取的最高精度、最低延迟信号源。

| | |
|---|---|
| **信号来源** | Polygon 链上区块订阅 |
| **领先优势** | 比仓位 API 快 3–30 秒 |
| **检测方式** | ABI calldata 解码 |
| **网络** | Polygon（Matic） |
| **风控** | 完整熔断器 + 深度防护 |
| **状态** | 🚧 开发中 |

---

## 引擎特性

### 核心能力

| | 说明 |
|---|---|
| **跟单交易机器人** | 同时追踪多个钱包，按可配置比例镜像仓位变化 |
| **终端用户界面** | 基于 `ratatui` 的 TUI，实时日志流、按级别着色、机器人选择菜单 |
| **FAK / GTD 订单** | 即时成交或取消 / 有效期订单，自动解析市场 ID |
| **多钱包追踪** | 任意数量地址并发轮询，共享限流 |

### 安全与风控

| | 机制 |
|---|---|
| **熔断器** | 在可配置时间窗口内连续 N 笔大额交易后自动停止 |
| **订单簿深度防护** | 每笔订单前验证流动性，不在薄盘中成交 |
| **模拟运行模式** | 完整执行路径运行但不下真实订单，适合验证 |
| **最小交易规模** | 强制最低交易规模，避免负期望的微量交易 |

### 性能指标

| | 数据 |
|---|---|
| **事件处理** | < 1ms/事件 |
| **订单执行** | 端到端 < 100ms |
| **仓位轮询** | 约 200ms/钱包 |
| **内存** | 约 50MB 基线 |
| **CPU** | 现代硬件 < 5% |
| **并发** | 信号量限流（默认 25 次/10 秒） |

---

## 安装指南

### 前置要求

- **Rust 1.70+** — [安装地址](https://www.rust-lang.org/zh-CN/tools/install)
- **Polymarket 账户**，Polygon 上有 USDC
- **交易所授权** — 在 [polymarket.com](https://polymarket.com) 至少完成一笔交易以授权 USDC 支出

### 构建与运行

```bash
# 克隆仓库
git clone <repository-url>
cd Polymarket-Toolkits

# 生产构建（推荐）
cargo build --release
cargo run --release

# 调试构建
RUST_LOG=debug cargo run
```

---

## 配置说明

项目采用**双文件分离**：公开设置存放于 `config.json`，敏感凭证存放于 `config.yaml`。

```bash
cp config.yaml.example config.yaml
```

### `config.yaml` — 凭证（切勿提交）

```yaml
bot:
  private_key: "your_64_character_hex_private_key"   # 不含 0x 前缀
  funder_address: "0x0000000000000000000000000000000000000000"
```

### `config.json` — 所有其他设置

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
    "poll_interval": 5,
    "price_buffer": 0.00,
    "scaling_ratio": 1.00,
    "min_cash_value": 0.00,
    "min_share_count": 0.0
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

**`bot`**
- `wallets_to_track` — 要镜像的钱包地址列表
- `enable_trading` — `false` = 仅监控，不下单

**`trading`**
- `copy_percentage` — 复制被跟踪钱包仓位的百分比
- `poll_interval` — 仓位检查间隔（秒，越小越灵敏）
- `price_buffer` — 订单价格滑点容忍度
- `scaling_ratio` — 在跟单比例基础上叠加的倍数

**`risk`**
- `large_trade_shares` — 触发"大额交易"标记的最小份额数
- `consecutive_trigger` — 连续几笔大额交易触发熔断
- `sequence_window_secs` — 连续交易统计的滚动时间窗（秒）
- `min_depth_usd` — 继续下单所需的最小订单簿流动性
- `trip_duration_secs` — 熔断触发后的冷却时长

> **安全提醒：** `config.yaml` 已默认加入 `.gitignore`。生产部署请使用环境变量或密钥管理系统，切勿在共享环境中明文存储私钥。

---

## 使用指南

### 首次运行

1. 在 `config.json` 中设置 `"enable_trading": false`
2. 运行 `cargo run --release`，确认仓位检测正常工作
3. 查看几个轮询周期的日志，确认钱包与仓位大小符合预期
4. 准备好后，将 `"enable_trading"` 设为 `true`

### TUI 键盘操作

| 按键 | 操作 |
|------|------|
| `↑ / ↓` | 在机器人菜单中导航 |
| `Enter` | 启动选中的机器人 |
| `q` | 退出 |
| `Esc` | 从机器人界面返回 |

### 日志示例

```
[2026-02-16 10:30:45] INFO    初始化跟单交易机器人...
[2026-02-16 10:30:46] INFO    追踪 0x63ce34... — 15 个持仓
[2026-02-16 10:30:51] INFO    检测到买入：100.5 份 — "Bitcoin Up or Down"
[2026-02-16 10:30:51] INFO    复制：btc-updown-15m-1771884900 → 20.1 份
[2026-02-16 10:30:52] SUCCESS  订单确认。ID：0x1234...
```

---

## 架构设计

代码库围绕三个层次组织：**平台适配层**（各平台 API 客户端）、**共享服务层**（执行、缓存、风控）、**机器人层**（薄策略编排层，组合各服务）。

```
src/
├── main.rs                          # 入口 + TUI 事件循环
├── lib.rs                           # 核心库导出
│
├── config/
│   ├── mod.rs                       # AppConfig — 统一配置（json + yaml）
│   ├── settings.rs                  # 所有配置结构体与默认值
│   └── coin.rs                      # 币种/市场选择辅助
│
├── venues/                          # 各平台 API 适配层
│   ├── mod.rs                       # VenueId、Side、MarketRef — 共享类型
│   ├── polymarket/
│   │   └── mod.rs                   # 复用现有 service 层
│   ├── kalshi/
│   │   ├── mod.rs
│   │   └── client.rs                # Kalshi REST 客户端 + 订单类型
│   └── limitless/
│       ├── mod.rs
│       └── client.rs                # Limitless 链上客户端（Base）
│
├── bot/                             # 策略实现
│   ├── mod.rs                       # 模块导出 + 状态说明
│   ├── copy_trading.rs              # ✅ 跟单交易（生产就绪）
│   ├── arbitrage.rs                 # 🚧 BTC Up/Down 套利（5m / 15m / 1hr）
│   ├── cross_market_arb.rs          # 🚧 跨平台套利（PM ↔ Kalshi）
│   ├── direction_hunting.rs         # 🚧 方向猎取 + TP/SL
│   ├── spread_farming.rs            # 🚧 系统化价差捕捉
│   ├── sports_execution.rs          # 🚧 点击即 FAK 体育界面
│   ├── resolution_sniper.rs         # 🚧 接近确定性市场狙击
│   ├── orderbook_imbalance.rs       # 🚧 OBI 驱动订单流信号
│   ├── market_maker.rs              # 🚧 双边 GTD 流动性提供
│   └── whale_signal.rs              # 🚧 Polygon 链上 TX 监控
│
├── service/                         # 共享执行服务
│   ├── client.rs                    # Polymarket CLOB 客户端 + 认证
│   ├── trader.rs                    # CopyTrader 执行引擎
│   ├── positions.rs                 # 仓位轮询 + 变化检测
│   ├── orders.rs                    # FAK/GTD 订单下单
│   ├── market_cache.rs              # 市场元数据缓存（slug → token ID）
│   ├── token.rs                     # Token ID 辅助
│   └── price_feed.rs                # 跨平台实时价格聚合
│
├── ui/
│   ├── layout.rs                    # TUI 布局 + 键盘事件
│   └── components/logs.rs           # 日志条目渲染
│
├── utils/
│   ├── keyboard.rs                  # 原始终端输入
│   ├── risk_guard.rs                # 熔断器 + 多层风控
│   └── orderbook.rs                 # OBI、价差、深度分析辅助
│
└── models.rs                        # 共享数据类型
```

### 执行流程（跟单交易机器人）

```
加载配置（config.json + config.yaml）
    ↓
初始化 CopyTrader — 已认证的 Polymarket CLOB 客户端
    ↓
仓位轮询循环（每 poll_interval 秒）
    ↓
对比：前一状态 vs 当前状态
    ↓
对每个变化：
    ├─ 计算仓位大小（copy_percentage × 被追踪仓位）
    ├─ 解析市场 ID（Gamma API，带缓存）
    ├─ 风控检查（熔断器 + 深度）
    └─ 下单执行（FAK/GTD，并发，限流）
    ↓
更新缓存 → 重复轮询
```

### 设计原则

- **异步优先** — 所有 I/O 通过 Tokio；热路径中无阻塞调用
- **平台无关的机器人** — 策略引用 `venues::VenueId`，而非特定 API 类型
- **线程安全的共享状态** — `Arc<Mutex<T>>` 用于仓位缓存和熔断器
- **模块化服务** — 机器人是薄编排层；执行逻辑在服务层
- **响亮失败** — `anyhow::Result` 传播，每个错误路径均有完整上下文
- **最小化分配** — 优先使用栈缓冲；仅在必要时使用堆

---

## 安全与风险管理

### 熔断器

在以下情况自动触发：
- 在 `sequence_window_secs` 内连续发生 `consecutive_trigger` 笔大额交易（≥ `large_trade_shares`）
- 订单簿深度低于 `min_depth_usd`

触发后，所有订单执行被阻止 `trip_duration_secs` 秒。熔断状态与冷却时间在 TUI 中可见。

### 建议做法

| 阶段 | 操作 |
|------|------|
| 初始配置 | 至少用 `enable_trading: false` 运行一个完整会话 |
| 首笔实盘 | 将 `copy_percentage` 控制在 5–10%，直到信任信号 |
| 持续运营 | 关注熔断触发情况，它们能暴露执行异常 |
| 生产环境 | 使用专用钱包，仅部署计划用于交易的资金 |

---

## 故障排除

**`Failed to parse positions response as JSON`**
→ 检查 `data_api_base` URL 和 API 可访问性，确认限流设置。

**`Failed to get market_id for slug`**
→ 检查 `gamma_api_base`、网络连通性和 API 限制。

**`Trade execution returned None`**
→ 确认 `enable_trading: true`、USDC 余额充足且已完成交易所授权。

**`INSUFFICIENT_BALANCE/ALLOWANCE`**
→ 在 Polymarket.com 授权交易所，确认 `funder_address` 与代理钱包一致。

**`RISK_BLOCKED`**
→ 熔断器已激活，等待冷却或检查 `trip_duration_secs`。

**TUI 无法显示**
→ 终端需支持 ANSI 颜色，最低建议尺寸：80×24。

---

## 贡献

欢迎提交 Pull Request。提交前请确认：

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

分支命名规范：推荐使用 `feature/`、`fix/`、`refactor/` 前缀。

---

## 联系方式

项目持续活跃维护中——如果你在做 Polymarket 工具开发、算法策略研究，或有合作意向，欢迎联系：

<div align="center">

| 平台 | 链接 |
|------|------|
| **讨论区** | [GitHub Discussions](../../discussions) |
| **Telegram** | [@haredoggy](https://t.me/haredoggy) |
| **Discord** | [发消息](https://discord.com/users/1114372741672488990) |
| **WhatsApp** | [+1 (628) 666-6724](https://wa.me/16286666724?text=Hello%20there) |

*通常几小时内回复。欢迎问题反馈与严肃合作。*

</div>

---

## 致谢

- [py-clob-client](https://github.com/Polymarket/py-clob-client) — CLOB 交互参考实现
- [Tokio](https://tokio.rs/) — 异步运行时
- [Alloy](https://github.com/alloy-rs/alloy) — 以太坊基础类型
- [ratatui](https://github.com/ratatui-org/ratatui) — 终端 UI 框架

---

## 免责声明

> 预测市场交易涉及真实财务风险。本软件按现状提供，不对任何结果作出保证，亦不构成财务建议。使用真实资金前，请务必以 `enable_trading: false` 充分测试。请确保遵守 Polymarket 服务条款及所在地区的适用法规。

---

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)

**为预测市场社区而建**

[返回顶部](#polymarket-工具包)

</div>
