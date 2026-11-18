#!/usr/bin/env bash
# Membuat Pedoman-Penggunaan-utd-rs.pdf dari pedoman.html dengan Chromium headless.
# Gambar: gambar/ (khusus pedoman) dan ../tangkapan-layar/ (dipakai bersama README).
set -euo pipefail
cd "$(dirname "$0")"
CHROME=${CHROME:-$(command -v chromium || command -v google-chrome || command -v chromium-browser || \
  find "$HOME/.cache/ms-playwright" -name chrome-headless-shell -type f 2>/dev/null | head -1)}
[ -n "$CHROME" ] || { echo "Chromium tidak ditemukan; isi variabel CHROME." >&2; exit 1; }
"$CHROME" --headless --no-sandbox --disable-gpu --no-pdf-header-footer --allow-file-access-from-files \
  --virtual-time-budget=10000 --print-to-pdf="$PWD/Pedoman-Penggunaan-utd-rs.pdf" "file://$PWD/pedoman.html" 2>/dev/null
echo "Selesai: $PWD/Pedoman-Penggunaan-utd-rs.pdf"
