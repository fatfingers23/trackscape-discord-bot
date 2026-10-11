FROM rust:1.99-bookworm AS bot-builder
WORKDIR /app
COPY . /app
RUN cargo build --bin trackscape-discord-bot --release --locked
#
FROM rust:1.99-slim-bookworm AS bot
COPY --from=bot-builder /app/target/release/trackscape-discord-bot /usr/local/bin/trackscape-discord-bot
CMD ["trackscape-discord-bot"]
