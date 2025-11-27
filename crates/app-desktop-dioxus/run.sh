#!/bin/bash

# Script para ejecutar la app Dioxus Desktop

set -e

echo "🚀 Ejecutando App Desktop Dioxus"
echo ""

# Variables de entorno opcionales
export BACKEND_URL="${BACKEND_URL:-http://localhost:3000/api}"
export RUST_LOG="${RUST_LOG:-info}"

echo "📋 Configuración:"
echo "   Backend: $BACKEND_URL"
echo "   Log level: $RUST_LOG"
echo ""

# Ejecutar app
cd "$(dirname "$0")"

if [ "$1" == "dev" ]; then
    echo "🔥 Modo desarrollo (con hot reload)"
    dx serve --hot-reload
else
    echo "🏃 Ejecutando app..."
    cargo run --release
fi
