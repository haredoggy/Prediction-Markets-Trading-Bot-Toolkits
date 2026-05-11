import { TELEGRAM_URL, GITHUB_URL } from '../bots';
import { useT } from '../messages';

export function Hero() {
  const t = useT();
  return (
    <section id="top" className="relative pt-20 pb-24 overflow-hidden">
      <div className="container-x relative">
        <div className="flex flex-col items-center text-center">
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-purple-500/10 border border-purple-500/20 text-xs font-medium text-purple-300 mb-6">
            <span className="w-1.5 h-1.5 rounded-full bg-purple-400 animate-pulse" />
            {t.hero.badge}
          </div>

          <h1 className="text-5xl md:text-7xl font-extrabold tracking-tight leading-[1.05] max-w-5xl">
            {t.hero.headlineLine1}
            <br />
            <span className="gradient-text">{t.hero.headlineLine2}</span>
          </h1>

          <p className="mt-6 text-lg md:text-xl text-zinc-400 max-w-2xl leading-relaxed">
            {t.hero.description({
              polymarket: <span className="text-brand-polymarket font-semibold">Polymarket</span>,
              kalshi: <span className="text-brand-kalshi font-semibold">Kalshi</span>,
              limitless: <span className="text-brand-limitless font-semibold">Limitless</span>,
            })}
          </p>

          <div className="mt-10 flex flex-wrap items-center justify-center gap-4">
            <a href={TELEGRAM_URL} target="_blank" rel="noreferrer" className="btn-primary text-base px-6 py-3">
              <TelegramIcon />
              <span>{t.hero.ctaTelegram}</span>
              <ArrowIcon />
            </a>
            <a href={GITHUB_URL} target="_blank" rel="noreferrer" className="btn-secondary text-base px-6 py-3">
              <GitHubIcon />
              <span>{t.hero.ctaGithub}</span>
            </a>
          </div>

          <div className="mt-16 grid grid-cols-2 md:grid-cols-4 gap-px bg-border-subtle rounded-2xl overflow-hidden max-w-4xl w-full">
            {t.hero.stats.map((stat) => (
              <Stat key={stat.label} label={stat.label} value={stat.value} unit={stat.unit} />
            ))}
          </div>
        </div>
      </div>

      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-purple-500/10 rounded-full blur-[120px] -z-10 pointer-events-none" />
    </section>
  );
}

function Stat({ label, value, unit }: { label: string; value: string; unit: string }) {
  return (
    <div className="bg-bg-surface p-5 text-left">
      <div className="text-xs text-zinc-500 uppercase tracking-wider font-medium">{label}</div>
      <div className="mt-2 flex items-baseline gap-1.5">
        <span className="text-2xl md:text-3xl font-bold font-mono text-white">{value}</span>
        <span className="text-xs text-zinc-500">{unit}</span>
      </div>
    </div>
  );
}

function ArrowIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <path d="M5 12h14M13 6l6 6-6 6"/>
    </svg>
  );
}

function GitHubIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden>
      <path d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.55v-1.93c-3.2.7-3.88-1.54-3.88-1.54-.52-1.34-1.28-1.7-1.28-1.7-1.05-.72.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.55-.29-5.24-1.28-5.24-5.7 0-1.26.45-2.29 1.19-3.1-.12-.29-.52-1.47.11-3.06 0 0 .97-.31 3.18 1.18a11.1 11.1 0 015.79 0c2.2-1.49 3.18-1.18 3.18-1.18.63 1.59.23 2.77.11 3.06.74.81 1.19 1.84 1.19 3.1 0 4.43-2.7 5.41-5.27 5.69.41.36.78 1.07.78 2.15v3.19c0 .31.21.67.8.55 4.56-1.53 7.85-5.84 7.85-10.91C23.5 5.65 18.35.5 12 .5z"/>
    </svg>
  );
}

function TelegramIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden>
      <path d="M9.78 18.65l.28-4.23 7.68-6.92c.34-.31-.07-.46-.52-.19L7.74 13.3 3.64 12c-.88-.25-.89-.86.2-1.3l15.97-6.16c.73-.33 1.43.18 1.15 1.3l-2.72 12.81c-.19.91-.74 1.13-1.5.71L12.6 16.3l-1.99 1.93c-.23.23-.42.42-.83.42z"/>
    </svg>
  );
}
