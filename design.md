# VibeSentinel Design Decisions

> Architecture decisions, trade-offs, and rationale.

---

## ADR-001: Why Rust (not Python/C)

| Factor | Decision |
|---|---|
| Language | Rust (nightly) |
| Target | Embedded (ESP32-S3) |
| Training | Burn framework (Rust-native) |

**Rationale:**
- Single language across training → inference → firmware eliminates serialization bugs
- `no_std` support enables running without OS on microcontrollers
- `libm` provides deterministic floating-point math across architectures
- Panic safety prevents silent memory corruption in production

**Trade-off:** Slower iteration than Python during ML prototyping. Mitigated by Burn's ergonomic tensor API.

---

## ADR-002: Autoencoder Architecture (not CNN, not LSTM)

| Aspect | Autoencoder | CNN | LSTM |
|---|---|---|---|
| Params | 524 | ~5000+ | ~3000+ |
| Forward pass | < 1ms | ~5ms | ~10ms |
| Training data | Normal only | Normal + anomaly | Normal + anomaly |
| Interpretability | MSE score | Latent features | Temporal patterns |

**Rationale:**
- **524 parameters** fits in 2KB — entire model + weights in L1 cache
- No need for anomaly-labeled data (unsupervised — train on normal only)
- MSE reconstruction error is intuitive: high = broken
- Single forward pass = O(n) dot products, no temporal state

**Trade-off:** Can't detect time-series patterns (e.g., frequency drift over minutes). Acceptable for vibration monitoring where each window is independent.

---

## ADR-003: FEATURE_DIM=26 with INPUT_DIM=20

The extractor computes 26 features, but the autoencoder uses 20.

**Rationale:**
- 20 input features is the minimum needed for accurate anomaly detection
- The extra 6 features (reserved) allow future expansion without retraining the full pipeline
- Current 20 features: 3 axes × (RMS, Peak, Kurtosis, Crest Factor, FFT[0], FFT[1]) + axial/radial ratio + total RMS

**Trade-off:** Wasted computation for 6 unused features. Cost: ~6 extra multiply-adds per window (~0.5μs).

---

## ADR-004: Static Weights (not runtime loading)

Weights are compiled into the binary at build time.

**Rationale:**
- Zero heap allocation — arrays are `static` in `.rodata`
- Zero initialization cost — no flash reads at startup
- Impossible to load corrupted weights at runtime
- Simpler deployment: one `.bin` file

**Trade-off:** Weights can't be updated without firmware reflash. Mitigated by Phase 2 OTA updates.

---

## ADR-005: 3-Sigma Threshold Calibration

Threshold = `mean(val_mse) + 3.0 × std(val_mse)`

**Rationale:**
- Assumes validation reconstruction errors follow a normal distribution
- 3-sigma captures ~99.7% of normal data as "normal"
- Adjustable via `--sigma` CLI flag
- Matches standard statistical process control (SPC) methodology

**Trade-off:** Non-normal error distributions may need different k values. The `--sigma` flag allows tuning per deployment.

---

## ADR-006: Fixed-Point Math for Training (Burn NDArray)

Training uses Burn with NDArray (CPU) backend, not GPU (WGPU).

**Rationale:**
- No GPU dependency — runs on any laptop
- Deterministic results across runs (unlike GPU which has non-determinism)
- NDArray is simpler to debug than GPU-accelerated

**Trade-off:** 50-200 epochs take ~1-2 minutes on CPU vs ~10 seconds on GPU. Acceptable since training is done once per deployment.

---

## ADR-007: Sliding Window (128 samples @ 200Hz = 640ms)

| Parameter | Value | Reason |
|---|---|---|
| Window size | 128 samples | Power of 2 for FFT |
| Sample rate | 200 Hz | Captures vibration up to 100 Hz (Nyquist) |
| Window duration | 640 ms | Enough for 32+ cycles of 50 Hz mains |
| Step size | 64 samples (320ms) | 50% overlap for temporal continuity |

---

## ADR-008: Error Code Design (E001–E010)

Structured error codes let automated systems parse failures without NLP.

| Range | Category |
|---|---|
| E001–E002 | I2C / hardware initialization |
| E003–E005 | Sensor health / runtime |
| E006–E008 | Inference pipeline errors |
| E009–E010 | Recovery and saturation |

**Principle:** Every error code maps to a unique root cause. No generic "unknown error" codes.

---

## ADR-009: Firmware Main Loop Design

```rust
loop {
    sample();          // 5ms interval, timer-driven
    extract();         // 128-sample window → 26 features
    check_health();    // frozen sensor, NaN detection
    normalize();       // Z-score, clip [-3, 3]
    infer();           // Autoencoder forward pass
    detect();          // MSE > threshold?
    alert();           // GPIO LED
    log();             // Serial output (state-delta only)
}
```

**Rationale:** Synchronous, single-threaded, no async. Predictable 5ms cycle time. Total forward pass < 1ms leaves 4ms margin.

---

## ADR-010: Golden Vector Cross-Architecture Tests

Before deploying new weights to ESP32, verify that x86 output == Xtensa output.

**Method:**
1. Trainer computes 5 golden inputs + expected reconstructions
2. Exports to `weights.rs` as `GOLDEN_INPUTS` and `GOLDEN_RECONSTRUCTIONS`
3. Model crate's `test_cross_architecture_parity` runs the same inputs through `no_std` code
4. Floating-point tolerance: `1e-5`

**Why:** x86 uses SSE/AVX, Xtensa uses IEEE 754 with different rounding. Without golden vectors, a model that works on desktop might fail on hardware.

---

## Future Design Considerations

| Phase | Design Decision Needed |
|---|---|
| 2 | MQTT topic naming convention |
| 2 | OTA partition scheme |
| 3 | Model selection strategy (when to switch) |
| 4 | INT8 calibration dataset requirements |
| 4 | Fine-tuning learning rate for on-device |
| 5 | Dashboard API protocol (REST vs WebSocket) |
| 5 | Multi-device time synchronization |
