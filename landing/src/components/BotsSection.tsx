import { bots } from '../bots';
import { BotCard } from './BotCard';

export function BotsSection() {
  return (
    <section id="strategies" className="py-24 border-t border-border-subtle">
      <div className="container-x">
        <div className="max-w-3xl mb-16">
          <div className="text-sm font-semibold text-purple-400 uppercase tracking-wider mb-3">The Lineup</div>
          <h2 className="text-4xl md:text-5xl font-bold tracking-tight mb-5">
            Ten strategies. One engine.
          </h2>
          <p className="text-lg text-zinc-400 leading-relaxed">
            Each bot targets a distinct, well-defined market edge — copy trading, arbitrage, market making, on-chain signals.
            All share the same battle-tested execution core, risk layer, and venue-agnostic adapter stack.
            Pick the edge that fits your thesis; the infrastructure is already built.
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
