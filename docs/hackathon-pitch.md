# VibeSentinel — Hackathon Pitch

> **Edge AI for Predictive Maintenance. 100% Rust. Real-time. No Cloud.**

---

## The Problem

**$1 trillion** — that's the annual cost of unplanned downtime in industrial manufacturing. Most factories run on "fix it when it breaks" maintenance. Even scheduled maintenance wastes billions replacing healthy parts.

Current "smart" solutions send raw vibration data to the cloud. This requires WiFi, costs bandwidth, adds latency, and creates privacy headaches — all at scale, it's infeasible for thousands of machines.

## The Solution

**VibeSentinel** puts a neural network on a $10 ESP32-S3 microcontroller. It learns a machine's "normal" vibration signature, then detects anomalies in real-time — right on the sensor. No cloud, no latency, no bandwidth.

A **Deep Learning Autoencoder** compresses 128 vibration samples into a 6-dimensional "fingerprint" and reconstructs them. When reconstruction error spikes, something is wrong — bearing wear, imbalance, misalignment. The system blinks an LED instantly.

## Why This Wins

| Category | Edge |
|---|---|
| **100% Rust** | Training (Burn) → Firmware (no_std). Zero Python on device. Memory safe. |
| **Truly Edge** | Inference in ~microseconds on a $10 MCU. No WiFi needed. |
| **Privacy by Design** | Data never leaves the sensor. No cloud dependency. |
| **Full Pipeline** | Data generation → training → firmware export → real-time inference. |
| **Reproducible** | Golden vector tests guarantee trainer output matches embedded inference bit-exactly. |
| **Industrial Ready** | Structured error codes (E001-E010), watchdog, I2C recovery, saturation detection. |

## Demo Flow (5 minutes)

```
1. Sensor on running motor      → "Normal" — green LED, MSE low
2. Tap motor with wrench         → "Anomaly!" — red LED, MSE spikes
3. Serial output                  → Shows top-3 failing features, MSE value
4. Health report (60s)           → Timing stats, anomaly rate, error counts
```

The audience sees a physical motor, a tiny dev board, and instant visual feedback. No cloud dashboard loading screen.

## Competition Categories

- **Industrial IoT / Industry 4.0** — direct hit
- **Edge AI / TinyML** — exact sweet spot
- **Rust Embedded** — unique differentiator (most teams use Python/TensorFlow Lite)
- **Sustainability / Green Tech** — predictive maintenance prevents waste
- **Hardware Hack** — ESP32 + IMU sensor

## Expansion Ideas (to stand out)

| Feature | Impact | Effort |
|---|---|---|
| **BLE Dashboard** | Show MSE live on a phone app | Medium (ESP32 BLE is well-supported) |
| **Multi-sensor sync** | 3x ESP32s on one machine, triangulate fault location | Medium |
| **On-device learning** | Adapt threshold per machine without retraining | Hard (research-grade) |
| **Pre-anomaly buffer** | Dump 1s of raw data before anomaly for post-mortem | Low (circular buffer) |
| **Web-based dashboard** | React dashboard over WebSerial or MQTT | Medium |
| **Anomaly sound** | Play different tones for different fault types | Low (PWM buzzer) |

The **BLE Dashboard** is the biggest bang-for-buck — turns a "blinking LED" into a "smartphone alert" which judges love.

## Team Roles (ideal: 3 people)

| Role | Skills | Covers |
|---|---|---|
| **Embedded Rust** | ESP32, I2C, no_std | Firmware, sensor integration, optimization |
| **ML / DSP** | Autoencoders, signal processing, Burn | Training pipeline, feature engineering, calibration |
| **Frontend / Demo** | BLE, React, or Python viz | Live dashboard, demo setup, pitch deck |

With 2 people: merge ML + Embedded (the pipeline is already built), keep the frontend person.

## Pitch One-Liners

- "We put a neural network on a $10 chip so factories never have to guess when a machine will break."
- "No cloud. No WiFi. Just a blinking red LED that says 'fix me now' before the bearing explodes."
- "Most TinyML demos blink an LED. We predict mechanical failure at the edge."
- "Our model trains in Rust, exports to Rust, and infers in Rust — the same code from laptop to ESP32."

## Key Technical Differentiators

- **Cross-architecture parity tests** — golden vectors from the Burn trainer are compiled into the no_std firmware as test assertions. If the floating-point math diverges by even 1e-5, CI fails.
- **Hann windowing + 4-bin FFT** — real signal processing, not just raw samples. Catches spectrum-level faults.
- **26-dim feature vector** — RMS, peak, kurtosis, crest factor + 4 FFT bins per axis + cross-axis ratios.
- **Structured error codes** — every failure mode has a code, description, and fix. Field-debuggable.
- **Diagnostic health reports** — per-window timing stats, MSE distribution, anomaly sessions. Judges can see the system self-monitoring.

## What Judges Look For

| Criteria | How VibeSentinel Delivers |
|---|---|
| **Completeness** | Hardware → firmware → ML → data pipeline → visualization. End-to-end. |
| **Technical difficulty** | no_std neural inference, I2C bit-banging, FFT on MCU. Non-trivial. |
| **Real-world impact** | Predictive maintenance is a billion-dollar problem. Tangible. |
| **Demo quality** | Physical motor + LED + serial output. Works offline. No "WiFi dropped" excuses. |
| **Novelty** | Most teams use Python/Edge Impulse. A pure Rust pipeline is rare and memorable. |

## Getting Ready Checklist

- [ ] Train model on real recorded vibration data (not just synthetic sine waves)
- [ ] Record a "normal" and "fault" demo video
- [ ] Prepare the physical demo: motor, ESP32, IMU, LED, USB power bank
- [ ] Print error code reference card for judges
- [ ] Have `cargo run -p vibesentinel-trainer` ready to re-train live if asked
- [ ] Practice the 5-minute demo with timer
- [ ] Add BLE dashboard (if going for top prize)
