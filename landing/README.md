# Landing Page

Marketing landing page for the Polymarket Toolkits — React + Vite + TypeScript + Tailwind.

## Run locally

```bash
cd landing
npm install
npm run dev
```

Opens on `http://localhost:5173`.

## Build for production

```bash
npm run build
```

Outputs to `landing/dist/`. Deploy to any static host.

## Deploy to GitHub Pages

CI/CD is wired up at [`.github/workflows/deploy-landing.yml`](../.github/workflows/deploy-landing.yml). It builds and deploys automatically on every push to `main` that touches `landing/**`, or on manual dispatch.

**One-time setup on GitHub:**

1. Go to **Settings → Pages**
2. Under **Build and deployment**, set **Source** to **GitHub Actions**
3. Push to `main` (or run the workflow manually under the Actions tab)

The site will publish to `https://haredoggy.github.io/Prediction-Markets-Trading-Bot-Toolkits/`.

The workflow passes `VITE_BASE=/Prediction-Markets-Trading-Bot-Toolkits/` so asset URLs resolve correctly under the project-pages subpath. Local dev still uses `/` — no manual switching needed.

### Using a custom domain

1. Add a `CNAME` file to `landing/public/` containing your domain (e.g. `bots.example.com`)
2. Configure DNS to point at `haredoggy.github.io`
3. In the workflow, change `VITE_BASE` to `/` (custom domains live at the root)

## Structure

```
src/
├── App.tsx                 # Section orchestrator
├── bots.ts                 # Bot data (single source of truth — edit here)
├── main.tsx                # Entry
├── index.css               # Tailwind base + component classes
└── components/
    ├── Nav.tsx             # Sticky top nav with GitHub + Telegram CTAs
    ├── Hero.tsx            # Headline, dual CTA, performance stats teaser
    ├── BotsSection.tsx     # Section wrapper for the bot grid
    ├── BotCard.tsx         # Individual bot card
    ├── EngineSection.tsx   # Core features + performance metrics
    ├── SafetySection.tsx   # Risk layer + deployment ladder
    ├── CtaSection.tsx      # Final big CTA
    └── Footer.tsx          # Footer with disclaimer
```

## Editing content

- **Bot list / specs / hooks:** edit `src/bots.ts`. Card layout updates automatically.
- **Contact links:** `TELEGRAM_URL` and `GITHUB_URL` at the top of `src/bots.ts`.
- **Hero copy:** `src/components/Hero.tsx`.
- **Theme colors:** `tailwind.config.js` (`brand.polymarket`, `brand.kalshi`, `brand.limitless`, `bg.*`).

## Notes

- Tailwind v3 with JIT — the per-bot accent classes in `bots.ts` appear as literal strings so they survive content scanning.
- Strict TS, no unused locals — `npm run build` runs `tsc` before Vite.
