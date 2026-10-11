FROM rust:1.99-bookworm AS worker-builder
WORKDIR /app
COPY . /app
RUN cargo build --bin trackscape-discord-cron-job-worker --release --locked
#
FROM rust:1.99-slim-bookworm AS worker
COPY --from=worker-builder /app/target/release/trackscape-discord-cron-job-worker /usr/local/bin/trackscape-discord-cron-job-worker
CMD ["trackscape-discord-cron-job-worker"]