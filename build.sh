#!/usr/bin/env bash
# Compila DocFlow de forma NATIVA en el sistema donde se ejecuta.
#   Linux            -> .deb + .AppImage
#   Windows (Git Bash/MSYS) -> instalador NSIS .exe
# Salida: carpeta dist-release/ con los paquetes y SHA256SUMS.txt
set -euo pipefail
cd "$(dirname "$0")"

need() { command -v "$1" >/dev/null 2>&1 || { echo "Falta '$1'. Ver README (Requisitos)."; exit 1; }; }
need node; need npm; need cargo

case "$(uname -s)" in
  Linux)                BUNDLES="deb,appimage" ;;
  MINGW*|MSYS*|CYGWIN*) BUNDLES="nsis" ;;
  *) echo "Sistema no soportado: $(uname -s)"; exit 1 ;;
esac

[ -d node_modules ] || npm ci 2>/dev/null || npm install

echo "==> Pruebas del motor"
(cd src-tauri && cargo test --release)

echo "==> Empaquetando ($BUNDLES)"
# AppImage en algunas distros necesita esto para no fallar al empaquetar
export NO_STRIP=true
npm run tauri build -- --bundles "$BUNDLES"

OUT=dist-release
rm -rf "$OUT"; mkdir -p "$OUT"
find src-tauri/target/release/bundle -type f \( -name '*.deb' -o -name '*.AppImage' -o -name '*-setup.exe' \) -exec cp {} "$OUT"/ \;
(cd "$OUT" && { sha256sum * 2>/dev/null || shasum -a 256 *; } > SHA256SUMS.txt)
echo "==> Listo:"; ls -lh "$OUT"
