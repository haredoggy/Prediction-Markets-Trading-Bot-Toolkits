import { TELEGRAM_URL, GITHUB_URL } from '../bots';
import { useT } from '../messages';

export function CtaSection() {
  const t = useT();
  return (
    <section id="contact" className="py-24 border-t border-border-subtle">
      <div className="container-x">
        <div className="relative card p-10 md:p-16 overflow-hidden bg-gradient-to-br from-purple-500/5 via-bg-surface to-pink-500/5">
          <div className="relative max-w-3xl">
            <div className="text-sm font-semibold text-pink-400 uppercase tracking-wider mb-3">{t.cta.eyebrow}</div>
            <h2 className="text-4xl md:text-5xl font-bold tracking-tight mb-5">
              {t.cta.headline}
            </h2>
            <p className="text-lg text-zinc-400 leading-relaxed mb-8">
              {t.cta.description}
            </p>
            <div className="flex flex-wrap items-center gap-4">
              <a href={TELEGRAM_URL} target="_blank" rel="noreferrer" className="btn-primary text-base px-7 py-3.5">
                <TelegramIcon />
                <span>{t.cta.ctaTelegram}</span>
              </a>
              <a href={GITHUB_URL} target="_blank" rel="noreferrer" className="btn-secondary text-base px-7 py-3.5">
                <GitHubIcon />
                <span>{t.cta.ctaGithub}</span>
              </a>
            </div>

            <div className="mt-10 pt-8 border-t border-border-subtle grid grid-cols-1 sm:grid-cols-3 gap-6 text-sm">
              {t.cta.pillars.map((pillar) => (
                <Pillar key={pillar.title} title={pillar.title} body={pillar.body} />
              ))}
            </div>
          </div>

          <div className="absolute -top-20 -right-20 w-96 h-96 bg-purple-500/20 rounded-full blur-[100px] pointer-events-none" />
          <div className="absolute -bottom-20 -left-20 w-96 h-96 bg-pink-500/20 rounded-full blur-[100px] pointer-events-none" />
        </div>
      </div>
    </section>
  );
}

function Pillar({ title, body }: { title: string; body: string }) {
  return (
    <div>
      <div className="font-semibold text-white mb-1">{title}</div>
      <div className="text-zinc-400 text-sm leading-relaxed">{body}</div>
    </div>
  );
}

function GitHubIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden>
      <path d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.55v-1.93c-3.2.7-3.88-1.54-3.88-1.54-.52-1.34-1.28-1.7-1.28-1.7-1.05-.72.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.55-.29-5.24-1.28-5.24-5.7 0-1.26.45-2.29 1.19-3.1-.12-.29-.52-1.47.11-3.06 0 0 .97-.31 3.18 1.18a11.1 11.1 0 015.79 0c2.2-1.49 3.18-1.18 3.18-1.18.63 1.59.23 2.77.11 3.06.74.81 1.19 1.84 1.19 3.1 0 4.43-2.7 5.41-5.27 5.69.41.36.78 1.07.78 2.15v3.19c0 .31.21.67.8.55 4.56-1.53 7.85-5.84 7.85-10.91C23.5 5.65 18.35.5 12 .5z"/>
    </svg>
  );
}

function TelegramIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden>
      <path d="M9.78 18.65l.28-4.23 7.68-6.92c.34-.31-.07-.46-.52-.19L7.74 13.3 3.64 12c-.88-.25-.89-.86.2-1.3l15.97-6.16c.73-.33 1.43.18 1.15 1.3l-2.72 12.81c-.19.91-.74 1.13-1.5.71L12.6 16.3l-1.99 1.93c-.23.23-.42.42-.83.42z"/>
    </svg>
  );
}
