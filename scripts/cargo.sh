#!/usr/bin/env bash
# Menjalankan cargo di container utd-rs-dev (rust:1-bookworm + rustfmt/clippy; mesin dev tidak punya compiler C).
# Image dibuat otomatis bila belum ada. Cache di .dev/.
# Contoh: scripts/cargo.sh test ; scripts/cargo.sh clippy --all-targets
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p .dev/cargo
docker image inspect utd-rs-dev >/dev/null 2>&1 ||
  printf 'FROM rust:1-bookworm\nRUN rustup component add rustfmt clippy\n' | docker build -q -t utd-rs-dev - >/dev/null
exec docker run --rm -i --network host -u "$(id -u):$(id -g)" \
  -e CARGO_HOME=/src/.dev/cargo -e CARGO_TARGET_DIR=/src/target \
  -e DATABASE_URL="${DATABASE_URL:-postgres://postgres:utd-dev@localhost:54330/utdrs}" \
  -e RUST_LOG -e RUST_BACKTRACE -e UTD_COOKIE_SECURE -e UTD_ALAMAT -e UTD_DIR_WEB \
  -v "$PWD:/src" -w /src utd-rs-dev cargo "$@"
