export type BotStatus = 'production' | 'development';

export interface Spec {
  label: string;
  value: string;
}

export interface Bot {
  id: string;
  emoji: string;
  title: string;
  tagline: string;
  hook: string;
  description: string;
  specs: Spec[];
  status: BotStatus;
  accent: 'purple' | 'pink' | 'cyan' | 'amber' | 'emerald' | 'rose' | 'sky' | 'indigo' | 'fuchsia' | 'orange';
}

export const TELEGRAM_URL = 'https://t.me/haredoggy';
export const GITHUB_URL = 'https://github.com/haredoggy/Prediction-Markets-Trading-Bot-Toolkits';

export const bots: Bot[] = [
  {
    id: 'copy-trading',
    emoji: '🎯',
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
    status: 'production',
    accent: 'purple',
  },
  {
    id: 'btc-arb',
    emoji: '⚡',
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
    status: 'production',
    accent: 'amber',
  },
  {
    id: 'cross-arb',
    emoji: '💰',
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
    status: 'production',
    accent: 'emerald',
  },
  {
    id: 'direction-hunting',
    emoji: '🎯',
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
    status: 'production',
    accent: 'cyan',
  },
  {
    id: 'spread-farming',
    emoji: '📈',
    title: 'Spread Farming',
    tagline: 'Systematic, repeatable micro-edges.',
    hook: 'A thousand 0.5¢ wins compound into one big number.',
    description:
      'Disciplined, repeatable, boring in the best way — the kind of edge that survives every market regime. Sits at the spread, waits for fill conditions, executes with consistent sizing.',
    specs: [
      { label: 'Edge', value: 'Bid-ask spread' },
      { label: 'Logging', value: 'Per-trade + session' },
    ],
    status: 'production',
    accent: 'pink',
  },
  {
    id: 'sports',
    emoji: '🏆',
    title: 'Sports Betting Execution',
    tagline: 'Click-to-bet speed on live sports markets.',
    hook: 'Click. Filled. Done — in under 50ms.',
    description:
      'Beat the line move that costs every other manual bettor their edge before they\'ve even confirmed the order. Real-time odds combined with fast FAK execution.',
    specs: [
      { label: 'Sports', value: 'NBA, NFL, Soccer +' },
      { label: 'Execution', value: '< 50ms' },
    ],
    status: 'production',
    accent: 'orange',
  },
  {
    id: 'resolution-sniper',
    emoji: '🎯',
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
    status: 'production',
    accent: 'rose',
  },
  {
    id: 'orderbook-imbalance',
    emoji: '📊',
    title: 'Orderbook Imbalance',
    tagline: 'Pure order-flow signal, no external data required.',
    hook: 'No subscriptions, no external feeds, no broken APIs.',
    description:
      'The signal *is* the order book — self-contained, bulletproof, and impossible to front-run because nobody else sees what you see. Monitors live OBI at 500ms refresh.',
    specs: [
      { label: 'Signal', value: 'Live orderbook only' },
      { label: 'Refresh', value: '500ms' },
      { label: 'Venues', value: 'PM · Kalshi · Limitless' },
    ],
    status: 'production',
    accent: 'sky',
  },
  {
    id: 'market-making',
    emoji: '💰',
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
    status: 'production',
    accent: 'indigo',
  },
  {
    id: 'whale-signal',
    emoji: '⚡',
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
    status: 'production',
    accent: 'fuchsia',
  },
];

export interface AccentSet {
  ring: string;
  text: string;
  bg: string;
  glow: string;
  border: string;
}

export const accentClasses: Record<Bot['accent'], AccentSet> = {
  purple: {
    ring: 'ring-purple-500/30',
    text: 'text-purple-400',
    bg: 'bg-purple-500/10',
    glow: 'shadow-purple-500/20',
    border: 'border-purple-500/30',
  },
  pink: {
    ring: 'ring-pink-500/30',
    text: 'text-pink-400',
    bg: 'bg-pink-500/10',
    glow: 'shadow-pink-500/20',
    border: 'border-pink-500/30',
  },
  cyan: {
    ring: 'ring-cyan-500/30',
    text: 'text-cyan-400',
    bg: 'bg-cyan-500/10',
    glow: 'shadow-cyan-500/20',
    border: 'border-cyan-500/30',
  },
  amber: {
    ring: 'ring-amber-500/30',
    text: 'text-amber-400',
    bg: 'bg-amber-500/10',
    glow: 'shadow-amber-500/20',
    border: 'border-amber-500/30',
  },
  emerald: {
    ring: 'ring-emerald-500/30',
    text: 'text-emerald-400',
    bg: 'bg-emerald-500/10',
    glow: 'shadow-emerald-500/20',
    border: 'border-emerald-500/30',
  },
  rose: {
    ring: 'ring-rose-500/30',
    text: 'text-rose-400',
    bg: 'bg-rose-500/10',
    glow: 'shadow-rose-500/20',
    border: 'border-rose-500/30',
  },
  sky: {
    ring: 'ring-sky-500/30',
    text: 'text-sky-400',
    bg: 'bg-sky-500/10',
    glow: 'shadow-sky-500/20',
    border: 'border-sky-500/30',
  },
  indigo: {
    ring: 'ring-indigo-500/30',
    text: 'text-indigo-400',
    bg: 'bg-indigo-500/10',
    glow: 'shadow-indigo-500/20',
    border: 'border-indigo-500/30',
  },
  fuchsia: {
    ring: 'ring-fuchsia-500/30',
    text: 'text-fuchsia-400',
    bg: 'bg-fuchsia-500/10',
    glow: 'shadow-fuchsia-500/20',
    border: 'border-fuchsia-500/30',
  },
  orange: {
    ring: 'ring-orange-500/30',
    text: 'text-orange-400',
    bg: 'bg-orange-500/10',
    glow: 'shadow-orange-500/20',
    border: 'border-orange-500/30',
  },
};
