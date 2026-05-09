#!/bin/bash
set -e

echo "=== VibeSentinel Firmware Build ==="

# 1. Source ESP-IDF environment if not already set
if [ -z "$IDF_PATH" ]; then
    if [ -f "$HOME/esp/esp-idf/export.sh" ]; then
        echo "Sourcing ESP-IDF..."
        . "$HOME/esp/esp-idf/export.sh"
    else
        echo "Error: ESP-IDF not found. Run ./scripts/bootstrap.sh first."
        exit 1
    fi
fi

# 2. Run the build
cargo +esp build --release -p vibesentinel-firmware \
  --target xtensa-esp32s3-espidf \
  -Zbuild-std=std,panic_abort

echo ""
echo "=== Build Successful ==="
ls -lh target/xtensa-esp32s3-espidf/release/vibesentinel-firmware
