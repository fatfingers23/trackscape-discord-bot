FROM rust:1.78-bookworm AS worker-builder
WORKDIR /app
COPY . /app
RUN cargo build --bin trackscape-discord-cron-job-worker --release
#
FROM rust:1.78-slim-bookworm AS worker
COPY --from=worker-builder /app/target/release/trackscape-discord-cron-job-worker /usr/local/bin/trackscape-discord-cron-job-worker
CMD ["trackscape-discord-cron-job-worker"]