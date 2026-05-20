import type { ReactNode } from 'react';
import { useLang, type Lang } from './i18n';

interface Spec {
  label: string;
  value: string;
}

interface BotContent {
  title: string;
  tagline: string;
  hook: string;
  description: string;
  specs: Spec[];
}

interface HeroDescParts {
  polymarket: ReactNode;
  kalshi: ReactNode;
  limitless: ReactNode;
}

interface LiveSignalFootnoteParts {
  source: ReactNode;
  chainlink: ReactNode;
}

interface FooterDisclaimerParts {
  flag: ReactNode;
}

interface Feature {
  icon: string;
  title: string;
  body: string;
}

interface Layer {
  icon: string;
  title: string;
  body: string;
  accent: string;
}

interface Metric {
  metric: string;
  label: string;
}

interface LadderStep {
  stage: string;
  action: string;
}

interface Pillar {
  title: string;
  body: string;
}

interface Stat {
  label: string;
  value: string;
  unit: string;
}

export interface Messages {
  langName: { en: string; zh: string };
  nav: {
    strategies: string;
    engine: string;
    safety: string;
    contact: string;
    github: string;
    telegram: string;
    langSwitch: string;
  };
  hero: {
    badge: string;
    headlineLine1: string;
    headlineLine2: string;
    description: (parts: HeroDescParts) => ReactNode;
    ctaTelegram: string;
    ctaGithub: string;
    stats: Stat[];
  };
  liveSignal: {
    eyebrow: string;
    headline: string;
    sub: string;
    statusLive: string;
    statusOffline: string;
    statusConnecting: string;
    pair: string;
    sessionLabel: string;
    footnote: (parts: LiveSignalFootnoteParts) => ReactNode;
    attribution: string;
  };
  bots: {
    eyebrow: string;
    headline: string;
    description: string;
    statusProduction: string;
    statusDev: string;
    cardCta: string;
    cardCtaSuffix: string;
    items: Record<string, BotContent>;
  };
  engine: {
    eyebrow: string;
    headlineLine1: string;
    headlineLine2: string;
    description: string;
    features: Feature[];
    performanceEyebrow: string;
    performanceHeadline: string;
    metrics: Metric[];
  };
  safety: {
    eyebrow: string;
    headlineLine1: string;
    headlineLine2: string;
    description: string;
    layers: Layer[];
    ladderTitle: string;
    ladderSubtitle: string;
    ladder: LadderStep[];
  };
  cta: {
    eyebrow: string;
    headline: string;
    description: string;
    ctaTelegram: string;
    ctaGithub: string;
    pillars: Pillar[];
  };
  footer: {
    productName: string;
    tagline: string;
    disclaimerLabel: string;
    disclaimer: (parts: FooterDisclaimerParts) => ReactNode;
    rights: string;
  };
}

const messages: Record<Lang, Messages> = {
  en: {
    langName: { en: 'English', zh: '简体中文' },
    nav: {
      strategies: 'Strategies',
      engine: 'Engine',
      safety: 'Safety',
      contact: 'Contact',
      github: 'GitHub',
      telegram: 'Telegram',
      langSwitch: 'Language',
    },
    hero: {
      badge: '10 strategies · one battle-tested engine',
      headlineLine1: 'Prediction markets,',
      headlineLine2: 'traded at machine speed.',
      description: ({ polymarket, kalshi, limitless }) => (
        <>
          Ten production-grade Rust trading bots for {polymarket}, {kalshi}, and {limitless}.
          {' '}Copy trading, cross-venue arb, whale signals, market making — all sharing one risk layer, one execution core.
        </>
      ),
      ctaTelegram: 'Talk on Telegram',
      ctaGithub: 'View on GitHub',
      stats: [
        { label: 'Order execution', value: '< 100ms', unit: 'end-to-end' },
        { label: 'Event processing', value: '< 1ms', unit: 'per event' },
        { label: 'Memory baseline', value: '~50MB', unit: 'resident' },
        { label: 'CPU under load', value: '< 5%', unit: 'modern hw' },
      ],
    },
    liveSignal: {
      eyebrow: 'Live signal',
      headline: 'BTC/USDT — the market the bots watch',
      sub: 'Real-time spot price, streamed directly from the order book.',
      statusLive: 'Live',
      statusOffline: 'Offline',
      statusConnecting: 'Connecting',
      pair: 'BTC / USDT',
      sessionLabel: '· session',
      footnote: ({ source, chainlink }) => (
        <>
          Source: {source}. Polymarket&apos;s BTC Up/Down markets resolve against {chainlink},
          which aggregate from Binance, Coinbase, Kraken, and other major venues — so what you see here
          is a faithful real-time proxy for the price the bots trade against.
        </>
      ),
      attribution: 'Charts by TradingView',
    },
    bots: {
      eyebrow: 'The Lineup',
      headline: 'Ten strategies. One engine.',
      description:
        'Each bot targets a distinct, well-defined market edge — copy trading, arbitrage, market making, on-chain signals. All share the same battle-tested execution core, risk layer, and venue-agnostic adapter stack. Pick the edge that fits your thesis; the infrastructure is already built.',
      statusProduction: 'Production',
      statusDev: 'In dev',
      cardCta: 'Discuss this bot',
      cardCtaSuffix: '→ Telegram',
      items: {
        'copy-trading': {
          title: 'Copy Trading',
          tagline: 'Mirror top wallets automatically with configurable sizing and risk limits.',
          hook: 'Outsource alpha to wallets that already proved they have it.',
          description:
            'Zero research, zero chart-watching, zero second-guessing — the bot copies, you compound. Tracks one or more high-performing wallets and mirrors BUY/SELL actions in near-real time.',
          specs: [
            { label: 'Tracked wallets', value: 'Multiple' },
            { label: 'Order types', value: 'FAK / GTD' },
            { label: 'Risk', value: 'Circuit breaker + depth' },
            { label: 'Dry run', value: 'Fully supported' },
          ],
        },
        'btc-arb': {
          title: 'BTC 5m / 15m / 1hr Arbitrage',
          tagline: 'Speed on short-window BTC Up/Down markets.',
          hook: '42ms end-to-end — in the order book before you finish reading the title.',
          description:
            'Watches BTC Up/Down windows. When a pricing inefficiency emerges, the bot places a low-latency FAK before the window closes. Human reflexes don\'t compete here.',
          specs: [
            { label: 'Markets', value: 'BTC Up/Down 5m–1hr' },
            { label: 'Order type', value: 'FAK' },
            { label: 'Execution', value: '~42ms' },
          ],
        },
        'cross-arb': {
          title: 'Polymarket ↔ Kalshi Cross-Venue Arb',
          tagline: 'Cross-venue pricing inefficiencies on 15-min windows.',
          hook: 'Lock the spread, not the direction.',
          description:
            'Both legs hedged — your P&L is the price gap itself. Monitors the same market on both venues and executes hedged legs when a configurable price delta is detected.',
          specs: [
            { label: 'Venues', value: 'Polymarket ↔ Kalshi' },
            { label: 'Edge threshold', value: '≥ 0.8¢ configurable' },
            { label: 'Logging', value: 'Full P&L tracking' },
          ],
        },
        'direction-hunting': {
          title: 'Direction Hunting',
          tagline: 'Short-window momentum and flow setups.',
          hook: 'Define the edge once — let the engine grind 24/7.',
          description:
            'Entries, TP, and SL are fully automated. Continuously scans symbols and time windows for setups matching your criteria, then enters and manages exits.',
          specs: [
            { label: 'Windows', value: '5m, 15m configurable' },
            { label: 'Entry', value: 'Momentum / flow rules' },
            { label: 'Exit', value: 'TP + SL, auto' },
          ],
        },
        'spread-farming': {
          title: 'Spread Farming',
          tagline: 'Systematic, repeatable micro-edges.',
          hook: 'A thousand 0.5¢ wins compound into one big number.',
          description:
            'Disciplined, repeatable, boring in the best way — the kind of edge that survives every market regime. Sits at the spread, waits for fill conditions, executes with consistent sizing.',
          specs: [
            { label: 'Edge', value: 'Bid-ask spread' },
            { label: 'Logging', value: 'Per-trade + session' },
          ],
        },
        sports: {
          title: 'Sports Betting Execution',
          tagline: 'Click-to-bet speed on live sports markets.',
          hook: 'Click. Filled. Done — in under 50ms.',
          description:
            'Beat the line move that costs every other manual bettor their edge before they\'ve even confirmed the order. Real-time odds combined with fast FAK execution.',
          specs: [
            { label: 'Sports', value: 'NBA, NFL, Soccer +' },
            { label: 'Execution', value: '< 50ms' },
          ],
        },
        'resolution-sniper': {
          title: 'Resolution Sniper',
          tagline: 'High win-rate, low-variance plays unique to prediction markets.',
          hook: '95%+ probabilities at 95¢ → ride to the guaranteed $1.00 payout.',
          description:
            'The closest thing to free money any market has ever offered — and it doesn\'t exist outside prediction markets. Scans for near-certainty outcomes and holds to resolution.',
          specs: [
            { label: 'Certainty', value: 'Configurable' },
            { label: 'Payout', value: '$1.00 / share' },
            { label: 'Venues', value: 'PM · Kalshi · Limitless' },
          ],
        },
        'orderbook-imbalance': {
          title: 'Orderbook Imbalance',
          tagline: 'Pure order-flow signal, no external data required.',
          hook: 'No subscriptions, no external feeds, no broken APIs.',
          description:
            'The signal is the order book — self-contained, bulletproof, and impossible to front-run because nobody else sees what you see. Monitors live OBI at 500ms refresh.',
          specs: [
            { label: 'Signal', value: 'Live orderbook only' },
            { label: 'Refresh', value: '500ms' },
            { label: 'Venues', value: 'PM · Kalshi · Limitless' },
          ],
        },
        'market-making': {
          title: 'Market Making',
          tagline: 'Passive spread income on illiquid prediction markets.',
          hook: 'Be the house, not the gambler.',
          description:
            'Quote both sides, earn the spread on every fill — direction-agnostic income that scales with volume, not luck. GTD orders with inventory-aware skew.',
          specs: [
            { label: 'Order mgmt', value: 'Auto-cancel + requote' },
            { label: 'Inventory', value: 'Skew limits' },
            { label: 'Venues', value: 'Polymarket · Kalshi' },
          ],
        },
        'whale-signal': {
          title: 'On-Chain Whale Signal',
          tagline: 'Fastest possible signal — 3–30s ahead of the positions API.',
          hook: '3–30 seconds ahead of every other tracker on the planet.',
          description:
            'Direct from Polygon block data. You see the whale\'s order before the public positions API even acknowledges it exists. ABI calldata decoded the instant blocks land.',
          specs: [
            { label: 'Source', value: 'Polygon blocks' },
            { label: 'Lead time', value: '3–30s' },
            { label: 'Decode', value: 'ABI calldata' },
          ],
        },
      },
    },
    engine: {
      eyebrow: 'Under the hood',
      headlineLine1: 'Engineered in Rust.',
      headlineLine2: 'Tuned for prediction markets.',
      description:
        'Built on the guarantees Rust gives you — and the speed Tokio\'s async runtime makes possible. Every strategy shares the same execution path, the same risk hooks, the same observability surface.',
      features: [
        {
          icon: '🦀',
          title: 'Rust + Tokio',
          body: 'No GC pauses mid-trade. No data races in concurrent execution. No null-pointer surprises. Lean, predictable, fast under pressure.',
        },
        {
          icon: '🔌',
          title: 'Venue-agnostic adapters',
          body: 'One shared API surface across Polymarket, Kalshi, and Limitless. Strategies reference VenueId — not platform-specific types.',
        },
        {
          icon: '🖥️',
          title: 'Real-time TUI',
          body: 'ratatui-powered terminal interface with live log streaming, color-coded severity, and per-bot status views.',
        },
        {
          icon: '⚙️',
          title: 'FAK & GTD orders',
          body: 'Fill-or-Kill and Good-Till-Date order types with automatic market ID resolution. Concurrent execution, rate-limited.',
        },
      ],
      performanceEyebrow: 'Performance',
      performanceHeadline: 'Numbers that matter when milliseconds cost money.',
      metrics: [
        { metric: '< 1ms', label: 'Event processing' },
        { metric: '< 100ms', label: 'Order execution' },
        { metric: '~200ms', label: 'Position polling' },
        { metric: '~50MB', label: 'Memory baseline' },
        { metric: '< 5%', label: 'CPU utilization' },
        { metric: '25 / 10s', label: 'Rate limit (configurable)' },
      ],
    },
    safety: {
      eyebrow: 'Risk-first design',
      headlineLine1: 'Speed without guardrails',
      headlineLine2: 'is just expensive losing.',
      description:
        'Every order flows through a four-layer risk pipeline before it reaches the exchange. Circuit breakers, depth checks, size floors, and full dry-run — wired into the same execution core every bot uses.',
      layers: [
        {
          icon: '🛑',
          title: 'Circuit Breaker',
          body: 'Auto-halts after N consecutive large trades inside a configurable rolling window. Stops cascades before they start.',
          accent: 'text-rose-400',
        },
        {
          icon: '🛡️',
          title: 'Orderbook Depth Guard',
          body: 'Validates liquidity before every order. No fills into thin books — period.',
          accent: 'text-amber-400',
        },
        {
          icon: '🧪',
          title: 'Dry Run Mode',
          body: 'Full execution path runs without placing real orders. Validate signals and sizing with zero capital at risk.',
          accent: 'text-cyan-400',
        },
        {
          icon: '⚖️',
          title: 'Trade Size Floor',
          body: 'Minimum-size enforcement on every order. Filters out negative-EV micro-trades automatically.',
          accent: 'text-emerald-400',
        },
      ],
      ladderTitle: 'Deployment ladder',
      ladderSubtitle: 'A short checklist for going from zero to production.',
      ladder: [
        { stage: 'Setup', action: 'Run with enable_trading: false for one full session.' },
        { stage: 'First trades', action: 'Keep copy_percentage at 5–10% until you trust the signal.' },
        { stage: 'Ongoing', action: 'Watch circuit-breaker trips — they surface execution anomalies.' },
        { stage: 'Production', action: 'Use a dedicated wallet with only the capital you intend to deploy.' },
      ],
    },
    cta: {
      eyebrow: 'Get started',
      headline: 'Ready to put a bot to work?',
      description:
        'Telegram for direct conversation, scope discussion, or paid setup. GitHub for the open-source repo, configuration docs, and the live codebase. Response time on Telegram is typically a few hours.',
      ctaTelegram: '@HarrierOnChain on Telegram',
      ctaGithub: 'Browse the repo',
      pillars: [
        { title: 'Open source', body: 'MIT licensed. Read the code, fork it, build on it.' },
        { title: 'Actively maintained', body: 'The repo ships fixes and strategy upgrades regularly.' },
        { title: 'Direct line', body: 'No support tickets. You talk to the person who wrote the bot.' },
      ],
    },
    footer: {
      productName: 'Prediction Market Toolkits',
      tagline: 'Polymarket · Kalshi · Limitless',
      disclaimerLabel: 'Disclaimer.',
      disclaimer: ({ flag }) => (
        <>
          {' '}Trading prediction markets involves real financial risk. This software is provided as-is,
          without warranty or guarantee of any outcome. It is not financial advice. Always test with{' '}
          {flag}{' '}before deploying real capital. Ensure compliance with each venue&apos;s terms of service
          and applicable regulations in your jurisdiction.
        </>
      ),
      rights: 'MIT Licensed · Built for the prediction markets community.',
    },
  },

  zh: {
    langName: { en: 'English', zh: '简体中文' },
    nav: {
      strategies: '策略',
      engine: '引擎',
      safety: '安全',
      contact: '联系',
      github: 'GitHub',
      telegram: 'Telegram',
      langSwitch: '语言',
    },
    hero: {
      badge: '十款策略 · 同一套久经实战的引擎',
      headlineLine1: '预测市场，',
      headlineLine2: '以机器速度交易。',
      description: ({ polymarket, kalshi, limitless }) => (
        <>
          面向 {polymarket}、{kalshi}、{limitless} 三家预测市场打造的十款生产级 Rust 交易机器人。
          跟单交易、跨平台套利、鲸鱼信号、做市商——共享同一套风控层与执行核心。
        </>
      ),
      ctaTelegram: '通过 Telegram 联系',
      ctaGithub: '查看 GitHub',
      stats: [
        { label: '下单执行', value: '< 100ms', unit: '端到端' },
        { label: '事件处理', value: '< 1ms', unit: '每事件' },
        { label: '内存占用', value: '~50MB', unit: '常驻' },
        { label: 'CPU 占用', value: '< 5%', unit: '现代硬件' },
      ],
    },
    liveSignal: {
      eyebrow: '实时信号',
      headline: 'BTC/USDT — 机器人正在盯的市场',
      sub: '实时现货价格，直接来自订单簿。',
      statusLive: '实时',
      statusOffline: '离线',
      statusConnecting: '连接中',
      pair: 'BTC / USDT',
      sessionLabel: '· 本次会话',
      footnote: ({ source, chainlink }) => (
        <>
          数据源：{source}。Polymarket 的 BTC 涨跌市场以 {chainlink} 结算，
          而 Chainlink 聚合自 Binance、Coinbase、Kraken 等主要交易场所——所以你看到的
          就是机器人交易对手价格的忠实实时近似。
        </>
      ),
      attribution: '图表来自 TradingView',
    },
    bots: {
      eyebrow: '策略阵容',
      headline: '十款策略。一套引擎。',
      description:
        '每款机器人都瞄准一个清晰、独立的市场优势——跟单、套利、做市、链上信号。所有策略共享同一套久经实战的执行核心、风控层与平台无关的适配层。挑一个匹配你判断的优势上场；底层基础设施已经为你搭好了。',
      statusProduction: '生产可用',
      statusDev: '开发中',
      cardCta: '咨询这款机器人',
      cardCtaSuffix: '→ Telegram',
      items: {
        'copy-trading': {
          title: '跟单交易',
          tagline: '自动镜像顶级钱包的交易，仓位规模与风险限制可配置。',
          hook: '把研究外包给已经被证明拥有 alpha 的钱包。',
          description:
            '无需自研、无需盯盘、无需反复纠结——机器人复制，你做复利。近实时跟踪一个或多个高表现钱包，自动镜像其 BUY/SELL 动作。',
          specs: [
            { label: '追踪钱包', value: '多钱包并发' },
            { label: '订单类型', value: 'FAK / GTD' },
            { label: '风控', value: '熔断器 + 深度护卫' },
            { label: '空跑', value: '完整支持' },
          ],
        },
        'btc-arb': {
          title: 'BTC 5 分 / 15 分 / 1 小时套利',
          tagline: '短窗口 BTC 涨跌市场上的速度型策略。',
          hook: '端到端 42ms——你还没读完标题，机器人已经在订单簿里。',
          description:
            '监控 BTC 涨跌窗口。一旦出现定价偏差，机器人会在窗口关闭前以低延迟下达 FAK 订单。人类反应跟不上。',
          specs: [
            { label: '市场', value: 'BTC 涨跌 5m–1hr' },
            { label: '订单类型', value: 'FAK' },
            { label: '执行', value: '~42ms' },
          ],
        },
        'cross-arb': {
          title: 'Polymarket ↔ Kalshi 跨平台套利',
          tagline: '15 分钟窗口下的跨平台定价偏差。',
          hook: '锁定的是价差，不是方向。',
          description:
            '双腿对冲——你的盈亏来自价差本身。同时监控两个平台上的同一市场，价差达到阈值时执行对冲腿。',
          specs: [
            { label: '平台', value: 'Polymarket ↔ Kalshi' },
            { label: '价差阈值', value: '≥ 0.8¢ 可配置' },
            { label: '日志', value: '完整盈亏追踪' },
          ],
        },
        'direction-hunting': {
          title: '方向猎取',
          tagline: '短窗口动量与单边流向机会。',
          hook: '规则定义一次，引擎全天候执行。',
          description:
            '入场、止盈、止损全自动。持续扫描多个标的与时间窗口，匹配你设定的入场条件后入场并管理出场。',
          specs: [
            { label: '窗口', value: '5m、15m 可配置' },
            { label: '入场', value: '动量 / 流向规则' },
            { label: '出场', value: 'TP + SL 自动' },
          ],
        },
        'spread-farming': {
          title: '价差耕作',
          tagline: '系统化、可重复的微观优势。',
          hook: '一千次 0.5¢ 小胜，复利成一个大数字。',
          description:
            '有纪律、可重复、平淡得恰到好处——穿越所有市场环境的真正优势。挂在价差上，等成交条件齐备，按一致仓位规模执行。',
          specs: [
            { label: '优势', value: '买卖价差' },
            { label: '日志', value: '单笔 + 会话汇总' },
          ],
        },
        sports: {
          title: '体育博彩执行',
          tagline: '实时体育市场上的"点击即下单"速度。',
          hook: '点击。成交。完成——不到 50ms。',
          description:
            '在其他手动玩家还在确认订单时，你已经吃到了让他们破功的盘口移动。实时赔率结合快速 FAK 执行。',
          specs: [
            { label: '运动', value: 'NBA、NFL、足球+' },
            { label: '执行', value: '< 50ms' },
          ],
        },
        'resolution-sniper': {
          title: '结算狙击',
          tagline: '预测市场独有的高胜率、低方差玩法。',
          hook: '95%+ 概率买在 95¢ → 持有到 $1.00 派息。',
          description:
            '任何市场提供过的最接近"白送钱"的玩法——只在预测市场存在。扫描接近确定性的结果，持有至结算。',
          specs: [
            { label: '确定性', value: '可配置' },
            { label: '派息', value: '$1.00 / 股' },
            { label: '平台', value: 'PM · Kalshi · Limitless' },
          ],
        },
        'orderbook-imbalance': {
          title: '订单簿失衡',
          tagline: '纯订单流信号，无需任何外部数据。',
          hook: '不订阅、不接外部源、不依赖会坏的 API。',
          description:
            '信号本身就是订单簿——自给自足、坚不可摧、且无法被抢跑，因为没人能看到和你一样的视角。500ms 刷新的实时 OBI。',
          specs: [
            { label: '信号', value: '仅实时订单簿' },
            { label: '刷新', value: '500ms' },
            { label: '平台', value: 'PM · Kalshi · Limitless' },
          ],
        },
        'market-making': {
          title: '做市商',
          tagline: '在流动性差的预测市场被动赚取价差收入。',
          hook: '当庄家，不当赌客。',
          description:
            '双边挂单，每笔成交都吃价差——与方向无关的收入，随成交量而非运气增长。GTD 订单 + 库存敏感的报价倾斜。',
          specs: [
            { label: '订单管理', value: '成交自动撤 + 重报' },
            { label: '库存', value: '倾斜上限' },
            { label: '平台', value: 'Polymarket · Kalshi' },
          ],
        },
        'whale-signal': {
          title: '链上鲸鱼信号',
          tagline: '最快可能的信号——比仓位 API 快 3–30 秒。',
          hook: '比地球上其他所有跟踪器都快 3–30 秒。',
          description:
            '直接从 Polygon 区块数据获取。你能在公开仓位 API 还没意识到这笔交易存在之前，就看到鲸鱼的下单。区块落地瞬间解码 ABI calldata。',
          specs: [
            { label: '信号源', value: 'Polygon 区块' },
            { label: '领先时间', value: '3–30 秒' },
            { label: '解码', value: 'ABI calldata' },
          ],
        },
      },
    },
    engine: {
      eyebrow: '引擎之下',
      headlineLine1: '用 Rust 构建。',
      headlineLine2: '为预测市场调校。',
      description:
        '建立在 Rust 的保证之上——以及 Tokio 异步运行时所能提供的速度。每个策略共享同一条执行路径、同一套风控钩子、同一个可观测面。',
      features: [
        {
          icon: '🦀',
          title: 'Rust + Tokio',
          body: '交易中没有 GC 停顿。并发执行无数据竞争。没有空指针意外。精简、可预期，压力下稳定。',
        },
        {
          icon: '🔌',
          title: '平台无关适配层',
          body: '一套共享的 API 表面覆盖 Polymarket、Kalshi、Limitless。策略引用 VenueId，不引用平台特有类型。',
        },
        {
          icon: '🖥️',
          title: '实时 TUI',
          body: '基于 ratatui 的终端界面，实时日志流、按级别着色、按机器人状态分屏。',
        },
        {
          icon: '⚙️',
          title: 'FAK 与 GTD 订单',
          body: '即时成交或取消、有效期订单，自动解析市场 ID。并发执行、限速。',
        },
      ],
      performanceEyebrow: '性能',
      performanceHeadline: '当毫秒变成钱时，这些数字就是关键。',
      metrics: [
        { metric: '< 1ms', label: '事件处理' },
        { metric: '< 100ms', label: '下单执行' },
        { metric: '~200ms', label: '仓位轮询' },
        { metric: '~50MB', label: '内存基线' },
        { metric: '< 5%', label: 'CPU 占用' },
        { metric: '25 / 10s', label: '限速（可配置）' },
      ],
    },
    safety: {
      eyebrow: '风控优先',
      headlineLine1: '没有护栏的速度',
      headlineLine2: '只是更贵的亏损。',
      description:
        '每一笔订单在送达交易所之前都要经过四层风控管道。熔断器、深度检查、最小下单额、完整空跑——全部接入每个机器人共享的执行核心。',
      layers: [
        {
          icon: '🛑',
          title: '熔断器',
          body: '在配置窗口内出现 N 笔连续大额成交后自动暂停。从源头阻断连锁反应。',
          accent: 'text-rose-400',
        },
        {
          icon: '🛡️',
          title: '订单簿深度护卫',
          body: '每一笔订单前校验流动性。绝不在薄盘里成交。',
          accent: 'text-amber-400',
        },
        {
          icon: '🧪',
          title: '空跑模式',
          body: '完整执行链路运行但不真正下单。零资金风险下验证信号与仓位。',
          accent: 'text-cyan-400',
        },
        {
          icon: '⚖️',
          title: '最小下单额',
          body: '每一笔订单都强制最小规模。自动过滤负 EV 的微交易。',
          accent: 'text-emerald-400',
        },
      ],
      ladderTitle: '部署阶梯',
      ladderSubtitle: '从零到生产的简短清单。',
      ladder: [
        { stage: '初始部署', action: '用 enable_trading: false 跑完一整轮。' },
        { stage: '首次实盘', action: '信任信号前，copy_percentage 保持在 5–10%。' },
        { stage: '长期运行', action: '关注熔断触发事件——它们会暴露执行异常。' },
        { stage: '生产环境', action: '使用专用钱包，只放你打算部署的资金。' },
      ],
    },
    cta: {
      eyebrow: '开始使用',
      headline: '准备好让机器人开始工作了吗？',
      description:
        'Telegram 用于直接对话、需求讨论、或付费部署。GitHub 用于查看开源代码、配置文档、最新提交。Telegram 通常几小时内回复。',
      ctaTelegram: 'Telegram：@HarrierOnChain',
      ctaGithub: '查看代码仓库',
      pillars: [
        { title: '开源', body: 'MIT 许可。读它、Fork 它、在它之上构建。' },
        { title: '持续维护', body: '仓库会定期提交修复与策略升级。' },
        { title: '直接对话', body: '没有工单系统。你直接和写这段代码的人对话。' },
      ],
    },
    footer: {
      productName: '预测市场工具包',
      tagline: 'Polymarket · Kalshi · Limitless',
      disclaimerLabel: '免责声明。',
      disclaimer: ({ flag }) => (
        <>
          {' '}预测市场交易涉及真实的财务风险。本软件按"原样"提供，不附带任何形式的担保或对结果的保证，且不构成投资建议。
          投入真实资金前请务必先以 {flag} 进行测试。请确保遵守各平台的服务条款以及你所在司法管辖区的相关法规。
        </>
      ),
      rights: 'MIT 许可 · 为预测市场社区而构建。',
    },
  },
};

export function useT(): Messages {
  const { lang } = useLang();
  return messages[lang];
}

export { messages };
