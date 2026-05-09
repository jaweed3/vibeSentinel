# 🛡️ VibeSentinel: Edge Anomaly Detection

> **Predictive Maintenance at the Edge.** A complete Rust ecosystem for vibration-based anomaly detection using Autoencoders, running directly on ESP32-S3.

---

## 📖 Documentation

*   [**🌐 Overview**](./docs/overview.md): Project goals, target problem, and benefits.
*   [**🏗️ Architecture**](./docs/architecture.md): Deep dive into crate structure and data flow.
*   [**🚦 Getting Started**](./docs/getting-started.md): Prerequisites, installation, and deployment.
*   [**📈 Current State & Roadmap**](./docs/current-state-future-implementation.md): Milestone tracking and future vision.

---

## 🚀 Overview

VibeSentinel is an end-to-end industrial IoT solution designed to monitor machinery health via vibration analysis. It bypasses the cloud for real-time inference, using a lightweight Deep Learning model (Autoencoder) to detect anomalies directly on the hardware.

### Key Features
*   **🦀 100% Pure Rust**: From training (desktop) to inference (embedded).
*   **🧠 Autoencoder Architecture**: Learns the "normal" vibration signature; high reconstruction error signals machine failure.
*   **⚡ Ultra-Low Latency**: Local feature extraction (FFT + Stats) and neural inference.
*   **🛰️ No-Std Core**: Core signal processing and model crates are `#![no_std]` for maximum portability.

---

## 🏗️ Architecture

The project is structured as a Cargo Workspace:

| Crate | Responsibility | Environment |
| :--- | :--- | :--- |
| [`vibesentinel-features`](./crates/vibesentinel-features) | FFT, Kurtosis, RMS, and sliding window management. | `no_std` |
| [`vibesentinel-model`](./crates/vibesentinel-model) | Matrix math, activations, and the 20-10-4-10-20 AE architecture. | `no_std` |
| [`vibesentinel-trainer`](./crates/vibesentinel-trainer) | Model training using **Burn** and weight exportation to Rust code. | `std` (Desktop) |
| [`vibesentinel-firmware`](./crates/vibesentinel-firmware) | ESP32-S3 main loop, LSM6DS3 IMU driver, and LED alerting. | `esp-idf` |

---

## 🚦 Getting Started

### 1. Requirements
*   **Rust**: Nightly toolchain.
*   **Hardware**: ESP32-S3 + LSM6DS3 (I2C) or similar IMU.
*   **Python**: 3.11 (for ESP-IDF build system).

### 2. Quick Setup (Desktop)
Use our bootstrap script to fix Python issues and setup the ESP-IDF environment:
```bash
chmod +x scripts/*.sh
./scripts/bootstrap.sh
```

### 3. Training the Model
If you want to re-train the model with your own data or synthetic data:
```bash
cargo run --release -p vibesentinel-trainer
```
This will update `crates/vibesentinel-model/src/weights.rs` with fresh weights and a calibrated `ANOMALY_THRESHOLD`.

### 4. Flashing Firmware
Build and flash to your ESP32-S3:
```bash
./scripts/build.sh
# To flash (ensure espflash is installed):
espflash flash target/xtensa-esp32s3-espidf/release/vibesentinel-firmware
```

---

## 📊 Pipeline Logic

1.  **Sampling**: 200Hz acceleration data from X, Y, Z axes.
2.  **Windowing**: 128-sample sliding window.
3.  **Feature Extraction**:
    *   **Time Domain**: RMS, Peak-to-Peak, Kurtosis, Crest Factor.
    *   **Frequency Domain**: FFT bin magnitudes.
    *   **Relational**: Axial/Radial energy ratios.
4.  **Inference**:
    *   Normalize features using training-time Mean/Std.
    *   Forward pass through the Autoencoder.
    *   Compare Input vs. Output (MSE).
5.  **Alert**: If `MSE > ANOMALY_THRESHOLD`, trigger GPIO Alert.

---

## 🛠️ Developed with
*   **Burn**: For deep learning training.
*   **microfft**: For embedded spectral analysis.
*   **ESP-IDF-HAL**: For hardware abstraction.

---
*Developed by Gemini CLI for jaweed3.*
