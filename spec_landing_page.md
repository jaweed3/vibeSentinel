# VibeSentinel — Landing Page Spec

> Static site · Vite · GitHub Pages · Industrial Edge AI

---

## Brand Overview

| Attribute | Value |
|---|---|
| Company | VibeSentinel |
| Tagline | *Predict failures before they happen. No cloud. No WiFi. Just a \$10 chip.* |
| One-liner | Edge AI predictive maintenance — autoencoder anomaly detection on ESP32-S3, 100% Rust. |
| Demo | [asciinema recording](./assets/demo.gif) showing test → train → infer → export pipeline |
| Repository | [github.com/jaweed3/vibeSentinel](https://github.com/jaweed3/vibeSentinel) |
| Domain | vibesentinel.dev (or use `{user}.github.io/vibeSentinel`) |
| Deploy | GitHub Pages via `gh-pages` branch or `docs/` folder |

---

## Design Direction

**Theme:** Dark, industrial-cyber. Not a fluffy startup — this is a factory-floor tool.

| Token | Value | Usage |
|---|---|---|
| `--bg` | `#0a0e14` | Page background |
| `--surface` | `#131a22` | Card / section backgrounds |
| `--border` | `#1e2a36` | Subtle separators |
| `--accent` | `#00f0ff` (cyan) | Primary CTAs, links, active states |
| `--accent2` | `#ff3b30` (red) | Anomaly / alert indicators |
| `--muted` | `#6b7b8d` | Secondary text |
| `--text` | `#e6edf3` | Body text |
| `--font-mono` | `'JetBrains Mono', 'Fira Code', monospace` | Code blocks, metrics |
| `--font-sans` | `'Inter', system-ui, sans-serif` | Body text, headings |

Typography: Inter (headings/body) + JetBrains Mono (code/metrics). Fluid type scale: `clamp()`.

---

## Page Structure

### 1. Hero Section

Full-viewport dark background. Animated particle grid or subtle oscilloscope waveform (CSS-only).

```
┌──────────────────────────────────────────────────────┐
│  [logo] VibeSentinel                        [GitHub] │
│                                                      │
│         Predict machine failure before it             │
│         happens. On a $10 chip. No cloud.             │
│                                                      │
│  [subtitle]                                           │
│  Autoencoder anomaly detection running on ESP32-S3.   │
│  524 parameters. 100% Rust. Real-time inference.      │
│                                                      │
│     [Try the Demo ▸]    [View on GitHub ▸]           │
│                                                      │
│  ┌───┐ ┌───┐ ┌───┐ ┌───┐                             │
│  │524│ │26 │ │200│ │0  │  <- live metrics ticker      │
│  │params│features│Hz │ms latency│                     │
│  └───┘ └───┘ └───┘ └───┘                             │
└──────────────────────────────────────────────────────┘
```

**Elements:**
- Logo: text-based monogram "VS" in cyan on dark, or SVG waveform icon
- Headline: two lines, bold, large (`clamp(2rem, 5vw, 4rem)`)
- Subtitle: smaller, muted, mono font for the technical detail
- Two CTAs: "Try the Demo" (primary, cyan) and "View on GitHub" (outline)
- Metrics bar: 4 key numbers that auto-update or animate in on load

### 2. Problem Section

```
┌──────────────────────────────────────────────────────┐
│  $1 trillion — the annual cost of unplanned downtime  │
│  in industrial manufacturing.                         │
│                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
│  │  Reactive    │  │  Scheduled  │  │ Condition-  │   │
│  │  Maintenance │  │  Maintenance│  │ Based (Ours)│   │
│  │  Fix when    │  │  Replace    │  │ Predict     │   │
│  │  it breaks   │  │  on calendar│  │ before fail │   │
│  │  $$$$$$      │  │  $$$        │  │  $           │   │
│  └─────────────┘  └─────────────┘  └─────────────┘   │
│                                                      │
│  Most factories still use reactive maintenance.       │
│  "Smart" solutions send raw data to the cloud —       │
│  expensive, slow, privacy-hostile, and impractical    │
│  at scale.                                            │
└──────────────────────────────────────────────────────┘
```

### 3. Solution Section

"Edge AI. Zero compromises."

Split panel: left text, right visualized pipeline (Mermaid or SVG).

```
┌──────────────────────┬───────────────────────────────┐
│  [Headline]          │  [Pipeline Visualization]     │
│                      │                               │
│  We put a neural net │  IMU ──▶ Window ──▶ Features │
│  on a $10 ESP32-S3.  │         (128smp)    (26-dim) │
│  It learns your      │                               │
│  machine's "normal"  │  Features ──▶ Normalize ──▶ AE│
│  vibration signature │  (stats+FFT)   (Z-score)     │
│  and detects         │                   (20→10→4→10│
│  anomalies in        │                    →20)       │
│  real-time — right   │                               │
│  on the sensor.      │  AE ──▶ MSE ──▶ Threshold?   │
│                      │         calc      (E010)     │
│  No cloud.           │                   │           │
│  No bandwidth bills. │             [Red LED]         │
│  No privacy risks.   └───────────────────────────────┘
└──────────────────────────────────────────────────────┘
```

### 4. Technology Stack

Grid of 6 tech cards with icons.

```
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│ 100% Rust│  │ Autoenc. │  │   no_std │  │Burn 0.14│
│ Whole    │  │ 20→10→4  │  │ Zero heap│  │NDArray  │
│ pipeline │  │ →10→20   │  │ Zero alloc│  │backend  │
├──────────┤  ├──────────┤  ├──────────┤  ├──────────┤
│ Golden   │  │  ESP32-S3│  │  3-Sigma │  │ E001-010│
│ Vectors  │  │ LSM6DS3  │  │ Threshold│  │Structured│
│ 1e-5 tol │  │ 200Hz I2C│  │Calibrat. │  │Error Code│
└──────────┘  └──────────┘  └──────────┘  └──────────┘
```

Each card: icon (SVG or emoji) + title + one-liner.

### 5. Pipeline / How It Works

Numbered steps, full-width, alternating layout.

```
Step 1: SAMPLE ──── [icon: IMU chip]
  LSM6DS3 reads 3-axis acceleration at 200Hz.
  128 samples per window (640ms), 50% overlap.

Step 2: EXTRACT ──── [icon: waveform → bins]
  Hann window → 128-pt RFFT → magnitude spectrum
  Per-axis: RMS, Peak, Kurtosis, Crest Factor, FFT bins 0-3
  Cross-axis: Axial/Radial ratio, Total RMS
  → 26-dimensional feature vector

Step 3: INFER ──── [icon: neural net]
  Z-score normalize (clip ±3)
  Autoencoder: 20→10→4→10→20
  524 parameters, ~2KB, <1ms forward pass

Step 4: DETECT ──── [icon: alert triangle]
  MSE reconstruction error > threshold?
  3-sigma calibration from training data
  Triggers LED alert (E010) + serial health report
```

### 6. Demo Embed

Embed the asciinema recording from `./assets/demo.gif` with a play button overlay. Below it, a tabbed view:

```
┌──────────────────────────────────────────────────┐
│  ▶  VibeSentinel Demo — Test, Train, Infer       │
│  ┌──────────────────────────────────────────────┐ │
│  │              [asciinema GIF]                 │ │
│  └──────────────────────────────────────────────┘ │
│                                                   │
│  [Test]    [Train]    [Golden Vectors]    [Export]│
│                                                   │
│  Active tab shows relevant terminal output +      │
│  explanation of what's happening.                 │
└──────────────────────────────────────────────────┘
```

### 7. Roadmap Timeline

Visual timeline split by phase:

```
DONE ─── Phase 1: Core
  Features extraction · Autoencoder · Trainer · Firmware

NOW ──── Phase 2: Connectivity
  WiFi · MQTT · OTA updates · SD card logging

NEXT ─── Phase 3: Robustness
  Multi-model switching · Adaptive threshold · Multi-sensor

SOON ─── Phase 4: Advanced ML
  INT8 quantization · On-device fine-tuning · Sensor fusion

LATER ── Phase 5: Production
  Dashboard · Notifications · Multi-device · CI/CD
```

Each phase card: status badge, 2-3 bullet items, link to GitHub milestone.

### 8. Technical Metrics Strip

Full-width horizontal strip with animated counters:

```
┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
│ 524  │ │ 26   │ │ 200  │ │ 128  │ │ <1ms │ │ 31   │
│ Params│ │ Feats│ │ Hz   │ │Window│ │Inf.  │ │Tests │
└──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘
```

### 9. Call to Action

```
┌──────────────────────────────────────────────────────┐
│  Ready to deploy?                                    │
│  ┌──────────────────────┐  ┌──────────────────────┐  │
│  │  Get Started →       │  │  Star on GitHub →    │  │
│  │  docs/getting-started│  │  github.com/jaweed3  │  │
│  └──────────────────────┘  └──────────────────────┘  │
│                                                      │
│  Or just run:                                        │
│  ┌──────────────────────────────────────────────┐    │
│  │  git clone https://github.com/jaweed3/       │    │
│  │       vibeSentinel.git                        │    │
│  │  cd vibeSentinel                              │    │
│  │  cargo run --release -p vibesentinel-trainer  │    │
│  └──────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────┘
```

### 10. Footer

```
┌──────────────────────────────────────────────────────┐
│  VibeSentinel  ·  Edge AI Predictive Maintenance     │
│                                                      │
│  Built with Burn, microfft, ESP-IDF-HAL              │
│  100% Rust  ·  MIT License  ·  Open Source           │
│                                                      │
│  [GitHub] [Twitter/X] [Email]                        │
└──────────────────────────────────────────────────────┘
```

---

## Technical Implementation

### Stack

```
Vite (vanilla TS)   →  Build tool + dev server
TypeScript          →  Type safety
CSS (no framework)  →  Custom properties, no Tailwind
GitHub Pages        →  Deploy target
```

No React. No SPA framework. This is a static marketing site — vanilla TS keeps it fast, zero JS bundle for most of the page.

### Project Structure

```
landing/
├── index.html           # Entry point
├── vite.config.ts       # Vite conf with GH Pages base
├── tsconfig.json
├── package.json         # vite, typescript only
├── src/
│   ├── main.ts          # IntersectionObserver animations, metrics counter
│   ├── style.css        # Global styles + CSS custom properties
│   ├── components/
│   │   ├── hero.ts      # Hero section render + particle canvas
│   │   ├── metrics.ts   # Animated metrics bar
│   │   ├── tabs.ts      # Demo tab switcher
│   │   ├── timeline.ts  # Roadmap timeline
│   │   └── footer.ts
│   └── lib/
│       ├── animate.ts   # Counter animation, fade-in on scroll
│       └── particles.ts # Background particle grid (Canvas 2D)
├── public/
│   ├── demo.gif         # Asciinema GIF from /assets
│   └── og-image.png     # Open Graph preview
└── github-pages.yml     # GitHub Actions deploy workflow
```

### Key Behaviors

| Element | Behavior |
|---|---|
| Metrics bar | Count up from 0 on scroll into viewport. Pause at final value. |
| Hero particles | 60 FPS Canvas 2D particle grid. 200 particles. 1px lines between nearby particles. Drops to 30 FPS on mobile. |
| Demo tabs | Click switches GIF + annotation text. No page reload. CSS transition on opacity. |
| Tech cards | Scale(1.02) + cyan border on hover. Transition 200ms ease. |
| Timeline | Scroll-triggered line drawing. Phases fade in sequentially. |
| Pipeline steps | Alternating left/right layout on desktop, stacked on mobile. |

### Responsive Breakpoints

| Breakpoint | Layout |
|---|---|
| `>1024px` | Desktop: multi-column, full hero, side-by-side pipeline |
| `768-1024px` | Tablet: 2-column grids collapse to single |
| `<768px` | Mobile: stacked everything, smaller hero text, hamburger nav |

### Deploy

```yaml
# .github/workflows/deploy.yml
name: Deploy landing page
on:
  push:
    branches: [main]
    paths: ['landing/**']
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: npm ci && npm run build
        working-directory: landing
      - uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: landing/dist
```

### Performance Budget

| Metric | Target |
|---|---|
| Lighthouse Performance | ≥95 |
| First Contentful Paint | <1.5s |
| Total JS | <50KB (uncompressed) |
| Total page weight | <500KB (including GIF) |
| No render-blocking resources | Inline critical CSS |

### SEO

- Open Graph tags (title, description, image, url)
- Twitter card (summary_large_image)
- `og-image.png` — 1200×630, dark background + VibeSentinel logo + tagline
- `robots.txt`, `sitemap.xml`

---

## Color Strategy

The palette is dark industrial with cyan as the "intelligence" color and red as the "alert" color. This mirrors the actual hardware behavior (green LED normal, red LED anomaly).

| Context | Color |
|---|---|
| Normal state | Cyan `#00f0ff` — inference running, no anomaly |
| Alert state | Red `#ff3b30` — MSE threshold breached |
| Background | Near-black `#0a0e14` — the dark factory floor |
| Code / metrics | Mono on dark surface |
| Body text | Cool white `#e6edf3` |

---

## Copy Tone

- **Technical but not academic.** This is an engineering tool, not a research paper.
- **Direct.** "We put a neural network on a $10 chip." Not "Our solution leverages deep learning architectures deployed on low-cost microcontroller platforms."
- **Concrete numbers everywhere.** 524 parameters, 200Hz, 128 samples, <1ms, $10, 26 features, E001-E010.
- **No marketing fluff.** Every claim is backed by a test or a constant in the codebase.
- **Indonesian-friendly.** The intern guide is in Indonesian, which is a notable community aspect.

---

## Assets Needed

| Asset | Source | Format |
|---|---|---|
| Logo (monogram) | Text-based "VS" in cyan + mono font | Inline SVG |
| Logo (full) | Same + "VibeSentinel" text | Inline SVG |
| Demo GIF | `assets/demo.gif` | Optimized GIF/WebP |
| OG image | Generated from hero section | PNG 1200×630 |
| Pipeline diagram | Mermaid or hand-crafted SVG | SVG |
| Arrow icons | Custom or Phosphor icons | SVG sprites |
| Tech stack icons | Simple SVG badges | Inline SVG |

---

## Phases Beyond MVP

| Phase | Landing Page Update |
|---|---|
| Phase 2 ships | Add "Live Dashboard" screenshot, MQTT telemetry section |
| Phase 3 ships | Add "Multi-Model" toggle demo, adaptive threshold viz |
| Phase 4 ships | Add benchmark graph: INT8 vs FP32 speed/memory comparison |
| Phase 5 ships | Add "Try Cloud Dashboard" CTA, multi-device topology viz |
