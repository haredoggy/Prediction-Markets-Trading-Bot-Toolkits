import type { Bot } from '../bots';
import { accentClasses, TELEGRAM_URL } from '../bots';

export function BotCard({ bot, index }: { bot: Bot; index: number }) {
  const accent = accentClasses[bot.accent];

  return (
    <div className="card p-6 group flex flex-col h-full">
      <div className="flex items-start justify-between gap-3 mb-4">
        <div className="flex items-center gap-3">
          <div
            className={`w-11 h-11 rounded-lg ${accent.bg} ring-1 ${accent.ring} flex items-center justify-center text-xl shadow-lg ${accent.glow}`}
            aria-hidden
          >
            {bot.emoji}
          </div>
          <span className="text-zinc-600 font-mono text-xs">#{String(index + 1).padStart(2, '0')}</span>
        </div>
        <StatusPill status={bot.status} />
      </div>

      <h3 className="text-lg font-bold text-white leading-tight mb-1.5">{bot.title}</h3>
      <p className="text-sm text-zinc-400 italic mb-3">{bot.tagline}</p>

      <div className={`${accent.bg} border-l-2 ${accent.border} pl-3 py-2 mb-4`}>
        <p className={`text-sm font-semibold ${accent.text} leading-snug`}>{bot.hook}</p>
      </div>

      <p className="text-sm text-zinc-400 leading-relaxed mb-5 flex-1">{bot.description}</p>

      <div className="border-t border-border-subtle pt-3 mb-4">
        {bot.specs.map((spec) => (
          <div key={spec.label} className="spec-row">
            <span className="spec-label">{spec.label}</span>
            <span className="spec-value">{spec.value}</span>
          </div>
        ))}
      </div>

      <a
        href={TELEGRAM_URL}
        target="_blank"
        rel="noreferrer"
        className="inline-flex items-center justify-between gap-2 px-4 py-2.5 rounded-lg bg-bg-elevated border border-border-subtle hover:border-border-strong hover:bg-zinc-800/60 text-sm font-medium text-zinc-200 transition-all"
      >
        <span>Discuss this bot</span>
        <span className="font-mono text-xs text-zinc-500">→ Telegram</span>
      </a>
    </div>
  );
}

function StatusPill({ status }: { status: Bot['status'] }) {
  if (status === 'production') {
    return (
      <span className="pill-production">
        <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
        Production
      </span>
    );
  }
  return (
    <span className="pill-dev">
      <span className="w-1.5 h-1.5 rounded-full bg-amber-400" />
      In dev
    </span>
  );
}
