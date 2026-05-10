#!/usr/bin/env bash
set -euo pipefail

DATA_PATH="${1:-data/normal_vibration.csv}"
WEIGHTS_PATH="crates/vibesentinel-model/src/weights.rs"

echo "==> [1/3] Training model on: $DATA_PATH"
cargo run --release -p vibesentinel-trainer -- \
    --data "$DATA_PATH" \
    --output "$WEIGHTS_PATH" \
    --epochs 200 \
    --lr 0.001 \
    --sigma 3.0

echo "==> [2/3] Building ESP32-S3 firmware..."
cargo +esp build --release -p vibesentinel-firmware \
    --target xtensa-esp32s3-espidf \
    -Zbuild-std=std,panic_abort

echo "==> [3/3] Firmware built successfully"
ls -lh target/xtensa-esp32s3-espidf/release/vibesentinel-firmware
echo ""
echo "To flash: espflash flash target/xtensa-esp32s3-espidf/release/vibesentinel-firmware --monitor"
