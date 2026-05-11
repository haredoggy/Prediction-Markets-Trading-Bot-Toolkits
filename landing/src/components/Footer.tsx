import { TELEGRAM_URL, GITHUB_URL } from '../bots';

export function Footer() {
  return (
    <footer className="border-t border-border-subtle py-12">
      <div className="container-x">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-6">
          <div className="flex items-center gap-3">
            <span className="w-8 h-8 rounded-lg bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center text-sm font-bold">
              P
            </span>
            <div>
              <div className="font-bold text-white">Prediction Market Toolkits</div>
              <div className="text-xs text-zinc-500">Polymarket · Kalshi · Limitless</div>
            </div>
          </div>

          <div className="flex flex-wrap items-center gap-x-6 gap-y-2 text-sm text-zinc-400">
            <a href="#strategies" className="hover:text-white transition-colors">Strategies</a>
            <a href="#engine" className="hover:text-white transition-colors">Engine</a>
            <a href="#safety" className="hover:text-white transition-colors">Safety</a>
            <a href={GITHUB_URL} target="_blank" rel="noreferrer" className="hover:text-white transition-colors">GitHub</a>
            <a href={TELEGRAM_URL} target="_blank" rel="noreferrer" className="hover:text-white transition-colors">Telegram</a>
          </div>
        </div>

        <div className="mt-10 pt-6 border-t border-border-subtle">
          <p className="text-xs text-zinc-500 leading-relaxed max-w-4xl">
            <span className="font-semibold text-zinc-400">Disclaimer.</span>{' '}
            Trading prediction markets involves real financial risk. This software is provided as-is,
            without warranty or guarantee of any outcome. It is not financial advice. Always test with
            <code className="mx-1 px-1.5 py-0.5 bg-bg-elevated rounded text-zinc-300 font-mono text-[11px]">enable_trading: false</code>
            before deploying real capital. Ensure compliance with each venue's terms of service and
            applicable regulations in your jurisdiction.
          </p>
          <p className="mt-4 text-xs text-zinc-600">
            © {new Date().getFullYear()} · MIT Licensed · Built for the prediction markets community.
          </p>
        </div>
      </div>
    </footer>
  );
}
