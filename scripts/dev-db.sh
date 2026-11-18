#!/usr/bin/env bash
# PostgreSQL development di container utd-rs-dev-db, port 127.0.0.1:54330 (sandi utd-dev, hanya lokal).
set -euo pipefail
case "${1:-start}" in
  start)
    docker start utd-rs-dev-db >/dev/null 2>&1 || docker run -d --name utd-rs-dev-db -p 127.0.0.1:54330:5432 \
      -e POSTGRES_PASSWORD=utd-dev -e POSTGRES_DB=utdrs -v utd-rs-dev-db:/var/lib/postgresql/data postgres:17-alpine >/dev/null
    until docker exec utd-rs-dev-db pg_isready -U postgres >/dev/null 2>&1; do sleep 1; done
    echo "PostgreSQL development jalan di localhost:54330 (database utdrs)" ;;
  stop) docker stop utd-rs-dev-db >/dev/null && echo "PostgreSQL development berhenti" ;;
  psql) shift; exec docker exec -it utd-rs-dev-db psql -U postgres utdrs "$@" ;;
  *) echo "pakai: $0 [start|stop|psql]" >&2; exit 1 ;;
esac
