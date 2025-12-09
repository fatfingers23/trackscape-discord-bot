FROM rust:1.86-bookworm AS worker-builder
WORKDIR /app
COPY . /app
RUN cargo build --bin trackscape-discord-job-worker --release
#
FROM rust:1.86-slim-bookworm AS worker
COPY --from=worker-builder /app/target/release/trackscape-discord-job-worker /usr/local/bin/trackscape-discord-job-worker
CMD ["trackscape-discord-job-worker"]