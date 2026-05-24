# VibeSentinel — Edge AI Predictive Maintenance

> **100% Rust.** Autoencoder anomaly detection. ESP32-S3 deployment. 524 parameters.

<p align="center">
  <img src="./assets/demo.gif" alt="VibeSentinel Demo — Tests, Training &amp; Inference" width="100%">
</p>

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Pipeline](#pipeline)
- [Getting Started](#getting-started)
- [Debugging](#debugging)
- [Documentation](#documentation)

---

## Overview

VibeSentinel is an end-to-end Edge AI system for **predictive maintenance**. It monitors industrial machinery vibration using an Autoencoder neural network running directly on an **ESP32-S3** microcontroller — no cloud, no WiFi required.

### Key Differentiators

| | |
|---|---|
| **100% Rust** | Training (Burn), inference (no_std), and firmware (ESP-IDF) |
| **Tiny model** | 20→10→4→10→20 Autoencoder, **524 parameters**, ~2 KB |
| **no_std core** | Feature extraction and model crates are `#![no_std]`, zero heap, zero alloc |
| **Cross-arch parity** | Golden vectors verify bit-exact output between x86 and Xtensa |
| **Structured errors** | E001–E010 error codes for every failure mode |

### Demo

The recording above walks through:

1. **Cargo test** — 31 unit tests across feature extraction and model inference
2. **Training** — 50 epochs on synthetic vibration data + calibration report
3. **Inference** — Golden vector verification for cross-architecture parity
4. **Export** — Auto-generated `weights.rs` with threshold, mean, std, and golden vectors

---

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌──────────────────────┐
│  vibesentinel-  │     │  vibesentinel-   │     │  vibesentinel-       │
│  features       │────▶│  model            │◀────│  trainer             │
│  (no_std)       │     │  (no_std)         │     │  (std / Burn)        │
│  DSP pipeline   │     │  Autoencoder      │     │  Adam · MSE · 200ep  │
└─────────────────┘     └───────┬──────────┘     └──────────────────────┘
                                │
                    ┌───────────▼───────────┐
                    │  vibesentinel-        │
                    │  firmware             │
                    │  ESP32-S3 · LSM6DS3   │
                    │  200 Hz · LED alert   │
                    └───────────────────────┘
```

### Crates

| Crate | Responsibility | Target |
|---|---|---|
| `vibesentinel-features` | FFT, RMS, Kurtosis, Crest Factor, spectral centroid | `no_std` |
| `vibesentinel-model` | Linear layers, ReLU/Sigmoid activations, AE forward pass | `no_std` |
| `vibesentinel-trainer` | Adam optimizer, MSE loss, threshold calibration, weight export | Desktop (Burn) |
| `vibesentinel-firmware` | 200 Hz sampling loop, I2C driver, GPIO alert, health reporting | ESP32-S3 |

---

## Pipeline

```
Sensor ──▶ 128-sample window ──▶ 26 features ──▶ Z-score norm ──▶ AE ──▶ MSE > threshold? ──▶ LED
           (3-axis @ 200 Hz)      (FFT + Stats)    (clip ±3)      (20→10→4→10→20)    (E010)
```

### Feature Vector (26-dim)

- **Per-axis** (X, Y, Z): RMS, Peak, Kurtosis, Crest Factor, FFT bins 0–3 = 8 each × 3 = 24
- **Cross-axis**: Axial/Radial ratio, Total RMS = 2

### Model

| Layer | Shape | Params |
|---|---|---|
| Encoder 1 | 20 → 10 + ReLU | 210 |
| Encoder 2 | 10 → 4 + ReLU | 44 |
| Decoder 1 | 4 → 10 + ReLU | 50 |
| Decoder 2 | 10 → 20 + Sigmoid | 220 |
| **Total** | | **524** |

---

## Getting Started

### Requirements

- **Rust**: Nightly toolchain (see `rust-toolchain.toml`)
- **Hardware**: ESP32-S3 (tested on Seeed Studio XIAO ESP32S3 Sense)
- **IMU**: LSM6DS3 (I2C 0x6A) or MPU-6050 (0x68)
- **Python**: 3.11 (ESP-IDF build)

### Train

```bash
# Synthetic data (default: 200 epochs)
cargo run --release -p vibesentinel-trainer

# Real CSV data with custom params
cargo run --release -p vibesentinel-trainer -- \
    --data data/normal_vibration.csv \
    --epochs 200 \
    --learning-rate 0.001 \
    --sigma 3.0
```

### Test

```bash
# Core no_std crates
cargo test -p vibesentinel-features -p vibesentinel-model
```

### Flash

```bash
./scripts/build.sh
espflash flash target/xtensa-esp32s3-espidf/release/vibesentinel-firmware --monitor
```

---

## Debugging

Structured error codes printed as `[E###]` over serial:

| Code | Meaning | Fix |
|---|---|---|
| E001 | I2C timeout | Check wiring, pull-up resistors |
| E002 | IMU not detected | Check 3.3V, I2C address (0x6A) |
| E003 | Sensor frozen | Tap sensor, check connection |
| E005 | Heap < 50 KB | Check for leaks |
| E007 | NaN in features | Check sensor data |
| E008 | NaN in inference | Corrupted weights |
| E009 | I2C recovery failed | Power-cycle sensor |
| E010 | Signal saturation | Increase G-range |

Health reports: `[HEALTH] uptime=... windows=... heap=... errors=...`

---

## Documentation

| Guide | Description |
|---|---|
| [Intern Guide](./docs/intern-guide.md) | Full walkthrough (Indonesian) |
| [Architecture](./docs/architecture.md) | Crate responsibilities & data flow |
| [Getting Started](./docs/getting-started.md) | Setup & deployment |
| [Roadmap](./docs/current-state-future-implementation.md) | Milestones & future vision |

### Planning

| File | Description |
|---|---|
| [`vibesentinel_context.md`](./vibesentinel_context.md) | Technical reference: constants, build commands, file map, conventions |
| [`agents.md`](./agents.md) | AI agent configuration: roles, workflows, phase guidance |
| [`design.md`](./design.md) | Design decisions (ADRs): why Rust, why autoencoder, trade-offs |
| [GitHub Issues](https://github.com/jaweed3/vibeSentinel/issues) | Phase 2–5 milestones with granular tasks |

### Roadmap

| Phase | Focus | Target |
|---|---|---|
| 1 | Core: features, model, trainer, firmware | Done |
| [2](https://github.com/jaweed3/vibeSentinel/milestone/3) | Connectivity: WiFi, MQTT, OTA, SD card | Jul 2026 |
| [3](https://github.com/jaweed3/vibeSentinel/milestone/4) | Robustness: multi-model, adaptive threshold, multi-sensor | Sep 2026 |
| [4](https://github.com/jaweed3/vibeSentinel/milestone/5) | Advanced ML: INT8, fine-tuning, sensor fusion | Nov 2026 |
| [5](https://github.com/jaweed3/vibeSentinel/milestone/6) | Production: dashboard, notifications, CI/CD, scale | Jan 2027 |

---

*Built with [Burn](https://burn.dev), [microfft](https://crates.io/crates/microfft), and [ESP-IDF-HAL](https://github.com/esp-rs/esp-idf-hal).*
