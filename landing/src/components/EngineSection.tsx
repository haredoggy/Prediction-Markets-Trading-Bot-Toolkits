import { useT } from '../messages';

export function EngineSection() {
  const t = useT();
  return (
    <section id="engine" className="py-24 border-t border-border-subtle">
      <div className="container-x">
        <div className="max-w-3xl mb-16">
          <div className="text-sm font-semibold text-cyan-400 uppercase tracking-wider mb-3">{t.engine.eyebrow}</div>
          <h2 className="text-4xl md:text-5xl font-bold tracking-tight mb-5">
            {t.engine.headlineLine1}<br />
            <span className="text-zinc-500">{t.engine.headlineLine2}</span>
          </h2>
          <p className="text-lg text-zinc-400 leading-relaxed">
            {t.engine.description}
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-5 mb-16">
          {t.engine.features.map((feature) => (
            <div key={feature.title} className="card p-6">
              <div className="text-3xl mb-4">{feature.icon}</div>
              <h3 className="text-lg font-bold text-white mb-2">{feature.title}</h3>
              <p className="text-sm text-zinc-400 leading-relaxed">{feature.body}</p>
            </div>
          ))}
        </div>

        <div className="card p-8 md:p-10 bg-gradient-to-br from-bg-surface to-bg-elevated">
          <div className="text-xs font-semibold text-cyan-400 uppercase tracking-wider mb-2">{t.engine.performanceEyebrow}</div>
          <h3 className="text-2xl font-bold mb-8">{t.engine.performanceHeadline}</h3>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-6">
            {t.engine.metrics.map((p) => (
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
