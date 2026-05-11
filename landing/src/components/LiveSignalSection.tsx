import { useEffect, useRef, useState } from 'react';
import {
  createChart,
  CandlestickSeries,
  type UTCTimestamp,
} from 'lightweight-charts';
import { useT } from '../messages';

const SYMBOL = 'btcusdt';
const INTERVAL = '1m';
const HISTORY_LIMIT = 240;

type ConnStatus = 'connecting' | 'live' | 'offline';
type TickDir = 'up' | 'down' | null;

interface Candle {
  time: UTCTimestamp;
  open: number;
  high: number;
  low: number;
  close: number;
}

export function LiveSignalSection() {
  const t = useT();
  const containerRef = useRef<HTMLDivElement>(null);
  const prevPriceRef = useRef<number | null>(null);
  const [price, setPrice] = useState<number | null>(null);
  const [tickDir, setTickDir] = useState<TickDir>(null);
  const [status, setStatus] = useState<ConnStatus>('connecting');
  const [change24h, setChange24h] = useState<number | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const chart = createChart(containerRef.current, {
      layout: {
        background: { color: 'transparent' },
        textColor: '#a1a1aa',
        fontFamily: 'JetBrains Mono, ui-monospace, monospace',
      },
      grid: {
        vertLines: { color: 'rgba(31, 31, 42, 0.6)' },
        horzLines: { color: 'rgba(31, 31, 42, 0.6)' },
      },
      timeScale: {
        borderColor: '#1f1f2a',
        timeVisible: true,
        secondsVisible: false,
      },
      rightPriceScale: {
        borderColor: '#1f1f2a',
      },
      crosshair: {
        mode: 1,
        vertLine: { color: '#52525b', width: 1, style: 2 },
        horzLine: { color: '#52525b', width: 1, style: 2 },
      },
      autoSize: true,
    });

    const series = chart.addSeries(CandlestickSeries, {
      upColor: '#22c55e',
      downColor: '#ef4444',
      borderUpColor: '#22c55e',
      borderDownColor: '#ef4444',
      wickUpColor: '#22c55e',
      wickDownColor: '#ef4444',
    });

    let cancelled = false;

    // Historical klines (public CDN-style endpoint, works globally)
    fetch(
      `https://data-api.binance.vision/api/v3/klines?symbol=${SYMBOL.toUpperCase()}&interval=${INTERVAL}&limit=${HISTORY_LIMIT}`,
    )
      .then((r) => {
        if (!r.ok) throw new Error(`rest ${r.status}`);
        return r.json();
      })
      .then((rows: unknown) => {
        if (cancelled || !Array.isArray(rows)) return;
        const candles: Candle[] = rows.map((row) => {
          const r = row as [number, string, string, string, string, ...unknown[]];
          return {
            time: Math.floor(r[0] / 1000) as UTCTimestamp,
            open: parseFloat(r[1]),
            high: parseFloat(r[2]),
            low: parseFloat(r[3]),
            close: parseFloat(r[4]),
          };
        });
        series.setData(candles);
        if (candles.length > 0) {
          const last = candles[candles.length - 1].close;
          const first = candles[0].open;
          prevPriceRef.current = last;
          setPrice(last);
          setChange24h(((last - first) / first) * 100);
        }
      })
      .catch(() => {
        // Silent — WS will populate as ticks arrive.
      });

    let ws: WebSocket | null = null;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let backoffMs = 1000;

    const connect = () => {
      if (cancelled) return;
      ws = new WebSocket(`wss://stream.binance.com:9443/ws/${SYMBOL}@kline_${INTERVAL}`);

      ws.onopen = () => {
        if (cancelled) return;
        backoffMs = 1000;
        setStatus('live');
      };

      ws.onerror = () => {
        if (!cancelled) setStatus('offline');
      };

      ws.onclose = () => {
        if (cancelled) return;
        setStatus('offline');
        reconnectTimer = setTimeout(connect, backoffMs);
        backoffMs = Math.min(backoffMs * 2, 30_000);
      };

      ws.onmessage = (event) => {
        if (cancelled) return;
        try {
          const msg = JSON.parse(event.data as string);
          const k = msg.k;
          if (!k) return;
          const candle: Candle = {
            time: Math.floor(k.t / 1000) as UTCTimestamp,
            open: parseFloat(k.o),
            high: parseFloat(k.h),
            low: parseFloat(k.l),
            close: parseFloat(k.c),
          };
          series.update(candle);

          const close = candle.close;
          const prev = prevPriceRef.current;
          if (prev !== null && close !== prev) {
            setTickDir(close > prev ? 'up' : 'down');
          }
          prevPriceRef.current = close;
          setPrice(close);
        } catch {
          // ignore malformed frames
        }
      };
    };

    connect();

    return () => {
      cancelled = true;
      if (reconnectTimer) clearTimeout(reconnectTimer);
      if (ws) {
        try {
          ws.close();
        } catch {
          // ignore
        }
      }
      chart.remove();
    };
  }, []);

  return (
    <section id="live-signal" className="py-20 border-t border-border-subtle">
      <div className="container-x">
        <div className="flex flex-col md:flex-row md:items-end md:justify-between gap-6 mb-8">
          <div>
            <div className="text-sm font-semibold text-amber-400 uppercase tracking-wider mb-2 flex items-center gap-2">
              <StatusPill status={status} />
              {t.liveSignal.eyebrow}
            </div>
            <h2 className="text-3xl md:text-4xl font-bold tracking-tight">
              {t.liveSignal.headline}
            </h2>
            <p className="mt-2 text-sm text-zinc-400">
              {t.liveSignal.sub}
            </p>
          </div>

          <PriceTicker price={price} direction={tickDir} change24h={change24h} />
        </div>

        <div className="card p-3 md:p-4 bg-gradient-to-br from-bg-surface to-bg-elevated">
          <div ref={containerRef} className="w-full h-[360px] md:h-[440px]" />
        </div>

        <div className="mt-5 flex flex-col sm:flex-row sm:items-start sm:justify-between gap-3 text-xs text-zinc-500 max-w-4xl">
          <p className="leading-relaxed">
            {t.liveSignal.footnote({
              source: <span className="text-zinc-400">Binance spot WebSocket</span>,
              chainlink: <span className="text-zinc-400">Chainlink price feeds</span>,
            })}
          </p>
          <a
            href="https://www.tradingview.com/lightweight-charts/"
            target="_blank"
            rel="noreferrer"
            className="flex-shrink-0 text-zinc-500 hover:text-zinc-300 transition-colors whitespace-nowrap"
          >
            {t.liveSignal.attribution}
          </a>
        </div>
      </div>
    </section>
  );
}

function PriceTicker({
  price,
  direction,
  change24h,
}: {
  price: number | null;
  direction: TickDir;
  change24h: number | null;
}) {
  const t = useT();
  const tickColor =
    direction === 'up'
      ? 'text-emerald-400'
      : direction === 'down'
        ? 'text-rose-400'
        : 'text-zinc-100';

  const changeColor =
    change24h === null
      ? 'text-zinc-500'
      : change24h >= 0
        ? 'text-emerald-400'
        : 'text-rose-400';

  return (
    <div className="font-mono text-right">
      <div className="text-[10px] text-zinc-500 uppercase tracking-[0.2em] mb-1">{t.liveSignal.pair}</div>
      <div className={`text-3xl md:text-5xl font-bold tabular-nums transition-colors duration-150 ${tickColor}`}>
        {price === null
          ? '—'
          : `$${price.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}`}
      </div>
      {change24h !== null && (
        <div className={`text-sm font-semibold mt-1 ${changeColor}`}>
          {change24h >= 0 ? '+' : ''}
          {change24h.toFixed(2)}% <span className="text-zinc-500 font-normal">{t.liveSignal.sessionLabel}</span>
        </div>
      )}
    </div>
  );
}

function StatusPill({ status }: { status: ConnStatus }) {
  const t = useT();
  if (status === 'live') {
    return (
      <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/30 text-[10px] font-bold text-emerald-400 uppercase tracking-wider">
        <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
        {t.liveSignal.statusLive}
      </span>
    );
  }
  if (status === 'offline') {
    return (
      <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-rose-500/10 border border-rose-500/30 text-[10px] font-bold text-rose-400 uppercase tracking-wider">
        <span className="w-1.5 h-1.5 rounded-full bg-rose-400" />
        {t.liveSignal.statusOffline}
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-amber-500/10 border border-amber-500/30 text-[10px] font-bold text-amber-400 uppercase tracking-wider">
      <span className="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse" />
      {t.liveSignal.statusConnecting}
    </span>
  );
}
