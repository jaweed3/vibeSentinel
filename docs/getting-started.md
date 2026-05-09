# 🚦 Getting Started

Follow these steps to set up the environment and deploy VibeSentinel.

## 🛠️ Prerequisites
*   **Operating System**: macOS or Linux (preferred).
*   **Rust**: Nightly version (`rustup toolchain install nightly`).
*   **ESP Tools**: `espup` and `espflash` installed.
*   **Hardware**: ESP32-S3 and an LSM6DS3 Accelerometer.

## ⚡ Setup
1.  **Initialize Environment**:
    Our bootstrap script fixes common Python issues on macOS and installs the correct ESP-IDF version.
    ```bash
    ./scripts/bootstrap.sh
    ```

2.  **Verify Core Components**:
    Run tests to ensure math and model logic are correct:
    ```bash
    cargo test -p vibesentinel-features
    cargo test -p vibesentinel-model
    ```

## 🧠 Training
To train the model on synthetic or real data:
```bash
# Optional: add your CSV to data/normal.csv
cargo run --release -p vibesentinel-trainer
```
The trainer will:
1.  Compute Mean/Std of features.
2.  Train the Autoencoder for 50+ epochs.
3.  Calibrate the anomaly threshold.
4.  Export everything to `crates/vibesentinel-model/src/weights.rs`.

## 📡 Deployment
Compile and flash the firmware:
```bash
./scripts/build.sh
espflash flash target/xtensa-esp32s3-espidf/release/vibesentinel-firmware
```
Monitor logs:
```bash
espflash monitor
```
Look for lines like:
`MSE: 0.1245 | Threshold: 1.8693 | NORMAL`
