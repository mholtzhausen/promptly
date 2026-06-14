#!/bin/bash
set -e

# Get script directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "$DIR"

echo "Ensuring build dependencies..."
mkdir -p "$DIR/lib"
if [ ! -f "$DIR/lib/libxdo.so" ]; then
    if [ -f /usr/lib/x86_64-linux-gnu/libxdo.so.3 ]; then
        ln -sf /usr/lib/x86_64-linux-gnu/libxdo.so.3 "$DIR/lib/libxdo.so"
    elif [ -f /usr/lib/libxdo.so.3 ]; then
        ln -sf /usr/lib/libxdo.so.3 "$DIR/lib/libxdo.so"
    else
        echo "Error: libxdo.so.3 not found on system. Please install libxdo3."
        exit 1
    fi
fi

echo "Building prompt_tray in release mode..."
RUSTFLAGS="-L ./lib" cargo build --release

echo "Running prompt_tray..."
exec "$DIR/target/release/prompt_tray"
