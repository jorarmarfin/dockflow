#!/usr/bin/env bash
# Levanta DocFlow en modo desarrollo (ventana de escritorio con recarga en caliente).
# Uso: ./run.sh            -> app de escritorio (Tauri)
#      ./run.sh web        -> solo interfaz en el navegador (sin motor Rust)
#      ./run.sh test       -> pruebas (Rust + frontend)
set -euo pipefail
cd "$(dirname "$0")"

need() { command -v "$1" >/dev/null 2>&1 || { echo "Falta '$1'. Ver README (Requisitos)."; exit 1; }; }
need node; need npm; need cargo

[ -d node_modules ] || npm install

case "${1:-desktop}" in
  web)  npm run dev ;;
  test) (cd src-tauri && cargo test --release); npm test --if-present ;;
  *)
    if [ "$(uname -s)" = "Linux" ] && ! pkg-config --exists webkit2gtk-4.1 2>/dev/null; then
      echo "Falta libwebkit2gtk-4.1-dev (sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev build-essential libssl-dev librsvg2-dev)"; exit 1
    fi
    npm run tauri dev ;;
esac
