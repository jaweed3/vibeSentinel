#!/bin/bash
set -e

echo "=== VibeSentinel Bootstrap — Environment Setup ==="

# 1. Install pyenv and Python dependencies (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Installing system dependencies via Homebrew..."
    brew install pyenv openssl readline sqlite3 xz zlib tcl-tk
fi

# 2. Setup Python 3.11 via pyenv
export PATH="$HOME/.pyenv/bin:$PATH"
eval "$(pyenv init -)"

if ! pyenv versions | grep -q "3.11.9"; then
    echo "Installing Python 3.11.9 (this may take a while)..."
    PYTHON_CONFIGURE_OPTS="--enable-framework" pyenv install 3.11.9
fi
pyenv local 3.11.9

# 3. Setup ESP-IDF
mkdir -p ~/esp
if [ ! -d "$HOME/esp/esp-idf" ]; then
    echo "Cloning ESP-IDF v5.1..."
    git clone -b v5.1 --depth 1 --recursive https://github.com/espressif/esp-idf.git ~/esp/esp-idf
fi

echo "Installing ESP-IDF tools for ESP32-S3..."
cd ~/esp/esp-idf
./install.sh esp32s3

# 4. Create local .cargo/config.toml
echo "Configuring project .cargo/config.toml..."
cd - > /dev/null
mkdir -p .cargo
cat <<EOF > .cargo/config.toml
[env]
# Force build script to use your clean pyenv Python
PYTHON = { value = "$HOME/.pyenv/versions/3.11.9/bin/python3", force = true }
# Point to your manual ESP-IDF install
IDF_PATH = { value = "$HOME/esp/esp-idf", force = true }
ESP_IDF_VERSION = { value = "v5.1", force = true }
EOF

echo "=== Bootstrap Complete ==="
echo "Run '. ~/esp/esp-idf/export.sh' then './build.sh' to compile firmware."
