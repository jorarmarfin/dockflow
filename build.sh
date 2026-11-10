#!/usr/bin/env bash
# Compila DocFlow de forma NATIVA en el sistema donde se ejecuta.
#   Linux            -> .deb + .AppImage
#   Windows (Git Bash/MSYS) -> instalador Inno Setup .exe (requiere Inno Setup 6)
# Salida: carpeta dist-release/ con los paquetes y SHA256SUMS.txt
set -euo pipefail
cd "$(dirname "$0")"

need() { command -v "$1" >/dev/null 2>&1 || { echo "Falta '$1'. Ver README (Requisitos)."; exit 1; }; }
need node; need npm; need cargo

case "$(uname -s)" in
  Linux)                BUNDLES="deb,appimage" ;;
  MINGW*|MSYS*|CYGWIN*) BUNDLES="" ;;
  *) echo "Sistema no soportado: $(uname -s)"; exit 1 ;;
esac

[ -d node_modules ] || npm ci 2>/dev/null || npm install

echo "==> Pruebas del motor"
(cd src-tauri && cargo test --release)

OUT=dist-release
rm -rf "$OUT"; mkdir -p "$OUT"

if [ -z "$BUNDLES" ]; then
  echo "==> Compilando ejecutable (Windows)"
  npm run tauri build -- --no-bundle
  ISCC="$(command -v iscc || true)"
  for c in "/c/Program Files (x86)/Inno Setup 6/ISCC.exe" "/c/Program Files/Inno Setup 6/ISCC.exe"; do
    [ -n "$ISCC" ] || { [ -x "$c" ] && ISCC="$c"; }
  done
  [ -n "$ISCC" ] || { echo "Falta Inno Setup 6 (https://jrsoftware.org/isdl.php)"; exit 1; }
  VERSION="$(node -p "require('./src-tauri/tauri.conf.json').version")"
  echo "==> Instalador Inno Setup v$VERSION"
  MSYS2_ARG_CONV_EXCL='*' "$ISCC" "/DAppVersion=$VERSION" installer/docflow.iss
else
  echo "==> Empaquetando ($BUNDLES)"
  # AppImage en algunas distros necesita esto para no fallar al empaquetar
  export NO_STRIP=true
  npm run tauri build -- --bundles "$BUNDLES"
  find src-tauri/target/release/bundle -type f \( -name '*.deb' -o -name '*.AppImage' \) -exec cp {} "$OUT"/ \;
fi
(cd "$OUT" && { sha256sum * 2>/dev/null || shasum -a 256 *; } > SHA256SUMS.txt)
echo "==> Listo:"; ls -lh "$OUT"
