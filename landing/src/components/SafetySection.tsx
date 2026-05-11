import { useT } from '../messages';

export function SafetySection() {
  const t = useT();
  return (
    <section id="safety" className="py-24 border-t border-border-subtle">
      <div className="container-x">
        <div className="max-w-3xl mb-16">
          <div className="text-sm font-semibold text-rose-400 uppercase tracking-wider mb-3">{t.safety.eyebrow}</div>
          <h2 className="text-4xl md:text-5xl font-bold tracking-tight mb-5">
            {t.safety.headlineLine1}<br />
            <span className="text-zinc-500">{t.safety.headlineLine2}</span>
          </h2>
          <p className="text-lg text-zinc-400 leading-relaxed">
            {t.safety.description}
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-5 mb-12">
          {t.safety.layers.map((layer) => (
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
          <h3 className="text-xl font-bold mb-1">{t.safety.ladderTitle}</h3>
          <p className="text-sm text-zinc-500 mb-6">{t.safety.ladderSubtitle}</p>
          <div className="space-y-3">
            {t.safety.ladder.map((r, i) => (
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
