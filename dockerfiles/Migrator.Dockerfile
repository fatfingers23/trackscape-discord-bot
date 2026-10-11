# Builds a fully static (musl) linux/amd64 binary of the one-off
# MongoDB -> Postgres migrator and exports it on its own.
#
# Static on purpose: the result runs on any Ubuntu LTS (18.04, 20.04, 22.04,
# 24.04, ...) without matching the host's glibc, and needs no libssl/libcrypto
# installed at runtime.
#
#   docker buildx build --platform linux/amd64 \
#     --target export --output type=local,dest=./dist \
#     -f dockerfiles/Migrator.Dockerfile .
#
# The binary lands in ./dist/trackscape-discord-migrator.
FROM rust:1.99-alpine AS builder

# musl toolchain plus a static OpenSSL (reqwest's native-tls needs it), so the
# final binary links nothing at runtime.
RUN apk add --no-cache build-base pkgconf openssl-dev openssl-libs-static

# SQLX_OFFLINE uses the committed .sqlx metadata (no live DB at build time).
# OPENSSL_STATIC makes openssl-sys link Alpine's static OpenSSL instead of the
# shared one.
ENV SQLX_OFFLINE=true \
    OPENSSL_STATIC=1

WORKDIR /app
COPY . /app
RUN cargo build --bin trackscape-discord-migrator --release --locked

# Export stage: just the single static binary.
FROM scratch AS export
COPY --from=builder /app/target/release/trackscape-discord-migrator /
