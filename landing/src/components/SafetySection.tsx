const safetyLayers = [
  {
    title: 'Circuit Breaker',
    body: 'Auto-halts after N consecutive large trades inside a configurable rolling window. Stops cascades before they start.',
    icon: '🛑',
    accent: 'text-rose-400',
  },
  {
    title: 'Orderbook Depth Guard',
    body: 'Validates liquidity before every order. No fills into thin books — period.',
    icon: '🛡️',
    accent: 'text-amber-400',
  },
  {
    title: 'Dry Run Mode',
    body: 'Full execution path runs without placing real orders. Validate signals and sizing with zero capital at risk.',
    icon: '🧪',
    accent: 'text-cyan-400',
  },
  {
    title: 'Trade Size Floor',
    body: 'Minimum-size enforcement on every order. Filters out negative-EV micro-trades automatically.',
    icon: '⚖️',
    accent: 'text-emerald-400',
  },
];

const recommendations = [
  { stage: 'Setup', action: 'Run with enable_trading: false for one full session.' },
  { stage: 'First trades', action: 'Keep copy_percentage at 5–10% until you trust the signal.' },
  { stage: 'Ongoing', action: 'Watch circuit-breaker trips — they surface execution anomalies.' },
  { stage: 'Production', action: 'Use a dedicated wallet with only the capital you intend to deploy.' },
];

export function SafetySection() {
  return (
    <section id="safety" className="py-24 border-t border-border-subtle">
      <div className="container-x">
        <div className="max-w-3xl mb-16">
          <div className="text-sm font-semibold text-rose-400 uppercase tracking-wider mb-3">Risk-first design</div>
          <h2 className="text-4xl md:text-5xl font-bold tracking-tight mb-5">
            Speed without guardrails<br />
            <span className="text-zinc-500">is just expensive losing.</span>
          </h2>
          <p className="text-lg text-zinc-400 leading-relaxed">
            Every order flows through a four-layer risk pipeline before it reaches the exchange.
            Circuit breakers, depth checks, size floors, and full dry-run — wired into the same execution core every bot uses.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-5 mb-12">
          {safetyLayers.map((layer) => (
            <div key={layer.title} className="card p-6">
              <div className="flex items-center gap-3 mb-3">
                <span className="text-2xl">{layer.icon}</span>
                <h3 className={`text-lg font-bold ${layer.accent}`}>{layer.title}</h3>
              </div>
              <p className="text-sm text-zinc-400 leading-relaxed">{layer.body}</p>
            </div>
          ))}
        </div>

        <div className="card p-8">
          <h3 className="text-xl font-bold mb-1">Deployment ladder</h3>
          <p className="text-sm text-zinc-500 mb-6">A short checklist for going from zero to production.</p>
          <div className="space-y-3">
            {recommendations.map((r, i) => (
              <div key={r.stage} className="flex items-start gap-4 py-3 border-b border-border-subtle last:border-b-0">
                <div className="flex-shrink-0 w-8 h-8 rounded-full bg-purple-500/10 ring-1 ring-purple-500/30 flex items-center justify-center text-xs font-mono font-bold text-purple-300">
                  {i + 1}
                </div>
                <div>
                  <div className="text-sm font-semibold text-white">{r.stage}</div>
                  <div className="text-sm text-zinc-400 mt-0.5">{r.action}</div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
