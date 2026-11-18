#!/usr/bin/env bash
# Menjalankan cargo di container rust:1-bookworm (mesin dev tidak punya compiler C). Cache di .dev/.
# Contoh: scripts/cargo.sh test ; scripts/cargo.sh clippy --all-targets
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p .dev/cargo
exec docker run --rm -i --network host -u "$(id -u):$(id -g)" \
  -e CARGO_HOME=/src/.dev/cargo -e CARGO_TARGET_DIR=/src/target \
  -e DATABASE_URL="${DATABASE_URL:-postgres://postgres:utd-dev@localhost:54330/utdrs}" \
  -e RUST_LOG -e RUST_BACKTRACE \
  -v "$PWD:/src" -w /src rust:1-bookworm cargo "$@"
