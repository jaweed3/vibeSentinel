# VibeSentinel Context

> Technical reference for AI agents and developers working on VibeSentinel.

---

## Project Overview

Edge AI predictive maintenance system. Autoencoder anomaly detection on ESP32-S3. 100% Rust.

| Aspect | Detail |
|---|---|
| Language | Rust (nightly) |
| ML Framework | Burn 0.14 (NDArray backend) |
| Target | ESP32-S3 (xtensa-esp32s3-espidf) |
| no_std crates | `vibesentinel-features`, `vibesentinel-model` |
| std crate | `vibesentinel-trainer` |
| Firmware | `vibesentinel-firmware` (esp-idf) |

---

## Crate Dependency Graph

```
vibesentinel-trainer (std)
  ├── vibesentinel-features (no_std, feature = "std")
  └── vibesentinel-model (no_std, feature = "std")

vibesentinel-firmware (esp-idf)
  ├── vibesentinel-features (no_std)
  └── vibesentinel-model (no_std)
```

## Key Constants (`crates/vibesentinel-features/src/lib.rs`)

| Constant | Value | Description |
|---|---|---|
| `WINDOW_SIZE` | 128 | Samples per axis per window |
| `INPUT_DIM` | 20 | Features fed into autoencoder |
| `HIDDEN1_DIM` | 10 | Encoder layer 1 output |
| `LATENT_DIM` | 4 | Bottleneck layer |
| `HIDDEN2_DIM` | 10 | Decoder layer 1 output |
| `OUTPUT_DIM` | 20 | Decoder layer 2 output (matches INPUT) |
| `FEATURE_DIM` | 26 | Total raw features extracted (before selection) |

Note: `INPUT_DIM=20` while `FEATURE_DIM=26`. The extractor produces 26 features, but the autoencoder uses a 20-dimensional subset. The remaining 6 features are reserved for future expansion.

## Architecture: Autoencoder

```
Input(20) → Linear(20→10) + ReLU → Linear(10→4) + ReLU → Linear(4→10) + ReLU → Linear(10→20) + Sigmoid → Output(20)
```

524 total parameters (~2KB).

## Build Commands

```bash
# Core tests (no_std)
cargo test -p vibesentinel-features -p vibesentinel-model

# Training with synthetic data
cargo run --release -p vibesentinel-trainer

# Training with custom params
cargo run --release -p vibesentinel-trainer -- \
    --data data/normal_vibration.csv \
    --epochs 200 \
    --learning-rate 0.001 \
    --sigma 3.0 \
    --aug-scale 0.0 \
    --output-path crates/vibesentinel-model/src/weights.rs

# Build firmware
./scripts/build.sh

# Full pipeline (train + build firmware)
./build-firmware.sh
```

## Coding Conventions

1. **no_std crates**: No allocator, no heap, no `std::vec::Vec`, no `std::string::String`. Everything is fixed-size stack arrays.
2. **Math**: Use `libm` for floating-point operations (`libm::sqrtf`, `libm::powf`, `libm::sinf`).
3. **FFT**: Use `microfft::rfft_128` for 128-point real FFT.
4. **Comments**: Do not add obvious comments. Code should be self-documenting.
5. **Tests**: Every public function must have unit tests. Tests run via `cargo test`.
6. **Weights**: `weights.rs` is auto-generated — never edit manually. Always regenerate after architecture changes.
7. **Error handling**: Firmware uses structured error codes E001–E010.
8. **Floating point precision**: Tests use `1e-5` tolerance for cross-architecture parity.

## File Map

```
vibesentinel/
├── crates/
│   ├── vibesentinel-features/src/
│   │   ├── lib.rs        # Module declarations, constants
│   │   ├── stats.rs      # RMS, Peak, Kurtosis, Crest Factor, Variance, Skewness
│   │   ├── fft.rs        # Hann window, 128-pt RFFT, spectral centroid
│   │   └── extractor.rs  # AccelWindow, 26-dim feature extraction
│   ├── vibesentinel-model/src/
│   │   ├── lib.rs        # Module declarations
│   │   ├── arch.rs       # Autoencoder forward, reconstruction_error, normalize_features
│   │   ├── activations.rs# ReLU, Sigmoid (libm)
│   │   ├── matmul.rs     # Generic linear_forward<IN, OUT>()
│   │   └── weights.rs    # AUTO-GENERATED: weights, biases, threshold, golden vectors
│   ├── vibesentinel-trainer/src/
│   │   ├── main.rs       # CLI entry (clap)
│   │   ├── model.rs      # Burn autoencoder module
│   │   ├── train.rs      # Training loop, validation, threshold calibration
│   │   ├── dataset.rs    # CSV loader, synthetic data, augmentation
│   │   └── export.rs     # Weight export to Rust source
│   └── vibesentinel-firmware/src/
│       ├── main.rs       # Main loop: sample → features → inference → alert
│       ├── config.rs     # Board config, sampling rate, WDT, I2C recovery
│       ├── imu.rs        # LSM6DS3 I2C driver
│       ├── alert.rs      # GPIO LED control
│       └── debug.rs      # Error codes E001-E010, health reports
├── data/
│   ├── normal_vibration.csv    # 120k rows, 10 min @ 200Hz
│   └── anomaly_vibration.csv   # 12k rows anomaly data
├── docs/
│   ├── intern-guide.md         # Full Indonesian walkthrough
│   ├── overview.md             # Project goals
│   ├── architecture.md         # Crate responsibilities & data flow
│   ├── getting-started.md      # Quick setup
│   ├── hackathon-pitch.md      # Pitch deck
│   └── current-state-future-implementation.md  # Roadmap
├── assets/
│   └── demo.gif                # Asciinema demo recording
├── scripts/
│   ├── bootstrap.sh             # ESP-IDF environment setup
│   └── build.sh                 # Firmware build
├── build-firmware.sh            # End-to-end: train + firmware build
├── README.md
├── Cargo.toml                   # Workspace root
├── rust-toolchain.toml          # nightly, rust-src, clippy
└── .gitignore
```

## Feature Extraction Pipeline

```
Raw XYZ [f32; 128] each
  → Hann window
  → 128-pt RFFT → magnitudes [f32; 65]
  → Per-axis: RMS, Peak, Kurtosis, Crest Factor, FFT bins 0-3 = 8 features
  → Cross-axis: Axial/Radial ratio, Total RMS = 2 features
  → Total: 8×3 + 2 = 26 features
```

## Testing Strategy

| Test Type | Location | Command |
|---|---|---|
| Unit tests (features) | `vibesentinel-features/src/*.rs` | `cargo test -p vibesentinel-features` |
| Unit tests (model) | `vibesentinel-model/src/*.rs` | `cargo test -p vibesentinel-model` |
| Dataset tests | `vibesentinel-trainer/src/dataset.rs` | `cargo test -p vibesentinel-trainer` |
| Golden vector parity | `vibesentinel-model/src/arch.rs` | `cargo test -p vibesentinel-model -- --ignored` |
| All core | — | `cargo test -p vibesentinel-features -p vibesentinel-model` |

## Phases Overview

| Phase | Focus | Status |
|---|---|---|
| 1 | Core: features, model, trainer, firmware | Done |
| 2 | Connectivity: WiFi, MQTT, OTA, SD card | Planned |
| 3 | Robustness: multi-model, adaptive threshold, multi-sensor | Planned |
| 4 | Advanced ML: INT8 quantization, fine-tuning, sensor fusion | Planned |
| 5 | Product: dashboard, notifications, multi-device, CI/CD | Planned |
