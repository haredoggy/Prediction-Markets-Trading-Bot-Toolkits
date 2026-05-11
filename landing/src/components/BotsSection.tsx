import { bots } from '../bots';
import { useT } from '../messages';
import { BotCard } from './BotCard';

export function BotsSection() {
  const t = useT();
  return (
    <section id="strategies" className="py-24 border-t border-border-subtle">
      <div className="container-x">
        <div className="max-w-3xl mb-16">
          <div className="text-sm font-semibold text-purple-400 uppercase tracking-wider mb-3">{t.bots.eyebrow}</div>
          <h2 className="text-4xl md:text-5xl font-bold tracking-tight mb-5">
            {t.bots.headline}
          </h2>
          <p className="text-lg text-zinc-400 leading-relaxed">
            {t.bots.description}
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-5">
          {bots.map((bot, i) => (
            <BotCard key={bot.id} bot={bot} index={i} />
          ))}
        </div>
      </div>
    </section>
  );
}
