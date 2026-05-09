# 📈 Current State & Future Implementation

## ✅ Current State (v0.1.0)
*   **Fully Functional Core**: Feature extraction (Time + Frequency domain) and Autoencoder inference are implemented and tested in `no_std`.
*   **Training Pipeline**: Integrated with `Burn` framework for desktop training and seamless weight export.
*   **Firmware**: Basic ESP32-S3 firmware with I2C IMU support and GPIO alerting is ready for cross-compilation.
*   **Automation**: Bootstrap and build scripts handle the complex ESP-IDF environment setup.

## 🔮 Future Implementation (Roadmap)

### Phase 1: Robustness & connectivity
*   [ ] **WiFi Telemetry**: Stream MSE and Feature vectors via MQTT to a central dashboard.
*   [ ] **OTA Updates**: Over-the-air model updates without re-flashing the entire firmware.
*   [ ] **Multiple Sensor Support**: Support for SPI-based sensors and higher G-scale accelerometers (e.g., ADXL345).

### Phase 2: Advanced ML
*   [ ] **Multi-Model Ensemble**: Separate models for different machine speeds/states.
*   [ ] **On-Device Fine-tuning**: Use the latent space representations for local adaptation.
*   [ ] **Quantization**: Implement Fixed-point math (INT8) for even lower latency and memory footprint.

### Phase 3: Monitoring UI
*   [ ] **Web Dashboard**: A React/Next.js dashboard for real-time vibration visualization.
*   [ ] **Historical Analysis**: Correlation between anomaly scores and actual maintenance logs.
