# Prediction Market Toolkits

<div align="center">

<img width="1472" height="615" alt="Polymarket Toolkits TUI" src="https://github.com/user-attachments/assets/b6c51ba1-14c6-4582-858c-e9441516dd1d" />
<img width="1224" height="843" alt="image" src="https://github.com/user-attachments/assets/66d9cb72-e14a-414f-93e5-600fb1d3f49f" />

### Multi-venue prediction market trading infrastructure — Polymarket · Kalshi · Limitless

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Tokio](https://img.shields.io/badge/async-tokio-blue.svg?style=flat-square)](https://tokio.rs/)
[![Polymarket](https://img.shields.io/badge/venue-Polymarket-6e40c9.svg?style=flat-square)](https://polymarket.com)
[![Kalshi](https://img.shields.io/badge/venue-Kalshi-0066cc.svg?style=flat-square)](https://kalshi.com)
[![Limitless](https://img.shields.io/badge/venue-Limitless-00b894.svg?style=flat-square)](https://limitless.exchange)

[Strategies](#strategies) • [Engine](#engine) • [Safety](#safety) • [Contact](#contact)

**🌐 Language / 语言:** [English](#polymarket-toolkits) • [简体中文](README.zh-CN.md)

</div>

---

## Strategies

A complete suite of ten production-grade trading bots, each engineered around a distinct, well-defined market edge. Every strategy runs on the same battle-tested execution core, risk layer, and venue-agnostic adapter stack — so you get consistent performance, unified risk controls, and a single operational surface across every play in the book. Pick the edge that fits your thesis; the infrastructure is already built.


### 1. Copy Trading

> Mirror top wallets automatically with configurable sizing and risk limits.

<img width="1843" height="879" alt="image" src="https://github.com/user-attachments/assets/57d64038-9567-4bf9-8954-a83e737ca416" />


🎯 **Outsource alpha to wallets that already proved they have it.** Zero research, zero chart-watching, zero second-guessing — the bot copies, you compound.

Tracks one or more high-performing wallets and mirrors BUY/SELL actions. Copy percentage, minimum trade size, and circuit breaker thresholds are all configurable.

| | |
|---|---|
| **Tracked wallets** | Multiple simultaneous |
| **Order types** | FAK / GTD |
| **Risk limits** | Circuit breaker + depth guard |
| **Dry run** | Fully supported |
| **Status** | ✅ Production-ready |

[contact](https://t.me/haredoggy)

---

### 2. BTC 5-min / 15-min / 1hr Arbitrage

> Speed on short-window BTC Up/Down markets.

<p align="center">
  <img width="49%" alt="image" src="https://github.com/user-attachments/assets/11df1045-7782-4085-bf7c-cae6d381273f" />
  <img width="49%" alt="image" src="https://github.com/user-attachments/assets/246c962b-a54e-497d-a40d-3812d447f4c1" />
</p>

⚡ **42ms end-to-end — the bot is in the order book before you'd finish reading the market title.** Human reflexes don't compete here.

Watches BTC Up/Down windows. When a pricing inefficiency or directional setup emerges, the bot places a low-latency FAK before the window closes. Dry-run and live modes.

| | |
|---|---|
| **Markets** | BTC Up/Down — 5m, 15m, 1hr |
| **Order type** | FAK |
| **Execution** | ~42ms end-to-end |
| **Status** | ✅ Production-ready |

[contact](https://t.me/haredoggy)

---

### 3. Polymarket ↔ Kalshi Cross-Market Arbitrage

> Cross-venue pricing inefficiencies on 15-min windows.

<img width="1543" height="654" alt="image" src="https://github.com/user-attachments/assets/2108db5d-369d-4f80-9e6e-2b03e63e291a" />


💰 **Lock the spread, not the direction.** Both legs hedged — your P&L is the price gap itself, regardless of where the underlying actually goes.

Monitors the same market on both venues. When a configurable price delta is detected, the bot executes hedged legs — buying the cheaper side and selling the expensive side — locking in the spread.

| | |
|---|---|
| **Venues** | Polymarket ↔ Kalshi |
| **Edge threshold** | Configurable (e.g. ≥ 0.8¢) |
| **Logging** | Full P&L tracking |
| **Status** | ✅ Production-ready |

[contact](https://t.me/haredoggy)

---

### 4. Direction Hunting

> Short-window momentum and flow setups.

<img width="1190" height="855" alt="image" src="https://github.com/user-attachments/assets/f18aa0ee-a357-41c7-bdb1-c2b9006ebc06" /><img width="1190" height="855" alt="image" src="https://github.com/user-attachments/assets/0b793b61-8274-445b-b610-bc7cc7b601b9" />

🎯 **Define the edge once — let the engine grind 24/7.** Entries, TP, and SL are fully automated, so you keep your weekends and the bot keeps the screen.

Continuously scans symbols and time windows for setups matching your criteria. On signal, enters and manages exits via configurable TP/SL. Real-time alerts.

| | |
|---|---|
| **Windows** | 5m, 15m (configurable) |
| **Entry criteria** | Configurable momentum / flow |
| **Exit** | TP + SL, auto-exit |
| **Status** | ✅ Production-ready |

[contact](https://t.me/haredoggy)

---

### 5. Spread Farming

> Systematic, repeatable micro-edges.

<img width="1052" height="798" alt="image" src="https://github.com/user-attachments/assets/6b231a2d-2e39-422e-b929-87f676289b58" />

📈 **A thousand 0.5¢ wins compound into one big number.** Disciplined, repeatable, boring in the best way — the kind of edge that survives every market regime.

Farms the bid-ask spread with rule-based entries and exits. Sits at the spread, waits for fill conditions to align, executes with consistent sizing. Per-trade and session P&L.

| | |
|---|---|
| **Edge** | Bid-ask spread, repeatable |
| **Logging** | Per-trade P&L + session totals |
| **Status** | ✅ Production-ready |

[contact](https://t.me/haredoggy)

---

### 6. Sports Betting Execution

> Click-to-bet speed on live sports markets.

<img width="1175" height="852" alt="image" src="https://github.com/user-attachments/assets/174ed883-5153-4114-87b8-5e4e76a20cbc" />

🏆 **Click. Filled. Done — in under 50ms.** Beat the line move that costs every other manual bettor their edge before they've even confirmed the order.

A focused live-sports interface that combines real-time odds with fast FAK execution. Pick a match, choose YES or NO, set size, hit Execute — order placed in under 50ms.

| | |
|---|---|
| **Sports** | NBA, NFL, Soccer, and more |
| **Execution** | < 50ms |
| **Status** | ✅ Production-ready |

[contact](https://t.me/haredoggy)

---

### 7. Resolution Sniper

> High win-rate, low-variance plays unique to prediction markets.

<img width="1052" height="798" alt="image" src="https://github.com/user-attachments/assets/6b231a2d-2e39-422e-b929-87f676289b58" />

🎯 **95%+ probabilities at 95¢ → ride to the guaranteed $1.00 payout.** The closest thing to free money that any market has ever offered — and it doesn't exist outside prediction markets.

Scans active markets for outcomes trading at near-certainty (e.g. ≥ 95% YES or ≤ 5% NO). On qualifying setups, buys the near-certain side and holds to the guaranteed $1.00 payout. No equivalent in traditional finance.

| | |
|---|---|
| **Certainty threshold** | Configurable |
| **Payout** | $1.00 per share at resolution |
| **Venues** | Polymarket · Kalshi · Limitless |
| **Status** | ✅ Production-ready |

[contact](https://t.me/haredoggy)

---

### 8. Orderbook Imbalance

> Pure order-flow signal, no external data required.

<img width="1017" height="789" alt="image" src="https://github.com/user-attachments/assets/05d3e6c5-ec26-420a-9cd6-a95ca6d0fff8" />

📊 **No subscriptions, no external feeds, no broken APIs.** The signal *is* the order book — self-contained, bulletproof, and impossible to front-run because nobody else can see what you see.

Monitors live bid/ask depth ratio (OBI). When OBI exceeds threshold, the bot fades into the dominant side. Signal derives entirely from the live orderbook at 500ms refresh — self-contained, no external feeds.

| | |
|---|---|
| **Signal source** | Live orderbook only |
| **Refresh rate** | 500ms |
| **Venues** | Polymarket · Kalshi · Limitless |
| **Status** | ✅ Production-ready |

[contact](https://t.me/haredoggy)

---

### 9. Market Making

> Passive spread income on illiquid prediction markets.

<img width="1013" height="784" alt="image" src="https://github.com/user-attachments/assets/97c31543-317b-4619-8421-8e9510c05e84" />

💰 **Be the house, not the gambler.** Quote both sides, earn the spread on every single fill — direction-agnostic income that scales with volume, not with luck.

Continuously quotes both sides with GTD orders. Inventory skewing rebalances quote prices when one side fills too heavily. Auto-cancels the opposite leg on fill.

| | |
|---|---|
| **Order management** | Auto-cancel on fill, auto-requote |
| **Inventory control** | Configurable skew limits |
| **Venues** | Polymarket · Kalshi |
| **Status** | ✅ Production-ready |

[contact](https://t.me/haredoggy)

---

### 10. On-Chain Whale Signal

> Fastest possible signal — 3–30 seconds ahead of the positions API.

<img width="791" height="449" alt="image" src="https://github.com/user-attachments/assets/c549cd4b-f40a-4253-a4b7-601cce44160b" />

⚡ **3–30 seconds ahead of every other tracker on the planet.** Direct from Polygon block data — you see the whale's order before the public positions API even acknowledges it exists.

Subscribes directly to Polygon block data and filters for transactions from tracked large wallets interacting with the Polymarket CLOB contract. On detection, decodes calldata (token ID, size, side) and mirrors immediately — typically 3–30s before the change appears in the public positions API.

| | |
|---|---|
| **Signal source** | Polygon on-chain block subscription |
| **Lead time** | 3–30s over positions API |
| **Detection** | ABI calldata decoding |
| **Status** | ✅ Production-ready |

[contact](https://t.me/haredoggy)

---

## Engine

### Performance

| | |
|---|---|
| **Event processing** | < 1ms per event |
| **Order execution** | < 100ms end-to-end |
| **Position polling** | ~200ms per wallet |
| **Memory** | ~50MB baseline |
| **CPU** | < 5% on modern hardware |
| **Concurrency** | Semaphore-based rate limiting (default: 25 req / 10s) |

---

## Safety

| | |
|---|---|
| **Circuit Breaker** | Auto-halts after N consecutive large trades inside a configurable window |
| **Depth Guard** | Validates orderbook liquidity before every order |
| **Dry Run** | Full execution path runs without placing real orders |
| **Trade Floor** | Minimum size enforcement against negative-EV micro-trades |

The circuit breaker fires when consecutive large trades exceed the configured threshold, or when orderbook depth falls below the minimum. Once tripped, execution is blocked for the cooldown duration. Trip state and cooldown are logged and visible in the TUI.

**Recommendations:**

| Stage | Action |
|-------|--------|
| Initial setup | Run with `enable_trading: false` for a full session |
| First real trades | Keep `copy_percentage` at 5–10% until you trust the signal |
| Ongoing | Watch circuit breaker trips — they surface execution anomalies |
| Production | Dedicated wallet with only the capital you intend to deploy |

---

## Contact

Built and maintained actively. If you're working on Polymarket tooling, algorithmic strategies, or want to collaborate:

<div align="center">

| Platform | Link |
|----------|------|
| **Discussions** | [GitHub Discussions](../../discussions) |
| **Telegram** | [@haredoggy](https://t.me/haredoggy) |

*Response time is typically within a few hours. Open to questions, feedback, and serious collaborations.*

</div>

---

## Disclaimer

> Trading prediction markets involves real financial risk. This software is provided as-is, without warranty or guarantee of any outcome. It is not financial advice. Always test with `enable_trading: false` before deploying real capital. Ensure compliance with Polymarket's terms of service and applicable regulations in your jurisdiction.

---

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)

**Built for the Prediction Markets including Polymarket, Kalshi, Limitless etc**

[Back to top](#polymarket-toolkits)

</div>

[Power of Bot](http://x.com/theparuchh/status/2053766299281416621)