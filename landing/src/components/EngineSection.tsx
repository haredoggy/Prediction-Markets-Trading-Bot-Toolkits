const coreFeatures = [
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
];

const performanceMetrics = [
  { metric: '< 1ms', label: 'Event processing' },
  { metric: '< 100ms', label: 'Order execution' },
  { metric: '~200ms', label: 'Position polling' },
  { metric: '~50MB', label: 'Memory baseline' },
  { metric: '< 5%', label: 'CPU utilization' },
  { metric: '25 / 10s', label: 'Rate limit (configurable)' },
];

export function EngineSection() {
  return (
    <section id="engine" className="py-24 border-t border-border-subtle">
      <div className="container-x">
        <div className="max-w-3xl mb-16">
          <div className="text-sm font-semibold text-cyan-400 uppercase tracking-wider mb-3">Under the hood</div>
          <h2 className="text-4xl md:text-5xl font-bold tracking-tight mb-5">
            Engineered in Rust.<br />
            <span className="text-zinc-500">Tuned for prediction markets.</span>
          </h2>
          <p className="text-lg text-zinc-400 leading-relaxed">
            Built on the guarantees Rust gives you — and the speed Tokio's async runtime makes possible.
            Every strategy shares the same execution path, the same risk hooks, the same observability surface.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-5 mb-16">
          {coreFeatures.map((feature) => (
            <div key={feature.title} className="card p-6">
              <div className="text-3xl mb-4">{feature.icon}</div>
              <h3 className="text-lg font-bold text-white mb-2">{feature.title}</h3>
              <p className="text-sm text-zinc-400 leading-relaxed">{feature.body}</p>
            </div>
          ))}
        </div>

        <div className="card p-8 md:p-10 bg-gradient-to-br from-bg-surface to-bg-elevated">
          <div className="text-xs font-semibold text-cyan-400 uppercase tracking-wider mb-2">Performance</div>
          <h3 className="text-2xl font-bold mb-8">Numbers that matter when milliseconds cost money.</h3>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-6">
            {performanceMetrics.map((p) => (
              <div key={p.label}>
                <div className="text-3xl md:text-4xl font-bold font-mono text-white">{p.metric}</div>
                <div className="text-sm text-zinc-500 mt-1">{p.label}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
