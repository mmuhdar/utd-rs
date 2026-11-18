# Image produksi utd-rs: server Rust + antarmuka web statis dalam satu binary/direktori (NFR-17).
FROM node:24-alpine AS web
WORKDIR /src/web
COPY web/package.json web/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY web/ ./
RUN npm run build

FROM rust:1-bookworm AS server
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY migrations migrations
COPY src src
RUN cargo build --release --locked

FROM debian:bookworm-slim
COPY --from=server /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
RUN useradd --system --create-home utdrs
WORKDIR /app
COPY --from=server /src/target/release/utdrs /app/utdrs
COPY --from=web /src/web/build /app/web
USER utdrs
ENV UTD_ALAMAT=0.0.0.0:8080 \
    UTD_DIR_WEB=/app/web \
    RUST_LOG=info
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=3s --start-period=30s \
  CMD ["bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/8080 && printf 'GET /api/sehat HTTP/1.0\\r\\n\\r\\n' >&3 && grep -q '\"ok\"' <&3"]
ENTRYPOINT ["/app/utdrs"]
CMD ["serve"]
