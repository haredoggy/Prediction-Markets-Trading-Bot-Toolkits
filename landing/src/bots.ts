export type BotStatus = 'production' | 'development';

export type BotAccent =
  | 'purple'
  | 'pink'
  | 'cyan'
  | 'amber'
  | 'emerald'
  | 'rose'
  | 'sky'
  | 'indigo'
  | 'fuchsia'
  | 'orange';

export interface BotMeta {
  id: string;
  emoji: string;
  status: BotStatus;
  accent: BotAccent;
}

export const TELEGRAM_URL = 'https://t.me/HarrierOnChain';
export const GITHUB_URL = 'https://github.com/haredoggy/Prediction-Markets-Trading-Bot-Toolkits';

export const bots: BotMeta[] = [
  { id: 'copy-trading', emoji: '🎯', status: 'production', accent: 'purple' },
  { id: 'btc-arb', emoji: '⚡', status: 'production', accent: 'amber' },
  { id: 'cross-arb', emoji: '💰', status: 'production', accent: 'emerald' },
  { id: 'direction-hunting', emoji: '🎯', status: 'production', accent: 'cyan' },
  { id: 'spread-farming', emoji: '📈', status: 'production', accent: 'pink' },
  { id: 'sports', emoji: '🏆', status: 'production', accent: 'orange' },
  { id: 'resolution-sniper', emoji: '🎯', status: 'production', accent: 'rose' },
  { id: 'orderbook-imbalance', emoji: '📊', status: 'production', accent: 'sky' },
  { id: 'market-making', emoji: '💰', status: 'production', accent: 'indigo' },
  { id: 'whale-signal', emoji: '⚡', status: 'production', accent: 'fuchsia' },
];

export interface AccentSet {
  ring: string;
  text: string;
  bg: string;
  glow: string;
  border: string;
}

export const accentClasses: Record<BotAccent, AccentSet> = {
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
