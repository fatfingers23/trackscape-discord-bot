FROM rust:1.75-bookworm as worker-builder
WORKDIR /app
COPY . /app
RUN cargo build --bin trackscape-discord-job-worker --release
#
FROM rust:1.75-slim-bookworm as worker
COPY --from=worker-builder /app/target/release/trackscape-discord-job-worker /usr/local/bin/trackscape-discord-job-worker
CMD ["trackscape-discord-job-worker"]
