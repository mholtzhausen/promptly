#!/bin/bash
set -e

# Get script directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "$DIR"

echo "Installing frontend dependencies..."
npm --prefix "$DIR/frontend" ci

echo "Building frontend..."
npm --prefix "$DIR/frontend" run build

echo "Building promptly in release mode..."
cargo build --release

echo "Running promptly..."
exec "$DIR/target/release/promptly"
