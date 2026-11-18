#!/usr/bin/env bash
# Server development di localhost:8080 (database dev dari scripts/dev-db.sh). Argumen diteruskan ke binary,
# mis. `scripts/dev-server.sh seed-demo`. Antarmuka: `cd web && npm run dev` (proxy /api ke 8080).
set -euo pipefail
cd "$(dirname "$0")/.."
UTD_COOKIE_SECURE=0 UTD_ALAMAT=127.0.0.1:8080 exec scripts/cargo.sh run --quiet -- "$@"
